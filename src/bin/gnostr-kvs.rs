#![doc = include_str!("../../README.md")]
use std::{
    error::Error,
    hash::{DefaultHasher, Hash, Hasher},
    str,
    str::FromStr,
    time::Duration,
};

use clap::{Parser, ValueEnum};
//use distributed_commit_list::utils;
use futures::stream::StreamExt;
use git2::{Commit, DiffFormat, ObjectType, Repository};
use libp2p::StreamProtocol;
use libp2p::{
    core::transport::Transport,
    gossipsub,
    gossipsub::IdentTopic,
    identify, identity,
    kad::{
        self,
        // Kademlia, KademliaConfig, KademliaEvent,
        store::{MemoryStore, MemoryStoreConfig},
        Config as KadConfig,
    },
    mdns, noise, ping, rendezvous,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, Swarm,
};
use tokio::{
    io::{self, AsyncBufReadExt},
    select,
};
use tracing::{debug, info, trace, warn};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Network {
    Kusama,
    Polkadot,
    Ipfs,
    Ursa,
}

impl Network {
    #[rustfmt::skip]
    fn bootnodes(&self) -> Vec<(Multiaddr, PeerId)> {
    match self {
    Network::Kusama => {
    vec![
    ("/dns/p2p.cc3-0.kusama.network/tcp/30100".parse().unwrap(), FromStr::from_str("12D3KooWDgtynm4S9M3m6ZZhXYu2RrWKdvkCSScc25xKDVSg1Sjd").unwrap()),
    ("/dns/p2p.cc3-1.kusama.network/tcp/30100".parse().unwrap(), FromStr::from_str("12D3KooWNpGriWPmf621Lza9UWU9eLLBdCFaErf6d4HSK7Bcqnv4").unwrap()),
    ("/dns/p2p.cc3-2.kusama.network/tcp/30100".parse().unwrap(), FromStr::from_str("12D3KooWLmLiB4AenmN2g2mHbhNXbUcNiGi99sAkSk1kAQedp8uE").unwrap()),
    ("/dns/p2p.cc3-3.kusama.network/tcp/30100".parse().unwrap(), FromStr::from_str("12D3KooWEGHw84b4hfvXEfyq4XWEmWCbRGuHMHQMpby4BAtZ4xJf").unwrap()),
    ("/dns/p2p.cc3-4.kusama.network/tcp/30100".parse().unwrap(), FromStr::from_str("12D3KooWF9KDPRMN8WpeyXhEeURZGP8Dmo7go1tDqi7hTYpxV9uW").unwrap()),
    ("/dns/p2p.cc3-5.kusama.network/tcp/30100".parse().unwrap(), FromStr::from_str("12D3KooWDiwMeqzvgWNreS9sV1HW3pZv1PA7QGA7HUCo7FzN5gcA").unwrap()),
    ("/dns/kusama-bootnode-0.paritytech.net/tcp/30333".parse().unwrap(), FromStr::from_str("12D3KooWSueCPH3puP2PcvqPJdNaDNF3jMZjtJtDiSy35pWrbt5h").unwrap()),
    ("/dns/kusama-bootnode-1.paritytech.net/tcp/30333".parse().unwrap(), FromStr::from_str("12D3KooWQKqane1SqWJNWMQkbia9qiMWXkcHtAdfW5eVF8hbwEDw").unwrap())
    ]
    }
    Network::Polkadot => {
    vec![
    // ("/dns/p2p.cc1-0.polkadot.network/tcp/30100".parse().unwrap(), FromStr::from_str("12D3KooWEdsXX9657ppNqqrRuaCHFvuNemasgU5msLDwSJ6WqsKc").unwrap()),
    ("/dns/p2p.cc1-1.polkadot.network/tcp/30100".parse().unwrap(), FromStr::from_str("12D3KooWAtx477KzC8LwqLjWWUG6WF4Gqp2eNXmeqAG98ehAMWYH").unwrap()),
    ("/dns/p2p.cc1-2.polkadot.network/tcp/30100".parse().unwrap(), FromStr::from_str("12D3KooWAGCCPZbr9UWGXPtBosTZo91Hb5M3hU8v6xbKgnC5LVao").unwrap()),
    ("/dns/p2p.cc1-3.polkadot.network/tcp/30100".parse().unwrap(), FromStr::from_str("12D3KooWJ4eyPowiVcPU46pXuE2cDsiAmuBKXnFcFPapm4xKFdMJ").unwrap()),
    ("/dns/p2p.cc1-4.polkadot.network/tcp/30100".parse().unwrap(), FromStr::from_str("12D3KooWNMUcqwSj38oEq1zHeGnWKmMvrCFnpMftw7JzjAtRj2rU").unwrap()),
    ("/dns/p2p.cc1-5.polkadot.network/tcp/30100".parse().unwrap(), FromStr::from_str("12D3KooWDs6LnpmWDWgZyGtcLVr3E75CoBxzg1YZUPL5Bb1zz6fM").unwrap()),
    ("/dns/cc1-0.parity.tech/tcp/30333".parse().unwrap(), FromStr::from_str("12D3KooWSz8r2WyCdsfWHgPyvD8GKQdJ1UAiRmrcrs8sQB3fe2KU").unwrap()),
    ("/dns/cc1-1.parity.tech/tcp/30333".parse().unwrap(), FromStr::from_str("12D3KooWFN2mhgpkJsDBuNuE5427AcDrsib8EoqGMZmkxWwx3Md4").unwrap()),
    ]
    }
    Network::Ipfs => {
    vec![
    ("/ip4/104.131.131.82/tcp/4001".parse().unwrap(), FromStr::from_str("QmaCpDMGvV2BGHeYERUEnRQAwe3N8SzbUtfsmvsqQLuvuJ").unwrap()),
    ("/dnsaddr/bootstrap.libp2p.io".parse().unwrap(), FromStr::from_str("QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN").unwrap()),
    ("/dnsaddr/bootstrap.libp2p.io".parse().unwrap(), FromStr::from_str("QmQCU2EcMqAqQPR2i9bChDtGNJchTbq5TbXJJ16u19uLTa").unwrap()),
    ("/dnsaddr/bootstrap.libp2p.io".parse().unwrap(), FromStr::from_str("QmbLHAnMoJPWSCR5Zhtx6BHJX9KiKNN6tpvbUcqanj75Nb").unwrap()),
    ("/dnsaddr/bootstrap.libp2p.io".parse().unwrap(), FromStr::from_str("QmcZf59bWwK5XFi76CZX8cbJ4BhTzzA3gU1ZjYZcYW3dwt").unwrap()),
    ]
    }
    Network::Ursa => {
    vec![
    ("/dns/bootstrap-node-0.ursa.earth/tcp/6009".parse().unwrap(), FromStr::from_str("12D3KooWDji7xMLia6GAsyr4oiEFD2dd3zSryqNhfxU3Grzs1r9p").unwrap()),
    ]
    }
    }
    }

