//! Core peer-to-peer primitives used by `gnostr`.
//!
//! This module owns the shared swarm setup, message routing, peer identity
//! helpers, and the chat/P2P glue that higher-level commands build on.

pub use gnostr_p2p::{
    args, behaviour, command_handler, event_handler, git_integration, git_publisher, kvs, lookup,
    network_config, opt, swarm_builder, utils,
};

use std::{
    env,
    error::Error,
    hash::{DefaultHasher, Hash, Hasher},
    time::Duration,
};

use chrono::{Local, Timelike};
use futures::stream::StreamExt;
use libp2p::{
    autonat, dcutr, gossipsub, identify, identity,
    kad::{
        self,
        store::{MemoryStore, MemoryStoreConfig},
        Config as KadConfig, Mode, Quorum, Record, RecordKey,
    },
    mdns, noise, ping, relay, rendezvous,
    swarm::SwarmEvent,
    tcp, yamux, Multiaddr, PeerId,
};
use serde_json;
use tokio::{io, select};
use tracing::{debug, info, trace, warn};
use ureq::Agent;

use crate::{
    blockhash_async, blockheight_async,
    p2p::network_config::Network,
    types::Event,
};
use gnostr_chat::{
    msg::{Msg, MsgKind},
    ChatSubCommands,
};

//const TOPIC: &str = "gnostr";

/// Fetch a mempool API response asynchronously and return the body.
///
/// This is used by chat-driven and UI-driven flows that need a short-lived
/// external prompt without blocking the swarm event loop.
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

