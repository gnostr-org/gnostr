#![doc = include_str!("../../README.md")]
use std::error::Error;

use clap::Parser;
use futures::stream::StreamExt;
use libp2p::{gossipsub::IdentTopic, identity, kad, Multiaddr, PeerId};
use tokio::{
    io::{self, AsyncBufReadExt},
    select,
};
use tracing::{debug, info, warn};

use gnostr::p2p::args::Args;
use gnostr::p2p::command_handler::handle_input_line;
use gnostr::p2p::event_handler::handle_swarm_event;
use gnostr::p2p::git_publisher::run_git_publisher;
use gnostr::p2p::network_config::{IPFS_BOOTNODES};
use gnostr::p2p::swarm_builder;
use gnostr::p2p::utils::{generate_ed25519, init_subscriber};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _ = init_subscriber();
    let args = Args::parse();
    warn!("args={:?}", args);

    if let Some(ref _peer) = args.peer {}
    if let Some(ref _multiaddr) = args.multiaddr {}
    if let Some(ref _network) = args.network {}
    if let Some(ref _secret) = args.secret {}

    if let Some(true) = Some(args.peer.is_some()) {}
    if let Some(true) = Some(args.multiaddr.is_some()) {}
    if let Some(true) = Some(args.network.is_some()) {}
    if let Some(true) = Some(args.secret.is_some()) {}

    let keypair: identity::Keypair = generate_ed25519(args.secret);
    let public_key = keypair.public();
    let peer_id = PeerId::from_public_key(&public_key);
    warn!("Local PeerId: {}", peer_id);

    let mut swarm = swarm_builder::build_swarm(keypair)?;

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

    let _bootstrap_node: Multiaddr = "/dnsaddr/bootstrap.libp2p.io"
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
        warn!("Error during initial git processing: {}", e);
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