    fn protocol(&self) -> Option<String> {
        match self {
            Network::Kusama => Some("/ksmcc3/kad".to_string()),
            Network::Polkadot => Some("/dot/kad".to_string()),
            Network::Ipfs => None,
            Network::Ursa => Some("/ursa/kad/0.0.1".to_string()),
        }
    }
}

// --- Top-level NetworkBehaviour Definition ---
#[derive(NetworkBehaviour)]
struct Behaviour {
    ipfs: kad::Behaviour<kad::store::MemoryStore>,
    kademlia: kad::Behaviour<kad::store::MemoryStore>,
    mdns: mdns::tokio::Behaviour,
    identify: identify::Behaviour,
    rendezvous: rendezvous::server::Behaviour,
    ping: ping::Behaviour,
    gossipsub: gossipsub::Behaviour,
}

fn init_subscriber() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let filter_layer = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info"))?;

    let fmt_layer = fmt::layer().with_target(false).with_ansi(true);

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();

    Ok(())
}

const IPFS_BOOTNODES: [&str; 6] = [
    "QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN",
    "QmQCU2EcMqAqQPR2i9bChDtGNJchTbq5TbXJJ16u19uLTa",
    "QmbLHAnMoJPWSCR5Zhtx6BHJX9KiKNN6tpvbUcqanj75Nb",
    "QmcZf59bWwK5XFi76CZX8cbJ4BhTzzA3gU1ZjYZcYW3dwt",
    "12D3KooWH1URV3uTNQW6SZ1UFDnHN8NXwznAA8JeETTBm8oimjh9",
    "12D3KooWFhXabKDwALpzqMbto94sB7rvmZ6M28hs9Y9xSopDKwQr",
];
const IPFS_PROTO_NAME: StreamProtocol = StreamProtocol::new("/ipfs/kad/1.0.0");

