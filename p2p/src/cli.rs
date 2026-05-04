use anyhow::Result;
use clap::Parser;
use libp2p::{identity, multiaddr::Protocol, swarm::NetworkBehaviour, Multiaddr};

use crate::network_config::Network;

pub const HELP_TEMPLATE: &str = "\
{about-with-newline}
{usage-heading} {usage}

{all-args}
";

/// Shared node configuration for libp2p service binaries.
#[derive(Debug, Clone, Parser)]
#[command(
    help_template = HELP_TEMPLATE,
    next_line_help = true,
    disable_help_subcommand = true
)]
pub struct NodeOpts {
    /// Seed used to generate a deterministic Ed25519 identity.
    #[arg(long)]
    pub secret_key_seed: Option<u8>,

    /// Bind to a specific listen address instead of the default TCP and QUIC sockets.
    #[arg(long, value_name = "ADDR")]
    pub listen_address: Option<Multiaddr>,

    /// Port used when auto-binding TCP and QUIC listeners.
    #[arg(long, default_value_t = 0)]
    pub port: u16,

    /// Prefer IPv6 wildcard addresses when auto-binding listeners.
    #[arg(long)]
    pub use_ipv6: bool,
}

/// Bootnode selection for service binaries.
#[derive(Debug, Clone, Parser)]
#[command(
    help_template = HELP_TEMPLATE,
    next_line_help = true,
    disable_help_subcommand = true
)]
pub struct NetworkOpts {
    /// Optional bootstrap network whose bootnodes will be added to the DHT.
    #[arg(long)]
    pub network: Option<Network>,
}

/// Peer lookup arguments for the network client bin.
#[derive(Debug, Clone, Parser)]
#[command(
    about = "Lookup peers on the gnostr p2p network.",
    long_about = "Lookup peers either by multiaddr or by peer ID.\n\nThis client can dial a peer directly or query the DHT for a peer route. It reuses the same libp2p transport stack as the service daemon.",
    help_template = HELP_TEMPLATE,
    next_line_help = true,
    disable_help_subcommand = true,
    after_help = "Examples:\n  gnostr-p2p-client --peer 12D3KooW... --network ipfs\n  gnostr-p2p-client --multiaddr /ip4/127.0.0.1/tcp/4001"
)]
pub struct LookupOpts {
    /// Seed used to generate a deterministic Ed25519 identity.
    #[arg(long)]
    pub secret_key_seed: Option<u8>,

    /// Lookup a peer directly by dialing its multiaddr.
    #[arg(long, value_name = "ADDR", conflicts_with = "peer", required_unless_present = "peer")]
    pub multiaddr: Option<Multiaddr>,

    /// Lookup a peer through the DHT using its peer ID.
    #[arg(long, value_name = "PEER_ID", conflicts_with = "multiaddr", required_unless_present = "multiaddr")]
    pub peer: Option<String>,

    /// Optional bootstrap network whose bootnodes will be added to the DHT.
    #[arg(long)]
    pub network: Option<Network>,
}

pub fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();
}

pub fn keypair_from_seed(secret_key_seed: Option<u8>) -> identity::Keypair {
    match secret_key_seed {
        Some(seed) => {
            let mut bytes = [0u8; 32];
            bytes[0] = seed;
            identity::Keypair::ed25519_from_bytes(bytes).expect("only errors on wrong length")
        }
        None => identity::Keypair::generate_ed25519(),
    }
}

pub fn listen_default_addresses(
    swarm: &mut libp2p::Swarm<crate::behaviour::Behaviour>,
    listen_address: Option<Multiaddr>,
    port: u16,
    use_ipv6: bool,
) -> Result<()> {
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

pub fn listen_default_addresses_relay<B: NetworkBehaviour>(
    swarm: &mut libp2p::Swarm<B>,
    port: u16,
    use_ipv6: bool,
) -> Result<()> {
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
