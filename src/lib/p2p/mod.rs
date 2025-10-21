pub mod handle_input;
pub mod kvs;
pub mod opt;
pub mod chat;

use crate::blockhash::blockhash_async;
use crate::blockheight::blockheight_async;
use crate::p2p::chat::msg::{Msg, MsgKind};
use crate::p2p::chat::ChatSubCommands;
use chrono::{Local, Timelike};
use clap::ValueEnum;
use futures::stream::StreamExt;
use libp2p::StreamProtocol;
use libp2p::{
    //core::transport::Transport,
    gossipsub,
    //gossipsub::IdentTopic,
    identify, identity,
    kad::{
        self,
        // Kademlia, KademliaConfig, KademliaEvent,
        store::{MemoryStore, MemoryStoreConfig},
        Config as KadConfig,
    },
    mdns, noise, ping, rendezvous,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, Swarm,
};
use std::{
    env,
    error::Error,
    hash::{DefaultHasher, Hash, Hasher},
    str,
    str::FromStr,
    thread,
    //    time::Duration,
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

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Network {
    Kusama,
    Polkadot,
    Ipfs,
    Ursa,
}

impl Network {
    #[rustfmt::skip]
    fn bootnodes(&self) -> Vec<(Multiaddr, PeerId)> {
    match self {
    Network::Kusama => {
    vec![
    ("/dns/p2p.cc3-0.kusama.network/tcp/30100".parse().unwrap(), FromStr::from_str("12D3KooWDgtynm4S9M3m6ZZhXYu2RrWKdvkCSScc25xKDVSg1Sjd").unwrap()),
    ("/dns/p2p.cc3-1.kusama.network/tcp/30100".parse().unwrap(), FromStr::from_str("12D3KooWNpGriWPmf621Lza9UWU9eLLBdCFaErf6d4HSK7Bcqnv4").unwrap()),
    ("/dns/p2p.cc3-2.kusama.network/tcp/30100".parse().unwrap(), FromStr::from_str("12D3KooWLmLiB4AenmN2g2mHbhNXbUcNiGi99sAkSk1kAQedp8uE").unwrap()),
    ("/dns/p2p.cc3-3.kusama.network/tcp/30100".parse().unwrap(), FromStr::from_str("12D3KooWEGHw84b4hfvXEfyq4XWEmWCbRGuHMHQMpby4BAtZ4xJf").unwrap()),
    ("/dns/p2p.cc3-4.kusama.network/tcp/30100".parse().unwrap(), FromStr::from_str("12D3KooWF9KDPRMN8WpeyXhEeURZGP8Dmo7go1tDqi7hTYpxV9uW").unwrap()),
    ("/dns/p2p.cc3-5.kusama.network/tcp/30100".parse().unwrap(), FromStr::from_str("12D3KooWDiwMeqzvgWNreS9sV1HW3pZv1PA7QGA7HUCo7FzN5gcA").unwrap()),
    ("/dns/kusama-bootnode-0.paritytech.net/tcp/30333".parse().unwrap(), FromStr::from_str("12D3KooWSueCPH3puP2PcvqPJdNaDNF3jMZjtJtDiSy35pWrbt5h").unwrap()),
    ("/dns/kusama-bootnode-1.paritytech.net/tcp/30333".parse().unwrap(), FromStr::from_str("12D3KooWQKqane1SqWJNWMQkbia9qiMWXkcHtAdfW5eVF8hbwEDw").unwrap())
    ]
    }
    Network::Polkadot => {
    vec![
    // ("/dns/p2p.cc1-0.polkadot.network/tcp/30100".parse().unwrap(), FromStr::from_str("12D3KooWEdsXX9657ppNqqrRuaCHFvuNemasgU5msLDwSJ6WqsKc").unwrap()),
    ("/dns/p2p.cc1-1.polkadot.network/tcp/30100".parse().unwrap(), FromStr::from_str("12D3KooWAtx477KzC8LwqLjWWUG6WF4Gqp2eNXmeqAG98ehAMWYH").unwrap()),
    ("/dns/p2p.cc1-2.polkadot.network/tcp/30100".parse().unwrap(), FromStr::from_str("12D3KooWAGCCPZbr9UWGXPtBosTZo91Hb5M3hU8v6xbKgnC5LVao").unwrap()),
    ("/dns/p2p.cc1-3.polkadot.network/tcp/30100".parse().unwrap(), FromStr::from_str("12D3KooWJ4eyPowiVcPU46pXuE2cDsiAmuBKXnFcFPapm4xKFdMJ").unwrap()),
    ("/dns/p2p.cc1-4.polkadot.network/tcp/30100".parse().unwrap(), FromStr::from_str("12D3KooWNMUcqwSj38oEq1zHeGnWKmMvrCFnpMftw7JzjAtRj2rU").unwrap()),
    ("/dns/p2p.cc1-5.polkadot.network/tcp/30100".parse().unwrap(), FromStr::from_str("12D3KooWDs6LnpmWDWgZyGtcLVr3E75CoBxzg1YZUPL5Bb1zz6fM").unwrap()),
    ("/dns/cc1-0.parity.tech/tcp/30333".parse().unwrap(), FromStr::from_str("12D3KooWSz8r2WyCdsfWHgPyvD8GKQdJ1UAiRmrcrs8sQB3fe2KU").unwrap()),
    ("/dns/cc1-1.parity.tech/tcp/30333".parse().unwrap(), FromStr::from_str("12D3KooWFN2mhgpkJsDBuNuE5427AcDrsib8EoqGMZmkxWwx3Md4").unwrap()),
    ]
    }
    Network::Ipfs => {
    vec![
    ("/ip4/104.131.131.82/tcp/4001".parse().unwrap(), FromStr::from_str("QmaCpDMGvV2BGHeYERUEnRQAwe3N8SzbUtfsmvsqQLuvuJ").unwrap()),
    ("/dnsaddr/bootstrap.libp2p.io".parse().unwrap(), FromStr::from_str("QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN").unwrap()),
    ("/dnsaddr/bootstrap.libp2p.io".parse().unwrap(), FromStr::from_str("QmQCU2EcMqAqQPR2i9bChDtGNJchTbq5TbXJJ16u19uLTa").unwrap()),
    ("/dnsaddr/bootstrap.libp2p.io".parse().unwrap(), FromStr::from_str("QmbLHAnMoJPWSCR5Zhtx6BHJX9KiKNN6tpvbUcqanj75Nb").unwrap()),
    ("/dnsaddr/bootstrap.libp2p.io".parse().unwrap(), FromStr::from_str("QmcZf59bWwK5XFi76CZX8cbJ4BhTzzA3gU1ZjYZcYW3dwt").unwrap()),
    ]
    }
    Network::Ursa => {
    vec![
    ("/dns/bootstrap-node-0.ursa.earth/tcp/6009".parse().unwrap(), FromStr::from_str("12D3KooWDji7xMLia6GAsyr4oiEFD2dd3zSryqNhfxU3Grzs1r9p").unwrap()),
    ]
    }
    }
    }

    fn protocol(&self) -> Option<String> {
        match self {
            Network::Kusama => Some("/ksmcc3/kad".to_string()),
            Network::Polkadot => Some("/dot/kad".to_string()),
            Network::Ipfs => None,
            Network::Ursa => Some("/ursa/kad/0.0.1".to_string()),
        }
    }
}

const IPFS_BOOTNODES: [&str; 6] = [
    "QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN",
    "QmQCU2EcMqAqQPR2i9bChDtGNJchTbq5TbXJJ16u19uLTa",
    "QmbLHAnMoJPWSCR5Zhtx6BHJX9KiKNN6tpvbUcqanj75Nb",
    "QmcZf59bWwK5XFi76CZX8cbJ4BhTzzA3gU1ZjYZcYW3dwt",
    "12D3KooWH1URV3uTNQW6SZ1UFDnHN8NXwznAA8JeETTBm8oimjh9",
    "12D3KooWFhXabKDwALpzqMbto94sB7rvmZ6M28hs9Y9xSopDKwQr",
];
const IPFS_PROTO_NAME: StreamProtocol = StreamProtocol::new("/ipfs/kad/1.0.0");

/// MyBehaviour
// We create a custom network behaviour that combines Gossipsub and Mdns.
#[derive(NetworkBehaviour)]
struct MyBehaviour {
    ipfs: kad::Behaviour<kad::store::MemoryStore>,
    kademlia: kad::Behaviour<kad::store::MemoryStore>,
    mdns: mdns::tokio::Behaviour,
    identify: identify::Behaviour,
    rendezvous: rendezvous::server::Behaviour,
    ping: ping::Behaviour,
    gossipsub: gossipsub::Behaviour,
}

pub fn generate_ed25519(secret_key_seed: &[u8]) -> identity::Keypair {
    // let mut bytes = [0u8; 32];
    let mut bytes: [u8; 32] = GNOSTR_SHA256; //[
                                             //    0xca, 0x45, 0xfe, 0x80, 0x0a, 0x2c, 0x3b, 0x67, //
                                             //    0x8e, 0x0a, 0x87, 0x7a, 0xa7, 0x7e, 0x36, 0x76, //
                                             //    0x34, 0x0a, 0x59, 0xc9, 0xa7, 0x61, 0x5e, 0x30, //
                                             //    0x59, 0x76, 0xfb, 0x9b, 0xa8, 0xda, 0x48, 0x06, //
                                             //];

    bytes[31] = bytes[31] ^ secret_key_seed[31];
    for (i, byte) in bytes.iter().enumerate() {
        // Print context: the index and value (decimal and hex) of the current byte.
        trace!("Byte {:02} [{:3} / {:#04x}]: ", i, byte, byte);

        // A `u8` has 8 bits. We iterate from 7 down to 0 to print
        // the most significant bit (MSB) first.
        for j in (0..8).rev() {
            // Create a "mask" by shifting the number 1 to the left `j` times.
            // For j=7, mask is 10000000
            // For j=0, mask is 00000001
            let mask = 1 << j;

            // Use the bitwise AND operator `&` to check if the bit at the mask's
            // position is set. If the result is not 0, the bit is 1.
            if byte & mask == 0 {
                trace!("0");
            } else {
                trace!("1");
            }
        }
        // Add a newline to separate the output for each byte.
        trace!("\n");
    }

    // bytes[31] = secret_key_seed;

    for (i, byte) in bytes.iter().enumerate() {
        // Print context: the index and value (decimal and hex) of the current byte.
        trace!("Byte {:02} [{:3} / {:#04x}]: ", i, byte, byte);

        // A `u8` has 8 bits. We iterate from 7 down to 0 to print
        // the most significant bit (MSB) first.
        for j in (0..8).rev() {
            // Create a "mask" by shifting the number 1 to the left `j` times.
            // For j=7, mask is 10000000
            // For j=0, mask is 00000001
            let mask = 1 << j;

            // Use the bitwise AND operator `&` to check if the bit at the mask's
            // position is set. If the result is not 0, the bit is 1.
            if byte & mask == 0 {
                trace!("0");
            } else {
                trace!("1");
            }
        }
        // Add a newline to separate the output for each byte.
        trace!("\n");
    }

    let keypair =
        identity::Keypair::ed25519_from_bytes(bytes).expect("only errors on wrong length");
    // println!("141:{}", keypair.public().to_peer_id());
    generate_close_peer_id(bytes.clone(), 32usize);
    keypair
}

fn generate_close_peer_id(bytes: [u8; 32], common_bits: usize) -> PeerId {
    let mut close_bytes = [0u8; 32];
    close_bytes = bytes;

    for (i, byte) in close_bytes.iter().enumerate() {
        if i < 32 {
            // Print context: the index and value (decimal and hex) of the current byte.
            trace!("Byte i={:02} [{:3} / {:#04x}]: ", i, byte, byte);

            // A `u8` has 8 bits. We iterate from 7 down to 0 to print
            // the most significant bit (MSB) first.
            for j in (0..8).rev() {
                // Create a "mask" by shifting the number 1 to the left `j` times.
                // For j=7, mask is 10000000
                // For j=0, mask is 00000001
                let mask = 1 << j;

                // Use the bitwise AND operator `&` to check if the bit at the mask's
                // position is set. If the result is not 0, the bit is 1.
                if byte & mask == 0 {
                    trace!("0");
                } else {
                    trace!("1");
                }
            }
            // Add a newline to separate the output for each byte.
            trace!("\n");
        } // end if
    }
    let mut keypair =
        identity::Keypair::ed25519_from_bytes(close_bytes).expect("only errors on wrong length");
    trace!("262:{}", keypair.public().to_peer_id());

    close_bytes[31] = bytes[31] ^ 0u8;

    for (i, byte) in close_bytes.iter().enumerate() {
        // Print context: the index and value (decimal and hex) of the current byte.
        trace!("265:Byte {:02} [{:3} / {:#04x}]: ", i, byte, byte);

        // A `u8` has 8 bits. We iterate from 7 down to 0 to print
        // the most significant bit (MSB) first.
        for j in (0..8).rev() {
            // Create a "mask" by shifting the number 1 to the left `j` times.
            // For j=7, mask is 10000000
            // For j=0, mask is 00000001
            let mask = 1 << j;

            // Use the bitwise AND operator `&` to check if the bit at the mask's
            // position is set. If the result is not 0, the bit is 1.
            if byte & mask == 0 {
                trace!("0");
            } else {
                trace!("1");
            }
        }
        // Add a newline to separate the output for each byte.
        trace!("");
    }

    keypair =
        identity::Keypair::ed25519_from_bytes(close_bytes).expect("only errors on wrong length");
    trace!("292:{}", keypair.public().to_peer_id());
    keypair.public().to_peer_id()
}

const GNOSTR_HEX_STR: &str = "ca45fe800a2c3b678e0a877aa77e3676340a59c9a7615e305976fb9ba8da4806";

const GNOSTR_SHA256: [u8; 32] = [
    0xca, 0x45, 0xfe, 0x80, 0x0a, 0x2c, 0x3b, 0x67, 0x8e, 0x0a, 0x87, 0x7a, 0xa7, 0x7e, 0x36, 0x76,
    0x34, 0x0a, 0x59, 0xc9, 0xa7, 0x61, 0x5e, 0x30, 0x59, 0x76, 0xfb, 0x9b, 0xa8, 0xda, 0x48, 0x06,
];

/// evt_loop
pub async fn evt_loop(
    args: ChatSubCommands,
    mut send: tokio::sync::mpsc::Receiver<Msg>,
    recv: tokio::sync::mpsc::Sender<Msg>,
    topic: gossipsub::IdentTopic,
) -> Result<(), Box<dyn Error>> {
    let keypair: identity::Keypair = generate_ed25519(&*args.nsec.clone().unwrap().as_bytes());
    let keypair_clone: identity::Keypair = generate_ed25519(&*args.nsec.unwrap().as_bytes());
    let public_key = keypair.public();
    let peer_id = PeerId::from_public_key(&public_key);
    warn!("Local PeerId: {}", peer_id);

    // kad_store_config
    let kad_store_config = MemoryStoreConfig {
        max_provided_keys: usize::MAX,
        max_providers_per_key: usize::MAX,
        max_records: usize::MAX,
        max_value_bytes: usize::MAX,
    };
    let _kad_memstore = MemoryStore::with_config(peer_id.clone(), kad_store_config.clone());
	let _kad_config = KadConfig::new(IPFS_PROTO_NAME);
    let message_id_fn = |message: &gossipsub::Message| {
        let mut s = DefaultHasher::new();
        message.data.hash(&mut s);
        info!("message:\n{0:?}", message);
        info!("message.data:\n{0:?}", message.data);
        info!("message.source:\n{0:?}", message.source);
        info!("message.source:\n{0:1?}", message.source);
        info!("message.source.peer_id:\n{0:2?}", message.source.unwrap());
        // TODO https://docs.rs/gossipsub/latest/gossipsub/trait.DataTransform.html
        // send Recieved message back
        info!(
            "message.source.peer_id:\n{0:3}",
            message.source.unwrap().to_string()
        );
        info!("message.sequence_number:\n{0:?}", message.sequence_number);
        info!("message.topic:\n{0:?}", message.topic);
        info!("message.topic.hash:\n{0:0}", message.topic.clone());
        // println!("{:?}", s);
        gossipsub::MessageId::from(s.finish().to_string())
    };
    let gossipsub_config = gossipsub::ConfigBuilder::default()
        .heartbeat_interval(Duration::from_secs(1))
        .validation_mode(gossipsub::ValidationMode::Permissive)
        //.message_id_fn(message_id_fn)
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
            let mut kad_config = KadConfig::new(IPFS_PROTO_NAME);
            kad_config.set_query_timeout(Duration::from_secs(120));
            kad_config.set_replication_factor(std::num::NonZeroUsize::new(20).unwrap());
            kad_config.set_publication_interval(Some(Duration::from_secs(10)));
            kad_config.disjoint_query_paths(false);
            let kad_store = MemoryStore::with_config(peer_id.clone(), kad_store_config);
            let mut ipfs_cfg = kad::Config::new(IPFS_PROTO_NAME);
            ipfs_cfg.set_query_timeout(Duration::from_secs(5 * 60));
            let ipfs_store = kad::store::MemoryStore::new(key.public().to_peer_id());
            Ok(MyBehaviour {
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
                    //let m = Msg::default().set_content("p2p.rs:brief help prompt here!:2".to_string(), 2).set_kind(MsgKind::System);
                    ////NOTE:recv.send - send to self
                    //recv.send(m).await?;
                }
            }
            event = swarm.select_next_some() => match event {
                SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                    for (peer_id, _multiaddr) in list {
                        debug!("mDNS discovered a new peer: {peer_id}");
                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                    // let m = Msg::default().set_content(format!("discovered new peer: {peer_id}")).set_kind(MsgKind::System);
                        // recv.send(m).await?;
                    }
                },
                SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                    for (peer_id, _multiaddr) in list {
                        debug!("mDNS discover peer has expired: {peer_id}");
                        swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                        // let m = Msg::default().set_content(format!("peer expired: {peer_id}")).set_kind(MsgKind::System);
                        // recv.send(m).await?;
                    }
                },
                //
                SwarmEvent::Behaviour(MyBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                    propagation_source: peer_id,
                    message_id: id,
                    message,
                })) => {
                    debug!(
                        "Got message: '{}' with id: {id} from peer: {peer_id}",
                        String::from_utf8_lossy(&message.data),
                        //add type indicator to Msg::content[]
                        //send git commit info
                    );
                    match serde_json::from_slice::<Msg>(&message.data) {
                        //NOTE: from slice - reference serialized_commit!
                        //use MsgType::GitCommit
                        Ok(msg) => {
                            recv.send(msg).await?;
                        },
                        Err(e) => {
                            warn!("Error deserializing message: {e:?}");
                            let m = Msg::default().set_content(format!("Error deserializing message: {e:?}"), 0_usize).set_kind(MsgKind::System);
                            //NOTE recv.send - send to self
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