fn get_commit_diff_as_bytes(repo: &Repository, commit: &Commit) -> Result<Vec<u8>, git2::Error> {
    let tree = commit.tree()?;
    let parent_tree = if commit.parent_count() > 0 {
        Some(commit.parent(0)?.tree()?)
    } else {
        None
    };

    let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None)?;
    let mut buf = Vec::new();

    diff.print(DiffFormat::Patch, |_, _, line| {
        buf.extend_from_slice(line.content());
        true
    })?;

    Ok(buf)
}

fn get_commit_id_of_tag(repo: &Repository, tag_name: &str) -> Result<String, git2::Error> {
    let reference_name = format!("refs/tags/{}", tag_name);
    let reference = repo.find_reference(&reference_name)?;
    let object = reference.peel(ObjectType::Commit)?;
    Ok(object.id().to_string())
}

fn generate_ed25519(secret_key_seed: u8) -> identity::Keypair {
    // let mut bytes = [0u8; 32];
    let mut bytes: [u8; 32] = GNOSTR_SHA256; //[
                                             //    0xca, 0x45, 0xfe, 0x80, 0x0a, 0x2c, 0x3b, 0x67, //
                                             //    0x8e, 0x0a, 0x87, 0x7a, 0xa7, 0x7e, 0x36, 0x76, //
                                             //    0x34, 0x0a, 0x59, 0xc9, 0xa7, 0x61, 0x5e, 0x30, //
                                             //    0x59, 0x76, 0xfb, 0x9b, 0xa8, 0xda, 0x48, 0x06, //
                                             //];

    bytes[31] = bytes[31] ^ secret_key_seed;
    for (i, byte) in bytes.iter().enumerate() {
        // Print context: the index and value (decimal and hex) of the current byte.
        debug!("Byte {:02} [{:3} / {:#04x}]: ", i, byte, byte);

        // A `u8` has 8 bits. We iterate from 7 down to 0 to print
        // the most significant bit (MSB) first.
        for j in (0..8).rev() {
            // Create a "mask" by shifting the number 1 to the left `j` times.
            // For j=7, mask is 10000000
            // For j=0, mask is 00000001
            let mask = 1 << j;

            // Use the bitwise AND operator `&` to check if the bit at the mask's
            // position is set. If the result is not 0, the bit is 1.
            if byte & mask == 0 {
                debug!("0");
            } else {
                debug!("1");
            }
        }
        // Add a newline to separate the output for each byte.
        debug!("\n");
    }

    // bytes[31] = secret_key_seed;

    for (i, byte) in bytes.iter().enumerate() {
        // Print context: the index and value (decimal and hex) of the current byte.
        debug!("Byte {:02} [{:3} / {:#04x}]: ", i, byte, byte);

        // A `u8` has 8 bits. We iterate from 7 down to 0 to print
        // the most significant bit (MSB) first.
        for j in (0..8).rev() {
            // Create a "mask" by shifting the number 1 to the left `j` times.
            // For j=7, mask is 10000000
            // For j=0, mask is 00000001
            let mask = 1 << j;

            // Use the bitwise AND operator `&` to check if the bit at the mask's
            // position is set. If the result is not 0, the bit is 1.
            if byte & mask == 0 {
                debug!("0");
            } else {
                debug!("1");
            }
        }
        // Add a newline to separate the output for each byte.
        debug!("\n");
    }

    let keypair =
        identity::Keypair::ed25519_from_bytes(bytes).expect("only errors on wrong length");
    // println!("141:{}", keypair.public().to_peer_id());
    generate_close_peer_id(bytes.clone(), 32usize);
    keypair
}

