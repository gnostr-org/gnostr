use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::iter;
use std::net::IpAddr;
use std::path::Path;
use std::time::Duration;
#[cfg(test)]
use std::{println as debug, println as error, println as info, println as trace, println as warn};

//  use std::time::Instant;
//  use tokio::time::Instant;
use anyhow::{Context, Result};
use clap::Parser;
use duration_str::parse;
use futures::future::{select, Either};
use futures::StreamExt;
use gnostr_chat::protocol::*;
use libp2p::core::muxing::StreamMuxerBox;
use libp2p::kad::record::store::MemoryStore;
use libp2p::kad::{Kademlia, KademliaConfig};
use libp2p::multiaddr::{Multiaddr, Protocol};
use libp2p::request_response::{self, ProtocolSupport};
use libp2p::swarm::{NetworkBehaviour, Swarm, SwarmBuilder, SwarmEvent};
use libp2p::{
    dns, gossipsub, identify, identity, memory_connection_limits, quic, relay, PeerId,
    StreamProtocol, Transport,
};
use libp2p_webrtc as webrtc;
use libp2p_webrtc::tokio::Certificate;
#[cfg(not(test))]
use log::{debug, error, info, trace, warn};
use tokio::fs; // Workaround to use prinltn! for logs.

#[cfg(not(debug_assertions))]
const TICK_INTERVAL: Duration = Duration::from_secs(15);
#[cfg(debug_assertions)]
const TICK_INTERVAL: Duration = Duration::from_secs(2);

const KADEMLIA_PROTOCOL_NAME: StreamProtocol = StreamProtocol::new("/ipfs/kad/1.0.0");
const FILE_EXCHANGE_PROTOCOL: StreamProtocol =
    StreamProtocol::new("/universal-connectivity-file/1");
const PORT_WEBRTC: u16 = 9090;
const PORT_QUIC: u16 = 9091;
const LOCAL_KEY_PATH: &str = "./local_key";
const LOCAL_CERT_PATH: &str = "./cert.pem";
//const GOSSIPSUB_CHAT_TOPIC: &str = "gnostr";
const GOSSIPSUB_CHAT_FILE_TOPIC: &str = "universal-connectivity-file";
const BOOTSTRAP_NODES: [&str; 5] = [
    "/dnsaddr/universal-connectiviy.fly.io/p2p/\
     12D3KooWGahRw3ZnM4gAyd9FK75v4Bp5keFYTvkcAwhpEm28wbV3",
    "/dnsaddr/bootstrap.libp2p.io/p2p/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN",
    "/dnsaddr/bootstrap.libp2p.io/p2p/QmQCU2EcMqAqQPR2i9bChDtGNJchTbq5TbXJJ16u19uLTa",
    "/dnsaddr/bootstrap.libp2p.io/p2p/QmbLHAnMoJPWSCR5Zhtx6BHJX9KiKNN6tpvbUcqanj75Nb",
    "/dnsaddr/bootstrap.libp2p.io/p2p/QmcZf59bWwK5XFi76CZX8cbJ4BhTzzA3gU1ZjYZcYW3dwt",
];

//const GNOSTR_CONNECT_DEFAULT_SEEDER: &str =
//  "/ip4/37.16.6.234/udp/9091/quic-v1/p2p/
// 12D3KooWSAXQZuzHEKgau7HtyPc3EzArc8VG3Nh9TTYx4Sumip89";
///ip4/37.16.6.234/udp/9091/quic-v1/p2p/
/// 12D3KooWSAXQZuzHEKgau7HtyPc3EzArc8VG3Nh9TTYx4Sumip89 dns/gnostr-connect.fly.
/// dev/udp/9091/quic-v1/p2p/
/// 12D3KooWSAXQZuzHEKgau7HtyPc3EzArc8VG3Nh9TTYx4Sumip89

#[derive(Debug, Parser)]
#[clap(name = "gnostr-chat")]
struct Opt {
    /// Address to listen on.
    #[clap(long, default_value = "0.0.0.0")]
    listen_address: IpAddr,

    /// If known, the external address of this node. Will be used to correctly
    /// advertise our external address across all transports.
    #[clap(long, env)]
    external_address: Option<IpAddr>,

    /// Nodes to connect to on startup. Can be specified several times.
    #[clap(
        long,
        //default_value = "/dns/gnostr-connect.fly.dev/udp/9091/quic-v1",
        //default_value = GNOSTR_CONNECT_DEFAULT_SEEDER,
        default_value = "/dns/universal-connectivity-rust-peer.fly.dev/udp/9091/quic-v1",
    )]
    connect: Vec<Multiaddr>,
    /// Topic
    #[clap(
        long,
        //default_value = "/dns/gnostr-connect.fly.dev/udp/9091/quic-v1"
        default_value = "gnostr"
    )]
    topic: String,
    // Tick
    #[clap(long, default_value = "10")]
    tick: String,
}

