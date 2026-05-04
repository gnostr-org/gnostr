use clap::Parser;
use futures::StreamExt;
use libp2p::{
    identify, identity, ping, rendezvous,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, Multiaddr,
    noise,
};
use tracing_subscriber::EnvFilter;

#[derive(Debug, Parser)]
#[command(author, version, about = "gnostr p2p rendezvous server")]
struct Opt {
    #[arg(long)]
    secret_key_seed: Option<u8>,

    #[arg(long, default_value_t = 0)]
    port: u16,

    #[arg(long)]
    use_ipv6: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();

    let opt = Opt::parse();
    let keypair = keypair_from_seed(opt.secret_key_seed);

    let mut swarm = libp2p::SwarmBuilder::with_existing_identity(keypair)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_quic()
        .with_behaviour(|key| RendezvousBehaviour {
            ping: ping::Behaviour::new(ping::Config::new()),
            identify: identify::Behaviour::new(identify::Config::new(
                "/ipfs/id/1.0.0".to_string(),
                key.public(),
            )),
            rendezvous: rendezvous::server::Behaviour::new(
                rendezvous::server::Config::default(),
            ),
        })?
        .build();

    listen_default_addresses(&mut swarm, opt.port, opt.use_ipv6)?;

    while let Some(event) = swarm.next().await {
        match event {
            SwarmEvent::Behaviour(RendezvousBehaviourEvent::Identify(
                identify::Event::Received {
                    info: identify::Info { observed_addr, .. },
                    ..
                },
            )) => {
                swarm.add_external_address(observed_addr);
            }
            SwarmEvent::Behaviour(event) => println!("{event:?}"),
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("rendezvous listening on {address}");
            }
            _ => {}
        }
    }

    Ok(())
}

#[derive(NetworkBehaviour)]
struct RendezvousBehaviour {
    ping: ping::Behaviour,
    identify: identify::Behaviour,
    rendezvous: rendezvous::server::Behaviour,
}

fn keypair_from_seed(secret_key_seed: Option<u8>) -> identity::Keypair {
    match secret_key_seed {
        Some(seed) => {
            let mut bytes = [0u8; 32];
            bytes[0] = seed;
            identity::Keypair::ed25519_from_bytes(bytes).expect("only errors on wrong length")
        }
        None => identity::Keypair::generate_ed25519(),
    }
}

fn listen_default_addresses(
    swarm: &mut libp2p::Swarm<RendezvousBehaviour>,
    port: u16,
    use_ipv6: bool,
) -> anyhow::Result<()> {
    let ip = if use_ipv6 {
        libp2p::multiaddr::Protocol::from(std::net::Ipv6Addr::UNSPECIFIED)
    } else {
        libp2p::multiaddr::Protocol::from(std::net::Ipv4Addr::UNSPECIFIED)
    };
    let tcp = Multiaddr::empty().with(ip).with(libp2p::multiaddr::Protocol::Tcp(port));
    let quic = Multiaddr::empty()
        .with(if use_ipv6 {
            libp2p::multiaddr::Protocol::from(std::net::Ipv6Addr::UNSPECIFIED)
        } else {
            libp2p::multiaddr::Protocol::from(std::net::Ipv4Addr::UNSPECIFIED)
        })
        .with(libp2p::multiaddr::Protocol::Udp(port))
        .with(libp2p::multiaddr::Protocol::QuicV1);
    swarm.listen_on(tcp)?;
    swarm.listen_on(quic)?;
    Ok(())
}
