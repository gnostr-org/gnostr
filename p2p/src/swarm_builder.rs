use std::{
    error::Error,
    hash::{DefaultHasher, Hash, Hasher},
    time::Duration,
};

use libp2p::{
    autonat, dcutr, gossipsub, identify, identity, relay,
    kad::{
        self,
        store::{MemoryStore, MemoryStoreConfig},
        Config as KadConfig,
    },
    mdns, noise, ping, rendezvous,
    swarm::Swarm,
    tcp, yamux, PeerId, StreamProtocol,
};
use tokio::io;
use tracing::info;

use crate::p2p::{behaviour::Behaviour, network_config::active_protocol_name};

fn build_behaviour(
    key: &identity::Keypair,
    relay_client: libp2p::relay::client::Behaviour,
) -> Result<Behaviour, Box<dyn Error + Send + Sync>> {
    let message_id_fn = |message: &gossipsub::Message| {
        let mut s = DefaultHasher::new();
        message.data.hash(&mut s);
        crate::embedded_network::log_line(format!("message topic={}", message.topic));
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

    #[allow(clippy::redundant_closure)]
    let gossipsub_config = gossipsub::ConfigBuilder::default()
        .heartbeat_interval(Duration::from_secs(1))
        .validation_mode(gossipsub::ValidationMode::Permissive)
        .message_id_fn(message_id_fn)
        .build()
        .map_err(|msg| io::Error::other(msg))?;

    let local_peer_id = key.public().to_peer_id();
    let kad_store_config = MemoryStoreConfig {
        max_provided_keys: usize::MAX,
        max_providers_per_key: usize::MAX,
        max_records: usize::MAX,
        max_value_bytes: usize::MAX,
    };
    let protocol_name = active_protocol_name();
    let mut kad_config = KadConfig::new(StreamProtocol::try_from_owned(protocol_name.clone()).unwrap());
    kad_config.set_query_timeout(Duration::from_secs(120));
    kad_config.set_replication_factor(std::num::NonZeroUsize::new(20).unwrap());
    kad_config.set_publication_interval(Some(Duration::from_secs(10)));
    kad_config.disjoint_query_paths(false);
    let kad_store = MemoryStore::with_config(local_peer_id, kad_store_config);
    let mut ipfs_cfg = KadConfig::new(StreamProtocol::try_from_owned(protocol_name).unwrap());
    ipfs_cfg.set_query_timeout(Duration::from_secs(5 * 60));
    let ipfs_store = MemoryStore::new(local_peer_id);

    let relay_server = relay::Behaviour::new(local_peer_id, Default::default());
    let rendezvous_client = rendezvous::client::Behaviour::new(key.clone());
    let rendezvous_server = rendezvous::server::Behaviour::new(rendezvous::server::Config::default());

    Ok(Behaviour {
        relay_client,
        relay_server,
        autonat: autonat::Behaviour::new(local_peer_id, autonat::Config::default()),
        dcutr: dcutr::Behaviour::new(local_peer_id),
        gossipsub: gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(key.clone()),
            gossipsub_config,
        )
        .expect("Valid gossipsub config"),
        ipfs: kad::Behaviour::with_config(local_peer_id, ipfs_store, ipfs_cfg),
        kademlia: kad::Behaviour::with_config(local_peer_id, kad_store, kad_config),
        identify: identify::Behaviour::new(identify::Config::new(
            "/yamux/1.0.0".to_string(),
            key.public(),
        )),
        rendezvous_client,
        rendezvous: rendezvous_server,
        ping: ping::Behaviour::new(ping::Config::new().with_interval(Duration::from_secs(5))),
        mdns: mdns::tokio::Behaviour::new(mdns::Config::default(), local_peer_id)?,
    })
}

pub async fn build_swarm(
    keypair: identity::Keypair,
) -> Result<Swarm<Behaviour>, Box<dyn Error + Send + Sync>> {
    let peer_id = PeerId::from(keypair.public());
    crate::embedded_network::log_line(format!("local peer id: {peer_id}"));
    info!("Local PeerId: {}", peer_id);

    #[cfg(feature = "tor")]
    let tor_transport = crate::tor::build_transport(&keypair).await?;

    let builder = libp2p::SwarmBuilder::with_existing_identity(keypair)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        ;

    let builder = builder.with_quic();

    #[cfg(feature = "tor")]
    let builder = builder.with_other_transport(move |_| tor_transport)?;

    #[cfg(all(not(target_os = "ios"), not(target_os = "tvos")))]
    let swarm = builder
        .with_dns()?
        .with_websocket(noise::Config::new, yamux::Config::default)
        .await?
        .with_relay_client(noise::Config::new, yamux::Config::default)?
        .with_behaviour(|key, relay_client| build_behaviour(key, relay_client))?
        .build();

    #[cfg(any(target_os = "ios", target_os = "tvos"))]
    let swarm = builder
        .with_relay_client(noise::Config::new, yamux::Config::default)?
        .with_behaviour(|key, relay_client| build_behaviour(key, relay_client))?
        .build();

    Ok(swarm)
}

#[cfg(test)]
mod tests {
    use super::*;
    use libp2p::Multiaddr;

    #[tokio::test]
    async fn build_swarm_accepts_websocket_listen_address() {
        let keypair = crate::keypair_from_seed(Some(
            gnostr_asyncgit::default_gnostr_private_key_hex(),
        ));
        let mut swarm = build_swarm(keypair).await.expect("swarm");
        let addr: Multiaddr = "/ip4/127.0.0.1/tcp/0/ws".parse().expect("websocket multiaddr");
        swarm.listen_on(addr).expect("websocket listen");
    }
}