fn init_logger() {
    use env_logger::Builder;
    Builder::from_env(env_logger::Env::default().filter_or("LOG_LEVEL", "None"))
        .format(|buf, record| {
            writeln!(
                buf,
                "{}: {}: {}",
                record.level(),
                record.target(),
                record.args()
            )
        })
        //.format_timestamp(None)
        .format_timestamp_nanos()
        //.format_level(false)
        //.write_style_or("MY_LOG_STYLE", "always")
        .init();
}

/// An example WebRTC peer that will accept connections
#[tokio::main]
async fn main() -> Result<()> {
    init_logger();
    //let opt = Opt::parse();
    //info!(x="45"; "Some message");
    //info!(x="12"; "Another message {x}", x="12");
    //let data = (42, "Forty-two");
    //let private_data = "private";

    //log::log!(log::Level::Error, "Received errors: {}, {}", data.0, data.1);
    //log::log!(target: "app_events", log::Level::Warn, "App warning: {}, {}, {}",
    //data.0, data.1, private_data);

    //#[cfg(debug_assertions)]
    //env_logger::builder()
    //  .format(|buf, record| writeln!(buf, "{}: {}", record.level(),
    // record.args()))
    //.init();
    //#[cfg(not(debug_assertions))]
    //env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("
    // info"))
    //.format_timestamp(None)
    //.init();
    let opt = Opt::parse();

    let local_key = read_or_create_identity(Path::new(LOCAL_KEY_PATH))
        .await
        .context("Failed to read identity")?;
    let webrtc_cert = read_or_create_certificate(Path::new(LOCAL_CERT_PATH))
        .await
        .context("Failed to read certificate")?;

    //topic from gnostr-chat --topic <string>
    let mut swarm = create_swarm(local_key, webrtc_cert, opt.topic.clone())?;

    let address_webrtc = Multiaddr::from(opt.listen_address)
        .with(Protocol::Udp(PORT_WEBRTC))
        .with(Protocol::WebRTCDirect);

    let address_quic = Multiaddr::from(opt.listen_address)
        .with(Protocol::Udp(PORT_QUIC))
        .with(Protocol::QuicV1);

    swarm
        .listen_on(address_webrtc.clone())
        .expect("listen on webrtc");
    swarm
        .listen_on(address_quic.clone())
        .expect("listen on quic");

    for addr in opt.connect {
        if let Err(e) = swarm.dial(addr.clone()) {
            //debug!("Failed to dial {addr}: {e}");
            trace!("Failed to dial {addr}: {e}");
        }
    }

    for peer in BOOTSTRAP_NODES {
        print!("{}\n", peer);
        let multiaddr: Multiaddr = peer.parse().expect("Failed to parse Multiaddr");
        if let Err(e) = swarm.dial(multiaddr) {
            //debug!("Failed to dial {peer}: {e}");
            trace!("Failed to dial {peer}: {e}");
        }
    }

    //let chat_topic_hash =
    // gossipsub::IdentTopic::new(GOSSIPSUB_CHAT_TOPIC).hash();
    let chat_topic_hash = gossipsub::IdentTopic::new(opt.topic).hash();
    let file_topic_hash = gossipsub::IdentTopic::new(GOSSIPSUB_CHAT_FILE_TOPIC).hash();

    let mut tick = futures_timer::Delay::new(parse(opt.tick.clone()).expect("REASON"));

    loop {
        match select(swarm.next(), &mut tick).await {
            Either::Left((event, _)) => match event.unwrap() {
                SwarmEvent::NewListenAddr { address, .. } => {
                    if let Some(external_ip) = opt.external_address {
                        let external_address = address
                            .replace(0, |_| Some(external_ip.into()))
                            .expect("address.len > 1 and we always return `Some`");

                        swarm.add_external_address(external_address);
                    }

                    let p2p_address = address.with(Protocol::P2p(*swarm.local_peer_id()));
                    print!("Listening on {p2p_address}");
                }
                SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                    debug!("Connected to {peer_id}");
                }
                SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                    trace!("Failed to dial {peer_id:?}: {error}");
                }
                SwarmEvent::IncomingConnectionError { error, .. } => {
                    debug!("{:#}", anyhow::Error::from(error))
                }
                SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                    debug!("Connection to {peer_id} closed: {cause:?}");
                    swarm.behaviour_mut().kademlia.remove_peer(&peer_id);
                    debug!("Removed {peer_id} from the routing table (if it was in there).");
                }
                SwarmEvent::Behaviour(BehaviourEvent::Relay(e)) => {
                    trace!("{:?}", e);
                }
                SwarmEvent::Behaviour(BehaviourEvent::Gossipsub(
                    libp2p::gossipsub::Event::Message {
                        message_id: _,
                        propagation_source: _,
                        message,
                    },
                )) => {
                    //`source`, `data`, `sequence_number`, `topic`
                    //nanoseconds
                    //println!("message.sequence_number={:?}",message.sequence_number.unwrap() /
                    // 1000000000 );
                    if message.topic == chat_topic_hash {
                        debug!("message.topic={}", message.topic);
                        debug!("chat_topic_hash={}", chat_topic_hash);
                        trace!(
                            "message.sequence_number={:?}\n",
                            message.sequence_number.unwrap() / 1000000000
                        );
                        trace!(
                            "message.sequence_number={:?}\n",
                            message.sequence_number.unwrap() / 100000000
                        );
                        trace!(
                            "message.sequence_number={:?}\n",
                            message.sequence_number.unwrap() / 10000000
                        );
                        trace!(
                            "message.sequence_number={:?}\n",
                            message.sequence_number.unwrap() / 1000000
                        );
                        //info!(
                        //    "{}:{}:{:}:{}\n",
                        //    message.topic,
                        //    message.sequence_number.unwrap(),
                        //    message.source.unwrap(),
                        //    String::from_utf8(message.data).unwrap()
                        //);
                        print!(
                            "{}:{}:{:}:{}\n",
                            message.topic,
                            message.sequence_number.unwrap(),
                            message.source.unwrap(),
                            String::from_utf8(message.data).unwrap()
                        );
                        //debug!(
                        //    "{:}: {}",
                        //    message.source.unwrap().to_string(),
                        //    String::from_utf8(message.data).unwrap().to_string()
                        //);
                        continue;
                    } else {
                        println!(
                            "message.sequence_number={:?}",
                            message.sequence_number.unwrap() / 1000000000
                        );
                        info!("else.....");
                        info!(
                            "off topic:{:?}: {}",
                            message.source,
                            String::from_utf8(message.data.clone()).unwrap().to_string()
                        );
                    }

                    if message.topic == file_topic_hash {
                        let file_id = String::from_utf8(message.data).unwrap();
                        info!("Received file {} from {:?}", file_id, message.source);

                        let request_id = swarm.behaviour_mut().request_response.send_request(
                            &message.source.unwrap(),
                            FileRequest {
                                file_id: file_id.clone(),
                            },
                        );
                        info!(
                            "Requested file {} to {:?}: req_id:{:?}",
                            file_id, message.source, request_id
                        );
                        continue;
                    }

                    info!("Unexpected gossipsub topic hash: {:?}", message.topic);
                }
                SwarmEvent::Behaviour(BehaviourEvent::Gossipsub(
                    libp2p::gossipsub::Event::Subscribed { peer_id, topic },
                )) => {
                    debug!("{peer_id} subscribed to {topic}");
                }
                SwarmEvent::Behaviour(BehaviourEvent::Identify(e)) => {
                    debug!("BehaviourEvent::Identify {:?}", e);

                    if let identify::Event::Error { peer_id, error } = e {
                        match error {
                            libp2p::swarm::StreamUpgradeError::Timeout => {
                                // When a browser tab closes, we don't get a swarm event
                                // maybe there's a way to get this with TransportEvent
                                // but for now remove the peer from routing table if there's an
                                // Identify timeout
                                swarm.behaviour_mut().kademlia.remove_peer(&peer_id);
                                debug!(
                                    "Removed {peer_id} from the routing table (if it was in \
                                     there)."
                                );
                            }
                            _ => {
                                debug!("{error}");
                            }
                        }
                    } else if let identify::Event::Received {
                        peer_id,
                        info:
                            identify::Info {
                                listen_addrs,
                                protocols,
                                observed_addr,
                                ..
                            },
                    } = e
                    {
                        debug!("identify::Event::Received observed_addr: {}", observed_addr);

                        swarm.add_external_address(observed_addr);

                        // TODO: The following should no longer be necessary after https://github.com/libp2p/rust-libp2p/pull/4371.
                        if protocols.iter().any(|p| p == &KADEMLIA_PROTOCOL_NAME) {
                            for addr in listen_addrs {
                                debug!("identify::Event::Received listen addr: {}", addr);
                                // TODO (fixme): the below doesn't work because the address is still missing /webrtc/p2p even after https://github.com/libp2p/js-libp2p-webrtc/pull/121
                                // swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);

                                let webrtc_address = addr
                                    .with(Protocol::WebRTCDirect)
                                    .with(Protocol::P2p(peer_id));

                                swarm
                                    .behaviour_mut()
                                    .kademlia
                                    .add_address(&peer_id, webrtc_address.clone());
                                trace!("Added {webrtc_address} to the routing table.");
                            }
                        }
                    }
                }
                SwarmEvent::Behaviour(BehaviourEvent::Kademlia(e)) => {
                    debug!("Kademlia event: {:?}", e);
                }
                SwarmEvent::Behaviour(BehaviourEvent::RequestResponse(
                    request_response::Event::Message { message, .. },
                )) => match message {
                    request_response::Message::Request { request, .. } => {
                        //TODO: support ProtocolSupport::Full
                        debug!(
                            "umimplemented: request_response::Message::Request: {:?}",
                            request
                        );
                    }
                    request_response::Message::Response { response, .. } => {
                        debug!(
                            "request_response::Message::Response: size:{}",
                            response.file_body.len()
                        );
                        // TODO: store this file (in memory or disk) and
                        // provider it via Kademlia
                    }
                },
                SwarmEvent::Behaviour(BehaviourEvent::RequestResponse(
                    request_response::Event::OutboundFailure {
                        request_id, error, ..
                    },
                )) => {
                    error!(
                        "request_response::Event::OutboundFailure for request {:?}: {:?}",
                        request_id, error
                    );
                }
                event => {
                    debug!("Other type of event: {:?}", event);
                }
            },
            Either::Right(_) => {
                //TICK
                //
                tick = futures_timer::Delay::new(parse(opt.tick.clone()).expect("REASON"));

                //TODO format for nostr EVENT syndication
                trace!(
                    "{{ external_addresses: {:?}}}",
                    swarm.external_addresses().collect::<Vec<&Multiaddr>>()
                );

                if let Err(e) = swarm.behaviour_mut().kademlia.bootstrap() {
                    debug!("Failed to run Kademlia bootstrap: {e:?}");
                }

                //NOW
                //help: if used in a formatting string, curly braces are escaped with `{{` and
                // `}}`

                //debug!("{peer_id} subscribed to {topic}");
                let now = tokio::time::Instant::now();
                let message = format!("gnostr-chat ping: {:4}s", now.elapsed().as_secs_f64());

                //TODO
                //TODO format for nostr EVENT syndication
                // swarm.behaviour_mut().gosssip.publish(...
                if let Err(err) = swarm.behaviour_mut().gossipsub.publish(
                    gossipsub::IdentTopic::new(chat_topic_hash.to_string()),
                    message.as_bytes(),
                ) {
                    //error!("Failed to publish periodic message: {err}")
                    info!("Failed to publish periodic message: {err}")
                }
            } // END EITHER::Right
        }
    }
}

