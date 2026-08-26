use futures::stream::StreamExt;
use libp2p::{
    gossipsub,
    identify,
    kad::{self, store::{self}, Config as KadConfig},
    mdns, noise, ping,
    request_response::{self, ProtocolSupport},
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, StreamProtocol,
};
use std::error::Error;
use std::time::Duration;
use tokio::{io, select};
use tracing::{debug, warn};

use super::app::P2pEvent;

#[derive(NetworkBehaviour)]
pub struct MyBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
    pub identify: identify::Behaviour,
    pub kademlia: kad::Behaviour<kad::store::MemoryStore>,
    pub ping: ping::Behaviour,
}

pub async fn evt_loop(
    p2p_event_tx: tokio::sync::mpsc::Sender<P2pEvent>,
) -> Result<(), Box<dyn Error>> {
    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_quic()
        .with_behaviour(|key| {
            let gossipsub_config = gossipsub::ConfigBuilder::default()
                .heartbeat_interval(Duration::from_secs(10))
                .validation_mode(gossipsub::ValidationMode::Strict)
                .build()
                .map_err(|msg| io::Error::new(io::ErrorKind::Other, msg))?;

            let gossipsub = gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(key.clone()),
                gossipsub_config,
            )?;

            let mdns = libp2p::mdns::tokio::Behaviour::new(
                libp2p::mdns::Config::default(),
                key.public().to_peer_id(),
            )?;

            let identify = identify::Behaviour::new(identify::Config::new(
                "/ipfs/id/1.0.0".to_string(),
                key.public(),
            ));

            let kademlia = kad::Behaviour::new(
                key.public().to_peer_id(),
                kad::store::MemoryStore::new(key.public().to_peer_id()),
            );

            let ping = ping::Behaviour::new(ping::Config::new());

            Ok(MyBehaviour {
                gossipsub,
                mdns,
                identify,
                kademlia,
                ping,
            })
        })?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();

    let topic = gossipsub::IdentTopic::new("dashboard");
    swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

    swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    loop {
        select! {
            event = swarm.select_next_some() => match event {
                SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                    for (peer_id, _multiaddr) in list {
                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                        p2p_event_tx.send(P2pEvent::PeerConnected(peer_id.to_string())).await?;
                    }
                },
                SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                    for (peer_id, _multiaddr) in list {
                        if !swarm.is_connected(&peer_id) {
                            swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                            p2p_event_tx.send(P2pEvent::PeerDisconnected(peer_id.to_string())).await?;
                        }
                    }
                },
                _ => {}
            }
        }
    }
}