/// Derive a deterministic peer ID from a 32-byte seed.
///
/// Chat uses this for local test peers so multiple instances can attach to the
/// same topic while still behaving like distinct identities.
pub fn generate_close_peer_id(bytes: [u8; 32], _common_bits: usize) -> PeerId {
    let mut close_bytes;
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

    close_bytes[31] = bytes[31];

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

/// Run the shared P2P event loop for chat-style message exchange.
///
/// The loop subscribes to the topic, listens on TCP and QUIC, and forwards
/// incoming events back into the caller's message channel.
pub async fn evt_loop(
    args: ChatSubCommands,
    mut send: tokio::sync::mpsc::Receiver<Msg>,
    recv: tokio::sync::mpsc::Sender<Msg>,
    topic: gossipsub::IdentTopic,
) -> Result<(), Box<dyn Error>> {
    let keypair: identity::Keypair =
        crate::p2p::utils::generate_ed25519(args.nsec.clone().map(|s| s.as_bytes()[0]));
    let public_key = keypair.public();
    let peer_id = PeerId::from_public_key(&public_key);
    warn!("Local PeerId: {}", peer_id);

    let kad_store_config = MemoryStoreConfig {
        max_provided_keys: usize::MAX,
        max_providers_per_key: usize::MAX,
        max_records: usize::MAX,
        max_value_bytes: usize::MAX,
    };
    let _kad_memstore = MemoryStore::with_config(peer_id, kad_store_config.clone());
    let _kad_config = KadConfig::new(crate::p2p::network_config::IPFS_PROTO_NAME);
    let _message_id_fn = |message: &gossipsub::Message| {
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
    #[allow(clippy::redundant_closure)]
    let gossipsub_config = gossipsub::ConfigBuilder::default()
        .heartbeat_interval(Duration::from_secs(1))
        .validation_mode(gossipsub::ValidationMode::Permissive)
        .build()
        .map_err(|msg| io::Error::other(msg))?;

    let mut swarm = libp2p::SwarmBuilder::with_existing_identity(keypair)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_quic()
        .with_dns()?
        .with_relay_client(noise::Config::new, yamux::Config::default)?
        .with_behaviour(|key, relay_client| {
            let local_peer_id = key.public().to_peer_id();
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
            let kad_store = MemoryStore::with_config(local_peer_id, kad_store_config);
            let mut ipfs_cfg = KadConfig::new(crate::p2p::network_config::IPFS_PROTO_NAME);
            ipfs_cfg.set_query_timeout(Duration::from_secs(5 * 60));
            let ipfs_store = MemoryStore::new(local_peer_id);
            let relay_server = relay::Behaviour::new(local_peer_id, Default::default());
            let rendezvous_client = rendezvous::client::Behaviour::new(key.clone());
            let rendezvous_server =
                rendezvous::server::Behaviour::new(rendezvous::server::Config::default());
            Ok(crate::p2p::behaviour::Behaviour {
                relay_client,
                relay_server,
                autonat: autonat::Behaviour::new(local_peer_id, autonat::Config::default()),
                dcutr: dcutr::Behaviour::new(local_peer_id),
                gossipsub: gossipsub::Behaviour::new(
                    gossipsub::MessageAuthenticity::Signed(key.clone()),
                    gossipsub_config,
                )
                .expect(""),
                ipfs: kad::Behaviour::with_config(local_peer_id, ipfs_store, ipfs_cfg),
                kademlia: kad::Behaviour::with_config(local_peer_id, kad_store, kad_config),
                identify: identify::Behaviour::new(identify::Config::new(
                    "/yamux/1.0.0".to_string(),
                    key.public(),
                )),
                rendezvous_client,
                rendezvous: rendezvous_server,
                ping: ping::Behaviour::new(
                    ping::Config::new().with_interval(Duration::from_secs(60)),
                ),
                mdns: mdns::tokio::Behaviour::new(mdns::Config::default(), local_peer_id)?,
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

            if !current_second.is_multiple_of(2) {
                debug!("Current second ({}) is odd!", current_second);
                unsafe { env::set_var("BLOCKHEIGHT", &blockheight_async().await) };
                unsafe { env::set_var("BLOCKHASH", &blockhash_async().await) };
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
        tokio::time::sleep(Duration::from_millis(250)).await;

        select! {
            Some(m) = send.recv() => {
                if let Err(e) = swarm
                    .behaviour_mut().gossipsub
                    .publish(topic.clone(), serde_json::to_vec(&m)?)
                 {
                    debug!("Publish error: {e:?}");
                    //let mut m = Msg::default()
                    //    /**/.set_content(format!("{{\"blockheight\":\"{}\"}}", env::var("BLOCKHEIGHT").unwrap()), 0).set_kind(MsgKind::System);
                    ////NOTE:recv.send - send to self
                    //recv.send(m).await?;
                    //m = Msg::default()
                    //    /**/.set_content(format!("{{\"blockhash\":\"{}\"}}", env::var("BLOCKHASH").unwrap()), 0).set_kind(MsgKind::System);
                    ////NOTE:recv.send - send to self
                    //recv.send(m).await?;
                }
            }
            event = swarm.select_next_some() => match event {
                SwarmEvent::Behaviour(crate::p2p::behaviour::BehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                    for (peer_id, multiaddr) in list {
                        debug!("mDNS discovered a new peer: {peer_id}");
                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                        swarm.behaviour_mut().autonat.add_server(peer_id, Some(multiaddr.clone()));
                    }
                },
                SwarmEvent::Behaviour(crate::p2p::behaviour::BehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                    for (peer_id, _multiaddr) in list {
                        debug!("mDNS discover peer has expired: {peer_id}");
                        swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                        swarm.behaviour_mut().autonat.remove_server(&peer_id);
                    }
                },
                SwarmEvent::Behaviour(crate::p2p::behaviour::BehaviourEvent::Autonat(event)) => {
                    debug!("AutoNAT event: {event:?}");
                }
                SwarmEvent::Behaviour(crate::p2p::behaviour::BehaviourEvent::Dcutr(event)) => {
                    debug!("DCUtR event: {event:?}");
                }
                SwarmEvent::Behaviour(crate::p2p::behaviour::BehaviourEvent::RelayClient(event)) => {
                    debug!("Relay client event: {event:?}");
                }
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
                        Ok(mut msg) => {
                            if msg.kind == MsgKind::NostrEvent {
                                match serde_json::from_str::<Event>(&msg.content[0]) {
                                    Ok(event) => {
                                        debug!("Deserialized Nostr Event: {:?}", event);
                                        msg.nostr_event = Some(event.clone()); // Store the deserialized event
                                        // For now, let's just re-serialize it back into content for display
                                        msg = msg.set_content(format!("Nostr Event: {}", event.id.as_hex_string()), 0);
                                    },
                                    Err(e) => {
                                        warn!("Error deserializing Nostr event from message content: {e:?}");
                                        msg = msg.set_content(format!("Error processing Nostr Event: {e:?}"), 0);
                                        msg.kind = MsgKind::System;
                                    }
                                }
                            }
                            recv.send(msg).await?;
                        },
                        Err(e) => {
                            warn!("Error deserializing message: {e:?}");
                            let m = Msg::default().set_content(format!("Error deserializing message: {e:?}"), 0).set_kind(MsgKind::System);
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

fn service_announcement_record(service_name: &str, service_url: &str, peer_id: PeerId) -> Record {
    let key = format!("gnostr/services/{service_name}");
    let value = serde_json::json!({
        "service": service_name,
        "base_url": service_url,
        "peer_id": peer_id.to_string(),
    })
    .to_string()
    .into_bytes();

    Record {
        key: RecordKey::new(&key),
        value,
        publisher: Some(peer_id),
        expires: None,
    }
}

pub async fn advertise_service(
    service_name: String,
    service_url: String,
) -> Result<(), Box<dyn Error>> {
    let keypair = identity::Keypair::generate_ed25519();
    let mut swarm = crate::p2p::swarm_builder::build_swarm(keypair).await?;
    let peer_id = *swarm.local_peer_id();

    let bootstrap_addr: Multiaddr = "/dnsaddr/bootstrap.libp2p.io".parse()?;
    for (addr, boot_peer) in Network::Ipfs.bootnodes() {
        swarm
            .behaviour_mut()
            .ipfs
            .add_address(&boot_peer, addr.clone());
        swarm.behaviour_mut().kademlia.add_address(&boot_peer, addr);
    }
    for peer in crate::p2p::network_config::IPFS_BOOTNODES {
        let peer_id: PeerId = peer.parse()?;
        swarm
            .behaviour_mut()
            .ipfs
            .add_address(&peer_id, bootstrap_addr.clone());
        swarm
            .behaviour_mut()
            .kademlia
            .add_address(&peer_id, bootstrap_addr.clone());
    }

    swarm.behaviour_mut().kademlia.set_mode(Some(Mode::Server));
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
    swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;

    let record_key_name = format!("gnostr/services/{service_name}");
    let record_key = RecordKey::new(&record_key_name);
    let mut publish_interval = tokio::time::interval(std::time::Duration::from_secs(15 * 60));

    let publish = |swarm: &mut libp2p::Swarm<crate::p2p::behaviour::Behaviour>| {
        let record = service_announcement_record(&service_name, &service_url, peer_id);
        swarm
            .behaviour_mut()
            .kademlia
            .put_record(record, Quorum::Majority)?;
        swarm
            .behaviour_mut()
            .kademlia
            .start_providing(record_key.clone())?;
        Ok::<_, Box<dyn Error>>(())
    };

    publish(&mut swarm)?;
    info!("Advertising {service_name} at {service_url} as {peer_id}");

    loop {
        select! {
            _ = publish_interval.tick() => {
                publish(&mut swarm)?;
                info!("Refreshed {service_name} advertisement for {service_url}");
            }
            event = swarm.select_next_some() => {
                match event {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        debug!("advertiser listening on {address}");
                    }
                    SwarmEvent::Behaviour(crate::p2p::behaviour::BehaviourEvent::Kademlia(kad::Event::OutboundQueryProgressed { result, .. })) => {
                        debug!("advertiser kademlia event: {result:?}");
                    }
                    _ => {}
                }
            }
        }
    }
}
