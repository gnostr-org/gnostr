use ansi_term::Style;
use futures::future::TryFutureExt;
use futures::stream::StreamExt;
use libp2p::core::transport::Transport;
use libp2p::core::ConnectedPoint;
use libp2p::identify;
use libp2p::identity::Keypair;
use libp2p::kad::ProgressStep;
use libp2p::kad::{
    store::MemoryStore, GetClosestPeersOk,
    QueryResult,
};
use libp2p::ping;
use libp2p::relay;
use libp2p::{
    noise,
    swarm::{NetworkBehaviour},
    tcp,
    yamux,
    Multiaddr,
    PeerId,
    Swarm,
    StreamProtocol,
    SwarmBuilder,
};
use libp2p::swarm::SwarmEvent;
use log::debug;
use std::time::Duration;
use clap::{Parser, ValueEnum};
use crate::p2p::network_config::Network;
use thiserror::Error;



fn print_key(k: &str, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    writeln!(f, "{}:", Style::new().bold().paint(k))
}

fn print_key_value<V: std::fmt::Debug>(
    k: &str,
    v: V,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}: {v:?}", Style::new().bold().paint(k))
}

pub struct LookupClient {
    swarm: Swarm<LookupBehaviour>,
}

pub struct Peer {
    peer_id: PeerId,
    protocol_version: String,
    agent_version: String,
    listen_addrs: Vec<Multiaddr>,
    protocols: Vec<StreamProtocol>,
    observed_addr: Multiaddr,
}

impl std::fmt::Display for Peer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        print_key_value("PeerId", self.peer_id.clone(), f)?;
        print_key_value("Protocol version", self.protocol_version.clone(), f)?;
        print_key_value("Agent version", self.agent_version.clone(), f)?;
        print_key_value("Observed address", self.observed_addr.clone(), f)?;
        if !self.listen_addrs.is_empty() {
            print_key("Listen addresses", f)?;
            for addr in &self.listen_addrs {
                writeln!(f, "\t- {addr:?}")?;
            }
        }
        if !self.protocols.is_empty() {
            print_key("Protocols", f)?;
            for protocol in &self.protocols {
                writeln!(f, "\t- {protocol}")?;
            }
        }

        Ok(())
    }
}

impl LookupClient {
    pub fn new(network: Option<Network>) -> Self {
        // Create a random key for ourselves.
        let local_key = Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());

        //println!("Local peer id: {local_peer_id}");



            let (relay_transport, relay_client) = relay::client::new(local_peer_id);
            let mut swarm = SwarmBuilder::with_existing_identity(local_key)
                .with_async_std()
                .with_tcp(
                    tcp::Config::default(),
                    noise::Config::new,
                    yamux::Config::default,
                )
                .unwrap()
                .with_quic()
                .with_relay_client(noise::Config::new, yamux::Config::default)
                .unwrap()
                .with_behaviour(|key, relay_client| {
                    let local_peer_id = PeerId::from(key.public());

                    // Create a Kademlia behaviour.
                    let store = MemoryStore::new(local_peer_id);
                    let mut kademlia_config = libp2p::kad::Config::default();
                    if let Some(protocol_name) = network.clone().and_then(|n| n.protocol()) {
                        kademlia_config
                            .set_protocol_names(vec![
                                StreamProtocol::try_from_owned(protocol_name).unwrap()
                            ]);
                    }
                    let kademlia = libp2p::kad::Behaviour::new(local_peer_id, store);

                    let ping = ping::Behaviour::new(ping::Config::new());

                    let user_agent =
                        "substrate-node/v2.0.0-e3245d49d-x86_64-linux-gnu (unknown)".to_string();
                    let proto_version = "/substrate/1.0".to_string();
                    let identify = identify::Behaviour::new(
                        identify::Config::new(proto_version, key.public())
                            .with_agent_version(user_agent),
                    );

                    LookupBehaviour {
                        kademlia,
                        ping,
                        identify,
                        relay: relay_client,
                    }
                })
                .unwrap()
                .build();

        if let Some(network) = network {
            for (addr, peer_id) in network.bootnodes() {
                swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
            }
        }

