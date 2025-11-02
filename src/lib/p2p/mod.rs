pub mod command_handler;
pub mod kvs;
pub mod opt;
pub mod chat;
pub mod network_config;
pub mod behaviour;
pub mod utils;
pub mod git_integration;
pub mod git_publisher;

use crate::blockhash::blockhash_async;
use crate::blockheight::blockheight_async;
use crate::p2p::chat::msg::{Msg, MsgKind};
use crate::p2p::chat::ChatSubCommands;
use chrono::{Local, Timelike};
use futures::stream::StreamExt;
use libp2p::{
    gossipsub,
    identify, identity,
    kad::{
        self,
        store::{MemoryStore, MemoryStoreConfig},
        Config as KadConfig,
    },
    mdns, noise, ping, rendezvous,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId,
};
use std::{
    env,
    error::Error,
    hash::{DefaultHasher, Hash, Hasher},
    str,
    str::FromStr,
    thread,
};
use tokio::time::Duration;
use tokio::{io, select};
use tracing::{debug, info, trace, warn};
use ureq::Agent;
use serde_json;

//const TOPIC: &str = "gnostr";

/// async_prompt
pub async fn async_prompt(mempool_url: String) -> String {
    let s = tokio::spawn(async move {
        let agent: Agent = ureq::AgentBuilder::new()
            .timeout_read(Duration::from_secs(10))
            .timeout_write(Duration::from_secs(10))
            .build();
        let body: String = agent
            .get(&mempool_url)
            .call()
            .expect("")
            .into_string()
            .expect("mempool_url:body:into_string:fail!");

        body
    });

    s.await.unwrap()
}

pub fn generate_close_peer_id(bytes: [u8; 32], common_bits: usize) -> PeerId {
    let mut close_bytes = [0u8; 32];
    close_bytes = bytes;

    for (i, byte) in close_bytes.iter().enumerate() {
        if i < 32 {
            trace!("Byte i={:02} [{:3} / {:#04x}]: ", i, byte, byte);
            for j in (0..8).rev() {
                let mask = 1 << j;
                if byte & mask == 0 {
                    trace!("0");
                } else {
                    trace!("1");
                }
            }
            trace!("\n");
        }
    }
    let mut keypair = 
        identity::Keypair::ed25519_from_bytes(close_bytes).expect("only errors on wrong length");
    trace!("262:{}", keypair.public().to_peer_id());

    close_bytes[31] = bytes[31] ^ 0u8;

    for (i, byte) in close_bytes.iter().enumerate() {
        trace!("265:Byte {:02} [{:3} / {:#04x}]: ", i, byte, byte);
        for j in (0..8).rev() {
            let mask = 1 << j;
            if byte & mask == 0 {
                trace!("0");
            } else {
                trace!("1");
            }
        }
        trace!("");
    }

    keypair = 
        identity::Keypair::ed25519_from_bytes(close_bytes).expect("only errors on wrong length");
    trace!("292:{}", keypair.public().to_peer_id());
    keypair.public().to_peer_id()
}

