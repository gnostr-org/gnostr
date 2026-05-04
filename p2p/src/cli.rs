use anyhow::Result;
use clap::Parser;
use libp2p::{identity, multiaddr::Protocol, swarm::NetworkBehaviour, Multiaddr};
use sha2::{Digest, Sha256};

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
    pub secret_key_seed: Option<String>,

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
    pub secret_key_seed: Option<String>,

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

pub fn keypair_from_seed(secret_key_seed: Option<String>) -> identity::Keypair {
    match secret_key_seed {
        Some(seed) => identity::Keypair::ed25519_from_bytes(seed_bytes(&seed))
            .expect("only errors on wrong length"),
        None => identity::Keypair::generate_ed25519(),
    }
}

fn seed_bytes(seed: &str) -> [u8; 32] {
    if seed.len() == 64 && seed.chars().all(|c| c.is_ascii_hexdigit()) {
        let mut bytes = [0u8; 32];
        for (idx, chunk) in seed.as_bytes().chunks_exact(2).enumerate() {
            bytes[idx] = u8::from_str_radix(std::str::from_utf8(chunk).unwrap(), 16)
                .expect("validated hex digest");
        }
        return bytes;
    }

    let digest = Sha256::digest(seed.as_bytes());
    digest.into()
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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use libp2p::Multiaddr;

    #[test]
    fn keypair_from_seed_is_deterministic() {
        let seed = Some("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string());
        let left = keypair_from_seed(seed.clone());
        let right = keypair_from_seed(seed);
        assert_eq!(left.public().to_peer_id(), right.public().to_peer_id());
    }

    #[test]
    fn node_opts_parses_with_explicit_address() {
        let opts = NodeOpts::parse_from([
            "gnostr-p2p",
            "--listen-address",
            "/ip4/127.0.0.1/tcp/4001",
            "--port",
            "4001",
            "--use-ipv6",
        ]);

        assert_eq!(
            opts.listen_address,
            Some("/ip4/127.0.0.1/tcp/4001".parse::<Multiaddr>().unwrap())
        );
        assert_eq!(opts.port, 4001);
        assert!(opts.use_ipv6);
    }

    #[test]
    fn lookup_opts_rejects_missing_target_and_both_targets() {
        assert!(LookupOpts::try_parse_from(["gnostr-p2p-client"]).is_err());

        assert!(LookupOpts::try_parse_from([
            "gnostr-p2p-client",
            "--peer",
            "12D3KooWQKqane1SqWJNWMQkbia9qiMWXkcHtAdfW5eVF8hbwEDw",
            "--multiaddr",
            "/ip4/127.0.0.1/tcp/4001",
        ])
        .is_err());
    }

    #[test]
    fn lookup_opts_parses_single_target() {
        let opts = LookupOpts::try_parse_from([
            "gnostr-p2p-client",
            "--peer",
            "12D3KooWQKqane1SqWJNWMQkbia9qiMWXkcHtAdfW5eVF8hbwEDw",
            "--network",
            "ipfs",
        ])
        .expect("lookup opts");

        assert_eq!(
            opts.peer.as_deref(),
            Some("12D3KooWQKqane1SqWJNWMQkbia9qiMWXkcHtAdfW5eVF8hbwEDw")
        );
        assert!(opts.multiaddr.is_none());
        assert!(matches!(opts.network, Some(Network::Ipfs)));
    }
}
