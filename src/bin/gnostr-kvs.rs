#![doc = include_str!("../../README.md")]
use std::{
    error::Error,
    hash::{DefaultHasher, Hash, Hasher},
    str,
    str::FromStr,
    time::Duration,
};

use clap::Parser;
use futures::stream::StreamExt;
use libp2p::{gossipsub::IdentTopic, identity, swarm::SwarmEvent, Multiaddr, PeerId};
use tokio::{
    io::{self, AsyncBufReadExt},
    select,
};
use tracing::{debug, info, trace, warn};

use gnostr::p2p::command_handler::handle_input_line;
use gnostr::p2p::event_handler::handle_swarm_event;
use gnostr::p2p::git_publisher::run_git_publisher;
use gnostr::p2p::network_config::{Network, IPFS_BOOTNODES};
use gnostr::p2p::swarm_builder;
use gnostr::p2p::utils::{generate_ed25519, init_subscriber};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _ = init_subscriber();
    let args = Args::parse();
    warn!("args={{:?}}", args);

    if let Some(ref peer) = args.peer {}
    if let Some(ref multiaddr) = args.multiaddr {}
    if let Some(ref network) = args.network {}
    if let Some(ref secret) = args.secret {}

    if let Some(true) = Some(args.peer.is_some()) {}
    if let Some(true) = Some(args.multiaddr.is_some()) {}
    if let Some(true) = Some(args.network.is_some()) {}
    if let Some(true) = Some(args.secret.is_some()) {}

    let keypair: identity::Keypair = generate_ed25519(args.secret.clone());
    let public_key = keypair.public();
    let peer_id = PeerId::from_public_key(&public_key);
    warn!("Local PeerId: {{}}", peer_id);

    let kad_store_config = MemoryStoreConfig {
        max_provided_keys: usize::MAX,
        max_providers_per_key: usize::MAX,
        max_records: usize::MAX,
        max_value_bytes: usize::MAX,
    };
    let _kad_memstore = MemoryStore::with_config(peer_id.clone(), kad_store_config.clone());
    let _kad_config = KadConfig::default();
    let message_id_fn = |message: &gossipsub::Message| {
        let mut s = DefaultHasher::new();
        message.data.hash(&mut s);
        info!("message:\n{{0:?}}", message);
        info!("message.data:\n{{0:?}}", message.data);
        info!("message.source:\n{{0:?}}", message.source);
        info!("message.source:\n{{0:1?}}", message.source);
        info!("message.source.peer_id:\n{{0:2?}}", message.source.unwrap());
        info!(
            "message.source.peer_id:\n{{0:3}}",
            message.source.unwrap().to_string()
        );
        info!("message.sequence_number:\n{{0:?}}", message.sequence_number);
        info!("message.topic:\n{{0:?}}", message.topic);
        info!("message.topic.hash:\n{{0:0}}", message.topic.clone());
        gossipsub::MessageId::from(s.finish().to_string())
    };
    let gossipsub_config = gossipsub::ConfigBuilder::default()
        .heartbeat_interval(Duration::from_secs(1))
        .validation_mode(gossipsub::ValidationMode::Permissive)
        .message_id_fn(message_id_fn)
        .build()
        .map_err(|msg| io::Error::new(io::ErrorKind::Other, msg)?);

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
            let mut kad_config = KadConfig::default();
            kad_config.set_query_timeout(Duration::from_secs(120));
            kad_config.set_replication_factor(std::num::NonZeroUsize::new(20).unwrap());
            kad_config.set_publication_interval(Some(Duration::from_secs(10)));
            kad_config.disjoint_query_paths(false);
            let kad_store = MemoryStore::with_config(peer_id.clone(), kad_store_config);
            let mut ipfs_cfg = KadConfig::new(IPFS_PROTO_NAME);
            ipfs_cfg.set_query_timeout(Duration::from_secs(5 * 60));
            let ipfs_store = MemoryStore::new(key.public().to_peer_id());
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
        })? // This '?' is for the Result returned by the closure
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

    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)
        .expect("Failed to listen on address");
    info!("Starting initial git repository scan and data publishing...");
    if let Err(e) = run_git_publisher(&args, &mut swarm).await {
        warn!("Error during initial git processing: {{}}", e);
    }
    debug!("Initial data publishing complete.");

    let topic = IdentTopic::new("bitcoin_alert_system");
    swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

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

#[derive(Debug, Parser)]
struct Args {
    #[clap(long)]
    secret: Option<u8>,

    #[clap(long)]
    peer: Option<String>,

    #[clap(long)]
    multiaddr: Option<Multiaddr>,

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