fn generate_close_peer_id(bytes: [u8; 32], common_bits: usize) -> PeerId {
    let mut close_bytes = [0u8; 32];
    close_bytes = bytes;

    for (i, byte) in close_bytes.iter().enumerate() {
        if i < 32 {
            // Print context: the index and value (decimal and hex) of the current byte.
            debug!("Byte i={:02} [{:3} / {:#04x}]: ", i, byte, byte);

            // A `u8` has 8 bits. We iterate from 7 down to 0 to print
            // the most significant bit (MSB) first.
            for j in (0..8).rev() {
                // Create a "mask" by shifting the number 1 to the left `j` times.
                // For j=7, mask is 10000000
                // For j=0, mask is 00000001
                let mask = 1 << j;

                // Use the bitwise AND operator `&` to check if the bit at the mask's
                // position is set. If the result is not 0, the bit is 1.
                if byte & mask == 0 {
                    debug!("0");
                } else {
                    debug!("1");
                }
            }
            // Add a newline to separate the output for each byte.
            debug!("\n");
        } // end if
    }
    let mut keypair =
        identity::Keypair::ed25519_from_bytes(close_bytes).expect("only errors on wrong length");
    println!("262:{}", keypair.public().to_peer_id());

    close_bytes[31] = bytes[31] ^ 0u8;

    for (i, byte) in close_bytes.iter().enumerate() {
        // Print context: the index and value (decimal and hex) of the current byte.
        print!("265:Byte {:02} [{:3} / {:#04x}]: ", i, byte, byte);

        // A `u8` has 8 bits. We iterate from 7 down to 0 to print
        // the most significant bit (MSB) first.
        for j in (0..8).rev() {
            // Create a "mask" by shifting the number 1 to the left `j` times.
            // For j=7, mask is 10000000
            // For j=0, mask is 00000001
            let mask = 1 << j;

            // Use the bitwise AND operator `&` to check if the bit at the mask's
            // position is set. If the result is not 0, the bit is 1.
            if byte & mask == 0 {
                print!("0");
            } else {
                print!("1");
            }
        }
        // Add a newline to separate the output for each byte.
        println!();
    }

    keypair =
        identity::Keypair::ed25519_from_bytes(close_bytes).expect("only errors on wrong length");
    println!("292:{}", keypair.public().to_peer_id());
    keypair.public().to_peer_id()
}

const GNOSTR_HEX_STR: &str = "ca45fe800a2c3b678e0a877aa77e3676340a59c9a7615e305976fb9ba8da4806";

