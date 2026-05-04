use clap::Parser;
use futures::StreamExt;
use gnostr_p2p::{
    command_handler, event_handler,
    network_config::Network,
    swarm_builder::build_swarm,
};
use libp2p::{identity, kad, multiaddr::Protocol, Multiaddr};
use tokio::io::AsyncBufReadExt;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Parser)]
#[command(author, version, about = "gnostr p2p service daemon")]
struct Opt {
    #[arg(long)]
    secret_key_seed: Option<u8>,

    #[arg(long)]
    network: Option<Network>,

    #[arg(long)]
    listen_address: Option<Multiaddr>,

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
    let mut swarm = build_swarm(keypair)?;
    swarm.behaviour_mut().kademlia.set_mode(Some(kad::Mode::Server));

    if let Some(network) = opt.network {
        for (addr, peer_id) in network.bootnodes() {
            swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
        }
    }

    listen_default_addresses(&mut swarm, opt.listen_address, opt.port, opt.use_ipv6)?;

    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin()).lines();
    loop {
        tokio::select! {
            line = stdin.next_line() => {
                let Some(line) = line? else { break; };
                if line.trim().is_empty() {
                    continue;
                }
                command_handler::handle_input_line(&mut swarm, line).await;
            }
            event = swarm.select_next_some() => {
                event_handler::handle_swarm_event(&mut swarm, event).await;
            }
        }
    }

    Ok(())
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
    swarm: &mut libp2p::Swarm<gnostr_p2p::behaviour::Behaviour>,
    listen_address: Option<Multiaddr>,
    port: u16,
    use_ipv6: bool,
) -> anyhow::Result<()> {
    if let Some(addr) = listen_address {
        swarm.listen_on(addr)?;
        return Ok(());
    }

    let ip = if use_ipv6 {
        Protocol::from(std::net::Ipv6Addr::UNSPECIFIED)
    } else {
        Protocol::from(std::net::Ipv4Addr::UNSPECIFIED)
    };

    let tcp = Multiaddr::empty().with(ip).with(Protocol::Tcp(port));
    let quic = Multiaddr::empty()
        .with(if use_ipv6 {
            Protocol::from(std::net::Ipv6Addr::UNSPECIFIED)
        } else {
            Protocol::from(std::net::Ipv4Addr::UNSPECIFIED)
        })
        .with(Protocol::Udp(port))
        .with(Protocol::QuicV1);

    swarm.listen_on(tcp)?;
    swarm.listen_on(quic)?;
    Ok(())
}