#[derive(NetworkBehaviour)]
struct Behaviour {
    gossipsub: gossipsub::Behaviour,
    identify: identify::Behaviour,
    kademlia: Kademlia<MemoryStore>,
    relay: relay::Behaviour,
    request_response: request_response::Behaviour<FileExchangeCodec>,
    connection_limits: memory_connection_limits::Behaviour,
}

fn create_swarm(
    local_key: identity::Keypair,
    certificate: Certificate,
    topic: String,
) -> Result<Swarm<Behaviour>> {
    let local_peer_id = PeerId::from(local_key.public());
    debug!("Local peer id: {local_peer_id}");

    // To content-address message, we can take the hash of message and use it as an
    // ID.
    let message_id_fn = |message: &gossipsub::Message| {
        let mut s = DefaultHasher::new();
        message.data.hash(&mut s);
        gossipsub::MessageId::from(s.finish().to_string())
    };

    // Set a custom gossipsub configuration
    let gossipsub_config = gossipsub::ConfigBuilder::default()
        .validation_mode(gossipsub::ValidationMode::Permissive) // This sets the kind of message validation. The default is Strict (enforce message signing)
        .message_id_fn(message_id_fn) // content-address messages. No two messages of the same content will be propagated.
        .mesh_outbound_min(1)
        .mesh_n_low(1)
        .flood_publish(true)
        .build()
        .expect("Valid config");

    // build a gossipsub network behaviour
    let mut gossipsub = gossipsub::Behaviour::new(
        gossipsub::MessageAuthenticity::Signed(local_key.clone()),
        gossipsub_config,
    )
    .expect("Correct configuration");

    // Create/subscribe Gossipsub topics
    gossipsub.subscribe(&gossipsub::IdentTopic::new(topic))?;
    gossipsub.subscribe(&gossipsub::IdentTopic::new(GOSSIPSUB_CHAT_FILE_TOPIC))?;

    let transport = {
        let webrtc = webrtc::tokio::Transport::new(local_key.clone(), certificate);
        let quic = quic::tokio::Transport::new(quic::Config::new(&local_key));

        let mapped = webrtc.or_transport(quic).map(|fut, _| match fut {
            Either::Right((local_peer_id, conn)) => (local_peer_id, StreamMuxerBox::new(conn)),
            Either::Left((local_peer_id, conn)) => (local_peer_id, StreamMuxerBox::new(conn)),
        });

        dns::TokioDnsConfig::system(mapped)?.boxed()
    };

    let identify_config = identify::Behaviour::new(
        identify::Config::new("/ipfs/0.1.0".into(), local_key.public())
            .with_interval(Duration::from_secs(60)), /* do this so we can get timeouts for
                                                      * dropped WebRTC connections */
    );

    // Create a Kademlia behaviour.
    let mut cfg = KademliaConfig::default();
    cfg.set_protocol_names(vec![KADEMLIA_PROTOCOL_NAME]);
    let store = MemoryStore::new(local_peer_id);
    let kad_behaviour = Kademlia::with_config(local_peer_id, store, cfg);

    let behaviour = Behaviour {
        gossipsub,
        identify: identify_config,
        kademlia: kad_behaviour,
        relay: relay::Behaviour::new(
            local_peer_id,
            relay::Config {
                max_reservations: usize::MAX,
                max_reservations_per_peer: 100,
                reservation_rate_limiters: Vec::default(),
                circuit_src_rate_limiters: Vec::default(),
                max_circuits: usize::MAX,
                max_circuits_per_peer: 100,
                ..Default::default()
            },
        ),
        request_response: request_response::Behaviour::new(
            // TODO: support ProtocolSupport::Full
            iter::once((FILE_EXCHANGE_PROTOCOL, ProtocolSupport::Outbound)),
            Default::default(),
        ),
        connection_limits: memory_connection_limits::Behaviour::with_max_percentage(0.9),
    };
    Ok(
        SwarmBuilder::with_tokio_executor(transport, behaviour, local_peer_id)
            .idle_connection_timeout(Duration::from_secs(60))
            .build(),
    )
}