const GNOSTR_SHA256: [u8; 32] = [
    0xca, 0x45, 0xfe, 0x80, 0x0a, 0x2c, 0x3b, 0x67, 0x8e, 0x0a, 0x87, 0x7a, 0xa7, 0x7e, 0x36, 0x76,
    0x34, 0x0a, 0x59, 0xc9, 0xa7, 0x61, 0x5e, 0x30, 0x59, 0x76, 0xfb, 0x9b, 0xa8, 0xda, 0x48, 0x06,
];

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _ = init_subscriber();
    let args = Args::parse();
    warn!("args={:?}", args);

    if let Some(ref peer) = args.peer {}
    if let Some(ref multiaddr) = args.multiaddr {}
    if let Some(ref network) = args.network {}
    if let Some(ref secret) = args.secret {}

    if let Some(true) = Some(args.peer.is_some()) {}
    if let Some(true) = Some(args.multiaddr.is_some()) {}
    if let Some(true) = Some(args.network.is_some()) {}
    if let Some(true) = Some(args.secret.is_some()) {}

    let keypair: identity::Keypair = generate_ed25519(args.secret.clone().unwrap_or(0));
    let keypair_clone: identity::Keypair = generate_ed25519(args.secret.unwrap_or(0));
    let public_key = keypair.public();
    let peer_id = PeerId::from_public_key(&public_key);
    warn!("Local PeerId: {}", peer_id);
    // kad_store_config
    let kad_store_config = MemoryStoreConfig {
        max_provided_keys: usize::MAX,
        max_providers_per_key: usize::MAX,
        max_records: usize::MAX,
        max_value_bytes: usize::MAX,
    };
    let kad_memstore = MemoryStore::with_config(peer_id.clone(), kad_store_config.clone());
    let kad_config = KadConfig::default();
    let message_id_fn = |message: &gossipsub::Message| {
        let mut s = DefaultHasher::new();
        message.data.hash(&mut s);
        info!("message:\n{0:?}", message);
        info!("message.data:\n{0:?}", message.data);
        info!("message.source:\n{0:?}", message.source);
        info!("message.source:\n{0:1?}", message.source);
        info!("message.source.peer_id:\n{0:2?}", message.source.unwrap());
        // TODO https://docs.rs/gossipsub/latest/gossipsub/trait.DataTransform.html
        // send Recieved message back
        info!(
            "message.source.peer_id:\n{0:3}",
            message.source.unwrap().to_string()
        );
        info!("message.sequence_number:\n{0:?}", message.sequence_number);
        info!("message.topic:\n{0:?}", message.topic);
        info!("message.topic.hash:\n{0:0}", message.topic.clone());
        // println!("{:?}", s);
        gossipsub::MessageId::from(s.finish().to_string())
    };
    let gossipsub_config = gossipsub::ConfigBuilder::default()
        .heartbeat_interval(Duration::from_secs(1))
        .validation_mode(gossipsub::ValidationMode::Permissive)
        .message_id_fn(message_id_fn)
        .build()
        .map_err(|msg| io::Error::new(io::ErrorKind::Other, msg))?;

    let mut swarm = libp2p::SwarmBuilder::with_existing_identity(keypair)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_quic()
        .with_dns()?
        .with_behaviour(|key| {
            let kad_store_config = MemoryStoreConfig {
                max_provided_keys: usize::MAX,
                max_providers_per_key: usize::MAX,
                max_records: usize::MAX,
                max_value_bytes: usize::MAX,
            };
            let mut kad_config = kad::Config::default();
            kad_config.set_query_timeout(Duration::from_secs(120));
            kad_config.set_replication_factor(std::num::NonZeroUsize::new(20).unwrap());
            kad_config.set_publication_interval(Some(Duration::from_secs(10)));
            kad_config.disjoint_query_paths(false);
            let kad_store = MemoryStore::with_config(peer_id.clone(), kad_store_config);
            let mut ipfs_cfg = kad::Config::new(IPFS_PROTO_NAME);
            ipfs_cfg.set_query_timeout(Duration::from_secs(5 * 60));
            let ipfs_store = kad::store::MemoryStore::new(key.public().to_peer_id());
            Ok(Behaviour {
                gossipsub: gossipsub::Behaviour::new(
                    gossipsub::MessageAuthenticity::Signed(key.clone()),
                    gossipsub_config,
                )
                .expect(""),
                ipfs: kad::Behaviour::with_config(key.public().to_peer_id(), ipfs_store, ipfs_cfg),
                kademlia: kad::Behaviour::with_config(
                    key.public().to_peer_id(),
                    kad_store,
                    kad_config,
                ),
                identify: identify::Behaviour::new(identify::Config::new(
                    "/yamux/1.0.0".to_string(),
                    key.public(),
                )),
                rendezvous: rendezvous::server::Behaviour::new(
                    rendezvous::server::Config::default(),
                ),
                ping: ping::Behaviour::new(
                    ping::Config::new().with_interval(Duration::from_secs(60)),
                ),
                mdns: mdns::tokio::Behaviour::new(
                    mdns::Config::default(),
                    key.public().to_peer_id(),
                )?,
            })
        })?
        .build();

    for peer in &IPFS_BOOTNODES {
        swarm
            .behaviour_mut()
            .ipfs
            .add_address(&peer.parse()?, "/dnsaddr/bootstrap.libp2p.io".parse()?);
        swarm
            .behaviour_mut()
            .kademlia
            .add_address(&peer.parse()?, "/dnsaddr/bootstrap.libp2p.io".parse()?);
    }

    let bootstrap_node: Multiaddr = "/dnsaddr/bootstrap.libp2p.io"
        .parse()
        .expect("Hardcoded bootstrap address should be valid");
    for peer in &IPFS_BOOTNODES {
        let peer_id: PeerId = peer.parse()?;
        let addr: Multiaddr = "/dnsaddr/bootstrap.libp2p.io".parse()?;
        swarm
            .behaviour_mut()
            .ipfs
            .add_address(&peer_id, addr.clone());
        swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
    }

    swarm
        .behaviour_mut()
        .kademlia
        .set_mode(Some(kad::Mode::Server));

    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
    info!("Starting initial git repository scan and data publishing...");
    if let Err(e) = run(&args, &mut swarm).await {
        warn!("Error during initial git processing: {}", e);
    }
    debug!("Initial data publishing complete.");

    let topic = IdentTopic::new("bitcoin_alert_system");
    swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

    // --- Main Event Loop ---
    let mut stdin = io::BufReader::new(io::stdin()).lines();
    loop {
        select! {
            line = stdin.next_line() => {
                let line = line?.ok_or("stdin closed")?;
                handle_input_line(&mut swarm, line).await;
            }
            event = swarm.select_next_some() => {
                handle_swarm_event(&mut swarm, event).await;
            }
        }
    }
}

