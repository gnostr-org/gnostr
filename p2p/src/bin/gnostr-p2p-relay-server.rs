use clap::Parser;
use futures::StreamExt;
use libp2p::{
    identify, noise, ping, relay,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux,
};
use gnostr_p2p::cli;

#[derive(Debug, Parser)]
#[command(
    author,
    version,
    about = "Run a dedicated libp2p relay server.",
    long_about = "Run a relay-capable peer for libp2p circuit relay traffic.\n\nThis node is useful for peers that need an intermediate hop when direct connectivity fails.",
    help_template = cli::HELP_TEMPLATE,
    next_line_help = true,
    disable_help_subcommand = true
)]
struct Opt {
    #[command(flatten)]
    node: cli::NodeOpts,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    cli::init_tracing();

    let opt = Opt::parse();
    let keypair = cli::keypair_from_seed(opt.node.secret_key_seed);

    let mut swarm = libp2p::SwarmBuilder::with_existing_identity(keypair)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_quic()
        .with_behaviour(|key| RelayBehaviour {
            relay: relay::Behaviour::new(key.public().to_peer_id(), Default::default()),
            ping: ping::Behaviour::new(ping::Config::new()),
            identify: identify::Behaviour::new(identify::Config::new(
                "/ipfs/id/1.0.0".to_string(),
                key.public(),
            )),
        })?
        .build();

    cli::listen_default_addresses_relay(
        &mut swarm,
        opt.node.port,
        opt.node.use_ipv6,
    )?;
    while let Some(event) = swarm.next().await {
        match event {
            SwarmEvent::Behaviour(RelayBehaviourEvent::Identify(identify::Event::Received {
                info: identify::Info { observed_addr, .. },
                ..
            })) => {
                swarm.add_external_address(observed_addr);
            }
            SwarmEvent::Behaviour(event) => println!("{event:?}"),
            SwarmEvent::NewListenAddr { address, .. } => println!("relay listening on {address}"),
            _ => {}
        }
    }

    Ok(())
}

#[derive(NetworkBehaviour)]
struct RelayBehaviour {
    relay: relay::Behaviour,
    ping: ping::Behaviour,
    identify: identify::Behaviour,
}