async fn read_or_create_certificate(path: &Path) -> Result<Certificate> {
    if path.exists() {
        let pem = fs::read_to_string(&path).await?;

        debug!("Using existing certificate from {}", path.display());

        return Ok(Certificate::from_pem(&pem)?);
    }

    let cert = Certificate::generate(&mut rand::thread_rng())?;
    fs::write(&path, &cert.serialize_pem().as_bytes()).await?;

    debug!(
        "Generated new certificate and wrote it to {}",
        path.display()
    );

    Ok(cert)
}

async fn read_or_create_identity(path: &Path) -> Result<identity::Keypair> {
    if path.exists() {
        let bytes = fs::read(&path).await?;

        debug!("Using existing identity from {}", path.display());

        return Ok(identity::Keypair::from_protobuf_encoding(&bytes)?); // This only works for ed25519 but that is what we are using.
    }

    let identity = identity::Keypair::generate_ed25519();

    fs::write(&path, &identity.to_protobuf_encoding()?).await?;

    info!("Generated new identity and wrote it to {}", path.display());

    Ok(identity)
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}
pub fn greeting(name: &str) -> String {
    format!("Hello {}!", name)
}
pub fn greeting_hello(name: &str) -> String {
    //name not used!
    String::from("Hello!")
}

#[cfg(test)]
mod tests {
    #[cfg(test)]
    use std::{println as info, println as warn};

    #[cfg(not(test))]
    use log::{info, warn}; // Use log crate when building application

    use super::*; // Workaround to use prinltn! for logs.

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
        info!("{:?}", assert_eq!(result, 4));
        warn!("{:?}", assert_eq!(result, 4));
    }
    #[test]
    fn greeting_contains_name() {
        let result1 = greeting("Carol");
        assert!(result1.contains("Carol"));
        let result2 = greeting_hello("Carol");
        assert!(
            result2.contains("Hello"),
            "Greeting did not contain {}, value was `{}`",
            result1,
            result2
        );
    }
    #[test]
    #[should_panic]
    fn greeting_contains_name_panic() {
        let result1 = greeting("Carol");
        assert!(result1.contains("Carol"));
        let result2 = greeting_hello("Carol");
        assert!(
            result2.contains("Carol"),
            "Greeting did not contain {}, value was `{}`",
            result1,
            result2
        );
    }
}