async fn handle_swarm_event(swarm: &mut Swarm<Behaviour>, event: SwarmEvent<BehaviourEvent>) {
    match event {
        SwarmEvent::NewListenAddr { address, .. } => {
            warn!("Listening on {address}");
        }

        // Mdns
        SwarmEvent::Behaviour(BehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
            for (peer_id, multiaddr) in list {
                info!("mDNS discovered a new peer: {peer_id}\n{multiaddr}");
                swarm
                    .behaviour_mut()
                    .kademlia
                    .add_address(&peer_id, multiaddr);
            }
        }
        // Kademlia
        SwarmEvent::Behaviour(BehaviourEvent::Kademlia(kad::Event::OutboundQueryProgressed {
            result,
            ..
        })) => match result {
            kad::QueryResult::GetRecord(Ok(kad::GetRecordOk::FoundRecord(kad::PeerRecord {
                record,
                ..
            }))) => {
                println!(
                    "{{\"key\":{:?},\"value\":{:?}}}",
                    std::str::from_utf8(record.key.as_ref()).unwrap_or("invalid utf8"),
                    std::str::from_utf8(&record.value).unwrap_or("invalid utf8"),
                );
            }
            kad::QueryResult::GetRecord(Err(err)) => {
                warn!("Failed to get record: {err:?}");
            }
            kad::QueryResult::PutRecord(Ok(kad::PutRecordOk { key })) => {
                debug!(
                    "Successfully PUT record for key: {:?}",
                    std::str::from_utf8(key.as_ref())
                );
            }
            kad::QueryResult::PutRecord(Err(err)) => {
                trace!("Failed to PUT record: {err:?}");
            }
            kad::QueryResult::StartProviding(Ok(kad::AddProviderOk { key, .. })) => {
                debug!(
                    "Successfully started PROVIDING key: {:?}",
                    std::str::from_utf8(key.as_ref())
                );
            }
            kad::QueryResult::StartProviding(Err(err)) => {
                warn!("Failed to start PROVIDING: {err:?}");
            }
            _ => {}
        },
        // Gossipsub
        SwarmEvent::Behaviour(BehaviourEvent::Gossipsub(event)) => {
            // This is where we handle all events from the Gossipsub behaviour
            match event {
                gossipsub::Event::Message {
                    propagation_source,
                    message_id,
                    message,
                } => {
                    let topic_str = message.topic.to_string();
                    let message_text = String::from_utf8_lossy(&message.data);
                    println!(
                        "Received message: '{}' on topic '{}' from peer: {:?}",
                        message_text, topic_str, propagation_source
                    );
                }
                gossipsub::Event::Subscribed { peer_id, topic } => {
                    warn!(
                        "542:Peer {:?} subscribed to topic '{}'",
                        peer_id,
                        topic.to_string()
                    );
                }
                gossipsub::Event::Unsubscribed { peer_id, topic } => {
                    warn!(
                        "549:Peer {:?} unsubscribed from topic '{}'",
                        peer_id,
                        topic.to_string()
                    );
                }
                gossipsub::Event::GossipsubNotSupported { peer_id } => {
                    debug!("Peer {:?} does not support Gossipsub", peer_id);
                } //gossipsub::Event::SlowPeer { peer_id, .. } => {
                  //    warn!("SlowPeer {:?}", peer_id);
                  //}
            }
        }

        _ => {}
    }
}

