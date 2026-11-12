use libp2p::{
    autonat, dcutr, gossipsub, identify, kad, mdns, ping, relay, rendezvous,
    swarm::NetworkBehaviour,
};

#[derive(NetworkBehaviour)]
pub struct Behaviour {
    pub relay_client: relay::client::Behaviour,
    pub relay_server: relay::Behaviour,
    pub autonat: autonat::Behaviour,
    pub dcutr: dcutr::Behaviour,
    pub ipfs: kad::Behaviour<kad::store::MemoryStore>,
    pub kademlia: kad::Behaviour<kad::store::MemoryStore>,
    pub mdns: mdns::tokio::Behaviour,
    pub identify: identify::Behaviour,
    pub rendezvous_client: rendezvous::client::Behaviour,
    pub rendezvous: rendezvous::server::Behaviour,
    pub ping: ping::Behaviour,
    pub gossipsub: gossipsub::Behaviour,
}