/// evt_loop
pub async fn evt_loop(
    args: ChatSubCommands,
    mut send: tokio::sync::mpsc::Receiver<Msg>,
    recv: tokio::sync::mpsc::Sender<Msg>,
    topic: gossipsub::IdentTopic,
) -> Result<(), Box<dyn Error>> {
    let keypair: identity::Keypair = crate::p2p::utils::generate_ed25519(args.nsec.clone().map(|s| s.as_bytes()[0]));
    let public_key = keypair.public();
    let peer_id = PeerId::from_public_key(&public_key);
    warn!("Local PeerId: {}", peer_id);

    let kad_store_config = MemoryStoreConfig {
        max_provided_keys: usize::MAX,
        max_providers_per_key: usize::MAX,
        max_records: usize::MAX,
        max_value_bytes: usize::MAX,
    };
    let _kad_memstore = MemoryStore::with_config(peer_id.clone(), kad_store_config.clone());
	let _kad_config = KadConfig::new(crate::p2p::network_config::IPFS_PROTO_NAME);
    let message_id_fn = |message: &gossipsub::Message| {
        let mut s = DefaultHasher::new();
        message.data.hash(&mut s);
        info!("message:\n{0:?}", message);
        info!("message.data:\n{0:?}", message.data);
        info!("message.source:\n{0:?}", message.source);
        info!("message.source:\n{0:1?}", message.source);
        info!("message.source.peer_id:\n{0:2?}", message.source.unwrap());
        info!(
            "message.source.peer_id:\n{0:3}",
            message.source.unwrap().to_string()
        );
        info!("message.sequence_number:\n{0:?}", message.sequence_number);
        info!("message.topic:\n{0:?}", message.topic);
        info!("message.topic.hash:\n{0:0}", message.topic.clone());
        gossipsub::MessageId::from(s.finish().to_string())
    };
    let gossipsub_config = gossipsub::ConfigBuilder::default()
        .heartbeat_interval(Duration::from_secs(1))
        .validation_mode(gossipsub::ValidationMode::Permissive)
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
            let mut kad_config = KadConfig::new(crate::p2p::network_config::IPFS_PROTO_NAME);
            kad_config.set_query_timeout(Duration::from_secs(120));
            kad_config.set_replication_factor(std::num::NonZeroUsize::new(20).unwrap());
            kad_config.set_publication_interval(Some(Duration::from_secs(10)));
            kad_config.disjoint_query_paths(false);
            let kad_store = MemoryStore::with_config(peer_id.clone(), kad_store_config);
            let mut ipfs_cfg = KadConfig::new(crate::p2p::network_config::IPFS_PROTO_NAME);
            ipfs_cfg.set_query_timeout(Duration::from_secs(5 * 60));
            let ipfs_store = MemoryStore::new(key.public().to_peer_id());
            Ok(crate::p2p::behaviour::Behaviour {
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

    // subscribes to our topic
    swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

    // Listen on all interfaces and whatever port the OS assigns
    swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    debug!("Enter messages via STDIN and they will be sent to connected peers using Gossipsub");

    // Kick it off
    // Kick it off
    loop {
        debug!("p2p.rs:begin loop");

        // Check if the current second is odd
        let handle = tokio::spawn(async {
            let now = Local::now();

            // Get the current second
            let current_second = now.second();

            if current_second % 2 != 0 {
                debug!("Current second ({}) is odd!", current_second);
                env::set_var("BLOCKHEIGHT", &blockheight_async().await);
                env::set_var("BLOCKHASH", &blockhash_async().await);
            } else {
                debug!(
                    "Current second ({}) is even. Skipping this iteration.",
                    current_second
                );
            }
        });

        debug!("Still running other things while the task is awaited...");

        handle.await.unwrap_or(()); // Wait for the async task to complete
        debug!("All done!");

        // Wait for a second before checking again to avoid rapid looping
        thread::sleep(Duration::from_millis(250));

        select! {
            Some(m) = send.recv() => {
                if let Err(e) = swarm
                    .behaviour_mut().gossipsub
                    .publish(topic.clone(), serde_json::to_vec(&m)?)
                 {
                    debug!("Publish error: {e:?}");
                    let mut m = Msg::default()
                        /**/.set_content(format!("{{\"blockheight\":\"{}\"}}", env::var("BLOCKHEIGHT").unwrap()), 0).set_kind(MsgKind::System);
                    //NOTE:recv.send - send to self
                    recv.send(m).await?;
                    m = Msg::default()
                        /**/.set_content(format!("{{\"blockhash\":\"{}\"}}", env::var("BLOCKHASH").unwrap()), 0).set_kind(MsgKind::System);
                    //NOTE:recv.send - send to self
                    recv.send(m).await?;
                }
            }
            event = swarm.select_next_some() => match event {
                SwarmEvent::Behaviour(crate::p2p::behaviour::BehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                    for (peer_id, _multiaddr) in list {
                        debug!("mDNS discovered a new peer: {peer_id}");
                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                    }
                },
                SwarmEvent::Behaviour(crate::p2p::behaviour::BehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                    for (peer_id, _multiaddr) in list {
                        debug!("mDNS discover peer has expired: {peer_id}");
                        swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                    }
                },
                SwarmEvent::Behaviour(crate::p2p::behaviour::BehaviourEvent::Gossipsub(gossipsub::Event::Message { 
                    propagation_source: peer_id,
                    message_id: id,
                    message,
                })) => {
                    debug!(
                        "Got message: '{}' with id: {id} from peer: {peer_id}",
                        String::from_utf8_lossy(&message.data),
                    );
                    match serde_json::from_slice::<Msg>(&message.data) {
                        Ok(msg) => {
                            recv.send(msg).await?;
                        },
                        Err(e) => {
                            warn!("Error deserializing message: {e:?}");
                            let m = Msg::default().set_content(format!("Error deserializing message: {e:?}"), 0_usize).set_kind(MsgKind::System);
                            recv.send(m).await?;
                        }
                    }
                },
                SwarmEvent::NewListenAddr { address, .. } => {
                    debug!("Local node is listening on {address}");
                }
                _ => {} 
            }
        }
        debug!("p2p.rs:end loop");
    }
}