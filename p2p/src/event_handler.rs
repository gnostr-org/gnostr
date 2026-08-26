use libp2p::{
    autonat, gossipsub, identify, kad, mdns, multiaddr::Protocol, relay, rendezvous,
    swarm::SwarmEvent, Multiaddr,
};
use tracing::{debug, info, trace, warn};

use crate::utils::multiaddr_with_peer_id;

use super::behaviour::BehaviourEvent;

pub async fn handle_swarm_event(
    swarm: &mut libp2p::Swarm<super::behaviour::Behaviour>,
    event: SwarmEvent<BehaviourEvent>,
) {
    match event {
        SwarmEvent::NewListenAddr { address, .. } => {
            crate::embedded_network::log_line(format!("listening on {address}"));
            warn!("Listening on {address}");
        }
        SwarmEvent::Behaviour(BehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
            for (peer_id, multiaddr) in list {
                crate::embedded_network::log_line(format!("mDNS discovered peer {peer_id} at {multiaddr}"));
                info!("mDNS discovered a new peer: {peer_id}\n{multiaddr}");
                let peer_id_label = peer_id.to_string();
                let address_with_p2p = multiaddr_with_peer_id(&multiaddr, &peer_id);
                crate::embedded_network::record_peer_discovery(
                    peer_id.to_string(),
                    "mdns",
                    vec![address_with_p2p.to_string()],
                );
                swarm
                    .behaviour_mut()
                    .kademlia
                    .add_address(&peer_id, address_with_p2p.clone());
                swarm
                    .behaviour_mut()
                    .autonat
                    .add_server(peer_id, Some(address_with_p2p.clone()));
                if let Err(error) = swarm.dial(address_with_p2p) {
                    crate::embedded_network::log_line(format!("failed to dial discovered peer {peer_id_label}: {error}"));
                    warn!("Failed to dial discovered peer {peer_id_label}: {error}");
                }
            }
        }
        SwarmEvent::Behaviour(BehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
            for (peer_id, _multiaddr) in list {
                crate::embedded_network::log_line(format!("mDNS peer expired: {peer_id}"));
                info!("mDNS peer expired: {peer_id}");
                swarm.behaviour_mut().autonat.remove_server(&peer_id);
            }
        }
        SwarmEvent::Behaviour(BehaviourEvent::Identify(identify::Event::Received {
            peer_id,
            info: identify::Info {
                observed_addr,
                ..
            },
            ..
        })) => {
            crate::embedded_network::log_line(format!("observed external address from {peer_id}: {observed_addr}"));
            info!("Observed external address from {peer_id}: {observed_addr}");
            swarm.add_external_address(observed_addr.clone());
        }
        SwarmEvent::Behaviour(BehaviourEvent::Autonat(event)) => match event {
            autonat::Event::StatusChanged { old, new } => {
                crate::embedded_network::log_line(format!("autonat status changed: {old:?} -> {new:?}"));
                info!("AutoNAT status changed: {old:?} -> {new:?}");
            }
            other => {
                debug!("AutoNAT event: {other:?}");
            }
        },
        SwarmEvent::Behaviour(BehaviourEvent::Ping(event)) => match event.result {
            Ok(rtt) => {
                crate::embedded_network::log_line(format!(
                    "ping peer={} rtt_ms={}",
                    event.peer,
                    rtt.as_millis()
                ));
                info!("Ping peer={} rtt={rtt:?}", event.peer);
            }
            Err(error) => {
                crate::embedded_network::log_line(format!(
                    "ping failed peer={} error={error}",
                    event.peer
                ));
                warn!("Ping failed peer={} error={error}", event.peer);
            }
        },
        SwarmEvent::Behaviour(BehaviourEvent::Dcutr(event)) => {
            debug!("DCUtR event: {event:?}");
        }
        SwarmEvent::Behaviour(BehaviourEvent::RelayClient(event)) => match event {
            relay::client::Event::ReservationReqAccepted { relay_peer_id, .. } => {
                crate::embedded_network::log_line(format!("relay reservation accepted by {relay_peer_id}"));
                info!("Relay reservation accepted by {relay_peer_id}");
            }
            relay::client::Event::OutboundCircuitEstablished { relay_peer_id, .. } => {
                crate::embedded_network::log_line(format!("relay circuit established via {relay_peer_id}"));
                info!("Relay circuit established via {relay_peer_id}");
            }
            relay::client::Event::InboundCircuitEstablished { src_peer_id, .. } => {
                crate::embedded_network::log_line(format!("inbound relay circuit established from {src_peer_id}"));
                info!("Inbound relay circuit established from {src_peer_id}");
            }
        },
        SwarmEvent::Behaviour(BehaviourEvent::RelayServer(event)) => {
            debug!("Relay server event: {event:?}");
        }
        SwarmEvent::Behaviour(BehaviourEvent::RendezvousClient(event)) => match event {
            rendezvous::client::Event::Registered {
                namespace,
                ttl,
                rendezvous_node,
            } => {
                info!(
                    "Registered namespace '{}' at rendezvous node {} for {}s",
                    namespace, rendezvous_node, ttl
                );
            }
            rendezvous::client::Event::RegisterFailed {
                rendezvous_node,
                namespace,
                error,
            } => {
                warn!(
                    "Failed to register namespace '{}' at {}: {:?}",
                    namespace, rendezvous_node, error
                );
            }
            rendezvous::client::Event::Discovered {
                registrations,
                cookie: _,
                ..
            } => {
                for registration in registrations {
                    let peer = registration.record.peer_id();
                    let mut discovered_addresses = Vec::new();
                    for address in registration.record.addresses() {
                        let p2p_suffix = Protocol::P2p(peer);
                        let address_with_p2p = if address.ends_with(&Multiaddr::empty().with(p2p_suffix.clone())) {
                            address.clone()
                        } else {
                            address.clone().with(p2p_suffix)
                        };

                        discovered_addresses.push(address_with_p2p.to_string());
                        info!("Rendezvous discovered peer {peer} at {address_with_p2p}");
                        if let Err(error) = swarm.dial(address_with_p2p) {
                            crate::embedded_network::log_line(format!("failed to dial rendezvous peer {peer}: {error:?}"));
                            warn!("Failed to dial rendezvous peer {peer}: {error:?}");
                        }
                    }
                    if !discovered_addresses.is_empty() {
                        crate::embedded_network::record_peer_discovery(
                            peer.to_string(),
                            "rendezvous",
                            discovered_addresses,
                        );
                    }
                }
            }
            other => {
                debug!("Rendezvous client event: {other:?}");
            }
        },
        SwarmEvent::Behaviour(BehaviourEvent::Rendezvous(event)) => {
            debug!("Rendezvous server event: {event:?}");
        }
        SwarmEvent::Behaviour(BehaviourEvent::Kademlia(kad::Event::OutboundQueryProgressed {
            result,
            ..
        })) => match result {
            kad::QueryResult::GetProviders(Ok(kad::GetProvidersOk::FoundProviders {
                providers,
                ..
            })) => {
                for provider in providers {
                    crate::embedded_network::log_line(format!(
                        "discovery provider found: {provider}"
                    ));
                    info!("Discovery provider found: {provider}");
                    if let Err(error) = swarm.dial(provider) {
                        crate::embedded_network::log_line(format!(
                            "failed to dial discovery provider {provider}: {error:?}"
                        ));
                        warn!("Failed to dial discovery provider {provider}: {error:?}");
                    }
                }
            }
            kad::QueryResult::GetProviders(Ok(kad::GetProvidersOk::FinishedWithNoAdditionalRecord {
                ..
            })) => {}
            kad::QueryResult::GetProviders(Err(err)) => {
                warn!("Failed to get providers: {err:?}");
            }
            kad::QueryResult::GetRecord(Ok(kad::GetRecordOk::FoundRecord(kad::PeerRecord {
                record,
                ..
            }))) => {
                crate::embedded_network::log_line(format!(
                    "get record key={:?} value={:?}",
                    std::str::from_utf8(record.key.as_ref()).unwrap_or("invalid utf8"),
                    std::str::from_utf8(&record.value).unwrap_or("invalid utf8"),
                ));
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
        SwarmEvent::Behaviour(BehaviourEvent::Gossipsub(event)) => match event {
            gossipsub::Event::Message {
                propagation_source,
                message_id: _,
                message,
            } => {
                let topic_str = message.topic.to_string();
                let message_text = String::from_utf8_lossy(&message.data);
                if topic_str == crate::network_config::active_discovery_topic() {
                    if let Ok(payload) = serde_json::from_slice::<serde_json::Value>(&message.data) {
                        if payload.get("peer_id").and_then(|value| value.as_str()).is_some() {
                            if let Err(error) =
                                crate::embedded_network::handle_presence_message(swarm, &message.data)
                            {
                                crate::embedded_network::log_line(format!(
                                    "failed to handle presence from {propagation_source:?}: {error}"
                                ));
                                warn!("Failed to handle presence from {propagation_source:?}: {error}");
                            }
                        }
                    }
                }
                crate::embedded_network::log_line(format!(
                    "received message '{message_text}' on topic '{topic_str}' from peer {propagation_source:?}"
                ));
                println!(
                    "Received message: '{}' on topic '{}' from peer: {:?}",
                    message_text, topic_str, propagation_source
                );
            }
            gossipsub::Event::Subscribed { peer_id, topic } => {
                warn!(
                    "Peer {:?} subscribed to topic '{}'",
                    peer_id,
                    topic.to_string()
                );
            }
            gossipsub::Event::Unsubscribed { peer_id, topic } => {
                warn!(
                    "Peer {:?} unsubscribed from topic '{}'",
                    peer_id,
                    topic.to_string()
                );
            }
            gossipsub::Event::GossipsubNotSupported { peer_id } => {
                debug!("Peer {:?} does not support Gossipsub", peer_id);
            }
        },
        _ => {}
    }
}