// async fn handle_swarm_event(swarm: &mut Swarm<Behaviour>, event: SwarmEvent<BehaviourEvent>) {
async fn handle_input_line(swarm: &mut Swarm<Behaviour>, line: String) {
    let mut args = line.split_whitespace();
    match args.next() {
        Some("TOPIC") => {
            if let Some(key_str) = args.next() {
                //let key = kad::RecordKey::new(&key_str);
                // swarm.behaviour_mut().kademlia.get_record(key.clone());

                let topic = IdentTopic::new(key_str);
                println!("583:subscribe topic={}", topic.clone());
                swarm
                    .behaviour_mut()
                    .gossipsub
                    .subscribe(&topic)
                    .expect("failed to subscribe to TOPIC");
            } else {
                eprintln!("Usage: TOPIC <topic_string>");
            }
        }
        Some("GET") => {
            if let Some(key_str) = args.next() {
                let key = kad::RecordKey::new(&key_str);
                swarm.behaviour_mut().kademlia.get_record(key);
            } else {
                eprintln!("Usage: GET <key>");
            }
        }
        Some("GET_PROVIDERS") => {
            if let Some(key_str) = args.next() {
                let key = kad::RecordKey::new(&key_str);
                swarm.behaviour_mut().kademlia.get_providers(key);
            } else {
                eprintln!("Usage: GET_PROVIDERS <key>");
            }
        }
        Some("PUT") => {
            if let (Some(key_str), Some(value_str)) = (args.next(), args.next()) {
                let key = kad::RecordKey::new(&key_str);
                let value = value_str.as_bytes().to_vec();
                let record = kad::Record {
                    key: key.clone(),
                    value,
                    publisher: None,
                    expires: None,
                };
                if let Err(e) = swarm
                    .behaviour_mut()
                    .kademlia
                    .put_record(record.clone(), kad::Quorum::Majority)
                {
                    debug!("Failed to store record locally: {:?}", e);
                } else {
                    info!(
                        "put record.key:{:?} record.value:{:?}",
                        record.key, record.value
                    );
                }
                if let Err(e) = swarm.behaviour_mut().kademlia.start_providing(key.clone()) {
                    debug!("Failed to store record locally: {:?}", e);
                } else {
                    info!(
                        "started providing put record.key:{:?} record.value:{:?} key:{:?}",
                        record.key.clone(),
                        record.value,
                        key.clone()
                    );

                    let topic = IdentTopic::new(format!(
                        "{}",
                        std::str::from_utf8(record.key.as_ref()).unwrap_or("invalid utf8"),
                    ));

                    println!("652:subscribe topic={}", topic.clone());
                    swarm
                        .behaviour_mut()
                        .gossipsub
                        .subscribe(&topic)
                        .expect("failed to subscribe to TOPIC");
                    //} else {
                    eprintln!("Usage: PUT <key> <value>");
                }
            }
        }
        Some("PUT_PROVIDER") => {
            let key = {
                match args.next() {
                    Some(key) => kad::RecordKey::new(&key),
                    None => {
                        eprint!("gnostr> ");
                        return;
                    }
                }
            };
            if let Err(e) = swarm.behaviour_mut().kademlia.start_providing(key) {
                eprintln!("Failed to store record locally: {:?}", e);
            }
        }

        Some("QUIT") | Some("Q") | Some("EXIT") => {
            std::process::exit(0);
        }
        _ => {
            eprintln!("Commands: GET, GET_PROVIDERS, PUT, QUIT");
        }
    }
}

