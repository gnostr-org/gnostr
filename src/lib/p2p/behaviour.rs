use libp2p::prelude::*;
use libp2p::{gossipsub, identify, kad, mdns, ping, rendezvous, relay};
use libp2p_autonat as autonat;

#[derive(NetworkBehaviour)]
pub struct Behaviour {
    pub ipfs: kad::Behaviour<kad::store::MemoryStore>,
    pub kademlia: kad::Behaviour<kad::store::MemoryStore>,
    pub mdns: mdns::tokio::Behaviour,
    pub identify: identify::Behaviour,
    pub rendezvous: rendezvous::server::Behaviour,
    pub ping: ping::Behaviour,
    pub gossipsub: gossipsub::Behaviour,
    pub relay: relay::Behaviour,
    pub autonat: autonat::Behaviour,
}
