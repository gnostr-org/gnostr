use libp2p::{
    gossipsub, identify, identity,
    kad::{
        self,
        store::{MemoryStore, MemoryStoreConfig},
        Config as KadConfig,
    },
    mdns, noise, ping, rendezvous,
    swarm::Swarm,
    tcp, yamux, PeerId,
};
use std::{
    error::Error,
    hash::{DefaultHasher, Hash, Hasher},
    time::Duration,
};
use tokio::io;
use tracing::info;

use crate::p2p::behaviour::Behaviour;
use crate::p2p::network_config::IPFS_PROTO_NAME;

pub fn build_swarm(keypair: identity::Keypair) -> Result<Swarm<Behaviour>, Box<dyn Error>> {
    let peer_id = PeerId::from(keypair.public());
    info!("Local PeerId: {}", peer_id);

    let message_id_fn = |message: &gossipsub::Message| {
        let mut s = DefaultHasher::new();
        message.data.hash(&mut s);
        info!("message:\n{0:?}", message);
        info!("message.data:\n{0:?}", message.data);
        info!("message.source:\n{0:?}", message.source);
        if let Some(source) = message.source {
            info!("message.source.peer_id:\n{0:?}", source);
            info!("message.source.peer_id:\n{0}", source.to_string());
        }
        info!("message.sequence_number:\n{0:?}", message.sequence_number);
        info!("message.topic:\n{0:?}", message.topic);
        info!("message.topic.hash:\n{0:0}", message.topic.clone());
        gossipsub::MessageId::from(s.finish().to_string())
    };

    let gossipsub_config = gossipsub::ConfigBuilder::default()
        .heartbeat_interval(Duration::from_secs(1))
        .validation_mode(gossipsub::ValidationMode::Permissive)
        .message_id_fn(message_id_fn)
        .build()
        .map_err(|msg| io::Error::new(io::ErrorKind::Other, msg))?;

    let swarm = libp2p::SwarmBuilder::with_existing_identity(keypair)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_quic()
        .with_dns()?
        .with_behaviour(move |key| {
            let kad_store_config = MemoryStoreConfig {
                max_provided_keys: usize::MAX,
                max_providers_per_key: usize::MAX,
                max_records: usize::MAX,
                max_value_bytes: usize::MAX,
            };
            let mut kad_config = KadConfig::new(IPFS_PROTO_NAME.clone());
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
                .expect("Valid gossipsub config"),
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

    Ok(swarm)
}