        LookupClient { swarm }
    }

    pub async fn lookup_directly(mut self, dst_addr: Multiaddr) -> Result<Peer, LookupError> {
        self.swarm.dial(dst_addr.clone()).unwrap();

        loop {
            match self.swarm.next().await.expect("Infinite Stream.") {
                SwarmEvent::ConnectionEstablished {
                    peer_id,
                    endpoint,
                    num_established,
                    concurrent_dial_errors: _,
                    established_in: _,
                    connection_id: _,
                } => {
                    assert_eq!(Into::<u32>::into(num_established), 1);
                    match endpoint {
                        ConnectedPoint::Dialer {
                            address,
                            role_override: _,
                            ..
                        } => {
                            if address == dst_addr {
                                return self.wait_for_identify(peer_id).await;
                            }
                        }
                        ConnectedPoint::Listener { .. } => {}
                    }
                }
                SwarmEvent::OutgoingConnectionError {
                    peer_id: _,
                    connection_id: _,
                    error,
                } => return Err(LookupError::FailedToDialPeer { error }),
                SwarmEvent::Dialing { .. } => {}
                SwarmEvent::Behaviour(_) => {
                    // Ignore any behaviour events until we are connected to the
                    // destination peer. These should be events from the
                    // connection to a relay only.
                }
                e => panic!("{:?}", e),
            }
        }
    }

    pub async fn lookup_on_dht(mut self, peer: PeerId) -> Result<Peer, LookupError> {
        self.swarm.behaviour_mut().kademlia.get_closest_peers(peer);

        loop {
            match self.swarm.next().await.expect("Infinite Stream.") {
                SwarmEvent::ConnectionEstablished {
                    peer_id,
                    num_established,
                    ..
                } => {
                    assert_eq!(Into::<u32>::into(num_established), 1);
                    if peer_id == peer {
                        return self.wait_for_identify(peer).await;
                    }
                }
                SwarmEvent::Behaviour(LookupBehaviourEvent::Kademlia(
                    libp2p::kad::Event::OutboundQueryProgressed {
                        result: QueryResult::Bootstrap(_),
                        ..
                    },
                )) => {
                    panic!("Unexpected bootstrap.");
                }
                SwarmEvent::Behaviour(LookupBehaviourEvent::Kademlia(
                    libp2p::kad::Event::OutboundQueryProgressed {
                        result: QueryResult::GetClosestPeers(Ok(GetClosestPeersOk { peers, .. })),
                        step: ProgressStep { count: _, last },
                        ..
                    },
                )) => {
                    if peers.iter().any(|p| p.peer_id == peer) {
                        if !Swarm::is_connected(&self.swarm, &peer) {
                            // TODO: Kademlia might not be caching the address of the peer.
                            Swarm::dial(&mut self.swarm, peer).unwrap();
                        }

                        return self.wait_for_identify(peer).await;
                    }

                    if last {
                        return Err(LookupError::FailedToFindPeerOnDht);
                    }
                }
                _ => {}
            }
        }
    }

    async fn wait_for_identify(&mut self, peer: PeerId) -> Result<Peer, LookupError> {
        loop {
            match self.swarm.next().await.expect("Infinite Stream.") {
                SwarmEvent::Behaviour(LookupBehaviourEvent::Identify(
                    identify::Event::Received {
                        peer_id,
                        info:
                            identify::Info {
                                protocol_version,
                                agent_version,
                                listen_addrs,
                                protocols,
                                observed_addr,
                                ..
                            },
                        ..
                    },
                )) => {
                    if peer_id == peer {
                        return Ok(Peer {
                            peer_id,
                            protocol_version,
                            agent_version,
                            listen_addrs,
                            protocols,
                            observed_addr,
                        });
                    }
                }
                e => debug!("{e:?}"),
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum LookupError {
    #[error("Looking up the given peer timed out")]
    Timeout,
    #[error("Failed to dial peer {error}")]
    FailedToDialPeer { error: libp2p::swarm::DialError },
    #[error("Failed to find peer on DHT")]
    FailedToFindPeerOnDht,
}

#[derive(NetworkBehaviour)]
struct LookupBehaviour {
    pub(crate) kademlia: libp2p::kad::Behaviour<MemoryStore>,
    pub(crate) ping: ping::Behaviour,
    pub(crate) identify: identify::Behaviour,
    relay: relay::client::Behaviour,
}