async fn run(args: &Args, swarm: &mut Swarm<Behaviour>) -> Result<(), Box<dyn Error>> {
    let path = args.flag_git_dir.as_ref().map_or(".", |s| &s[..]);
    let repo = Repository::discover(path)?;
    if let Ok(tag_names) = repo.tag_names(None) {
        for tag_name_opt in tag_names.iter() {
            if let Some(tag_name) = tag_name_opt {
                if let Ok(commit_id) = get_commit_id_of_tag(&repo, tag_name) {
                    let key = kad::RecordKey::new(&tag_name);
                    let record = kad::Record {
                        key: key.clone(),
                        value: commit_id.into_bytes(),
                        publisher: Some(swarm.local_peer_id().clone()),
                        expires: None,
                    };
                    swarm
                        .behaviour_mut()
                        .kademlia
                        .put_record(record, kad::Quorum::Majority)?;
                    swarm
                        .behaviour_mut()
                        .kademlia
                        .start_providing(key.clone())?;

                    let topic = IdentTopic::new(tag_name);
                    debug!("676:subscribe topic={}", topic.clone());
                    swarm.behaviour_mut().gossipsub.subscribe(&topic)?;
                }
            }
        }
    }
    let mut revwalk = repo.revwalk()?;
    let base = if args.flag_reverse {
        git2::Sort::REVERSE
    } else {
        git2::Sort::NONE
    };
    let sorting = base
        | if args.flag_topo_order {
            git2::Sort::TOPOLOGICAL
        } else if args.flag_date_order {
            git2::Sort::TIME
        } else {
            git2::Sort::NONE
        };
    revwalk.set_sorting(sorting)?;

    if args.arg_commit.is_empty() {
        revwalk.push_head()?;
    } else {
        for commit_spec in &args.arg_commit {
            let obj = repo.revparse_single(commit_spec)?;
            revwalk.push(obj.id())?;
        }
    }

    let revwalk_iterator = revwalk
        .filter_map(Result::ok)
        .filter_map(|id| repo.find_commit(id).ok());

    for commit in revwalk_iterator.take(args.flag_max_count.unwrap_or(usize::MAX)) {
        let commit_id_str = commit.id().to_string();
        let msg_key = kad::RecordKey::new(&commit_id_str);
        let msg_record = kad::Record {
            key: msg_key.clone(),
            value: commit.message_bytes().to_vec(),
            publisher: Some(*swarm.local_peer_id()),
            expires: None,
        };
        swarm
            .behaviour_mut()
            .kademlia
            .put_record(msg_record, kad::Quorum::Majority)?;
        swarm.behaviour_mut().kademlia.start_providing(msg_key)?;
        if let Ok(diff_bytes) = get_commit_diff_as_bytes(&repo, &commit) {
            let diff_key_str = format!("{}/diff", commit_id_str);
            let diff_key = kad::RecordKey::new(&diff_key_str);
            let diff_record = kad::Record {
                key: diff_key.clone(),
                value: diff_bytes,
                publisher: Some(*swarm.local_peer_id()),
                expires: None,
            };
            swarm
                .behaviour_mut()
                .kademlia
                .put_record(diff_record, kad::Quorum::One)?;
            swarm.behaviour_mut().kademlia.start_providing(diff_key)?;
        }
    }

    Ok(())
}

// --- CLI Arguments Struct (unchanged from original) ---
#[derive(Debug, Parser)]
struct Args {
    #[clap(long)]
    secret: Option<u8>,

    // peer implies lookup by dht default --network ipfs
    #[clap(long)]
    peer: Option<String>,

    // multiaddr implies direct connect
    #[clap(long)]
    multiaddr: Option<Multiaddr>,

    // network
    #[clap(long, value_enum, default_value = &"ipfs")]
    network: Option<Network>,

    #[clap(long)]
    flag_topo_order: bool,
    #[clap(long)]
    flag_date_order: bool,
    #[clap(long)]
    flag_reverse: bool,
    #[clap(long)]
    flag_author: Option<String>,
    #[clap(long)]
    flag_committer: Option<String>,
    #[clap(long = "grep")]
    flag_grep: Option<String>,
    #[clap(long = "git-dir")]
    flag_git_dir: Option<String>,
    #[clap(long)]
    flag_skip: Option<usize>,
    #[clap(short = 'n', long)]
    flag_max_count: Option<usize>,
    #[clap(long)]
    flag_merges: bool,
    #[clap(long)]
    flag_no_merges: bool,
    #[clap(long)]
    flag_no_min_parents: bool,
    #[clap(long)]
    flag_no_max_parents: bool,
    #[clap(long)]
    flag_max_parents: Option<usize>,
    #[clap(long)]
    flag_min_parents: Option<usize>,
    #[clap(long, short)]
    flag_patch: bool,
    arg_commit: Vec<String>,
    #[clap(last = true)]
    arg_spec: Vec<String>,
}
