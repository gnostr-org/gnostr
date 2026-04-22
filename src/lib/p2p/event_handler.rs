use libp2p::{autonat, dcutr, gossipsub, kad, mdns, relay, swarm::SwarmEvent};
use tracing::{debug, info, trace, warn};

use super::behaviour::BehaviourEvent;

pub async fn handle_swarm_event(
    swarm: &mut libp2p::Swarm<super::behaviour::Behaviour>,
    event: SwarmEvent<BehaviourEvent>,
) {
    match event {
        SwarmEvent::NewListenAddr { address, .. } => {
            warn!("Listening on {address}");
        }
        SwarmEvent::Behaviour(BehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
            for (peer_id, multiaddr) in list {
                info!("mDNS discovered a new peer: {peer_id}\n{multiaddr}");
                swarm
                    .behaviour_mut()
                    .kademlia
                    .add_address(&peer_id, multiaddr.clone());
                swarm
                    .behaviour_mut()
                    .autonat
                    .add_server(peer_id, Some(multiaddr));
            }
        }
        SwarmEvent::Behaviour(BehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
            for (peer_id, _multiaddr) in list {
                info!("mDNS peer expired: {peer_id}");
                swarm.behaviour_mut().autonat.remove_server(&peer_id);
            }
        }
        SwarmEvent::Behaviour(BehaviourEvent::Autonat(event)) => match event {
            autonat::Event::StatusChanged { old, new } => {
                info!("AutoNAT status changed: {old:?} -> {new:?}");
            }
            other => {
                debug!("AutoNAT event: {other:?}");
            }
        },
        SwarmEvent::Behaviour(BehaviourEvent::Dcutr(event)) => {
            debug!("DCUtR event: {event:?}");
        }
        SwarmEvent::Behaviour(BehaviourEvent::Relay(event)) => match event {
            relay::client::Event::ReservationReqAccepted { relay_peer_id, .. } => {
                info!("Relay reservation accepted by {relay_peer_id}");
            }
            relay::client::Event::OutboundCircuitEstablished { relay_peer_id, .. } => {
                info!("Relay circuit established via {relay_peer_id}");
            }
            relay::client::Event::InboundCircuitEstablished { src_peer_id, .. } => {
                info!("Inbound relay circuit established from {src_peer_id}");
            }
        },
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
        SwarmEvent::Behaviour(BehaviourEvent::Gossipsub(event)) => match event {
            gossipsub::Event::Message {
                propagation_source,
                message_id: _,
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
