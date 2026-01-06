use futures::stream::StreamExt;
use libp2p::{
    gossipsub, identify, kad, mdns, noise, ping,
    request_response::{self, ProtocolSupport},
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, StreamProtocol,
};
use std::error::Error;
use std::time::Duration;
use tokio::{io, select};
use tracing::{debug, warn};

use std::collections::HashMap;
use parking_lot::Mutex;
use std::sync::Arc;
use terminal_size::{terminal_size, Height, Width};
use textwrap::{self, Options};

use ureq::Agent;

use crate::p2p::kvs::{FileRequest, FileResponse};
use crate::p2p::chat::msg::{Msg, MsgKind};

// Struct to manage message reassembly
pub struct MessageReassembler {
    // message_id -> (total_chunks, received_chunks, chunks)
    buffer: Mutex<HashMap<String, (usize, usize, Vec<Option<Msg>>)>>,
}

impl MessageReassembler {
    pub fn new() -> Self {
        MessageReassembler {
            buffer: Mutex::new(HashMap::new()),
        }
    }

    /// Adds a message chunk to the buffer and attempts reassembly.
    /// Returns Some(complete_message) if reassembly is successful, None otherwise.
    pub fn add_chunk_and_reassemble(&self, msg_chunk: Msg) -> Option<Msg> {
        if msg_chunk.message_id.is_none() || msg_chunk.sequence_num.is_none() || msg_chunk.total_chunks.is_none() {
            // Not a multi-part message, or missing sequencing info
            debug!("Received non-multi-part message or message with missing sequencing info.");
            return None;
        }

        let message_id = msg_chunk.message_id.clone().unwrap(); // Clone here
        let sequence_num = msg_chunk.sequence_num.unwrap();
        let total_chunks = msg_chunk.total_chunks.unwrap();

        debug!("AddChunk: Received chunk for message_id: {}, sequence_num: {}/{}, content_len: {}",
               message_id, sequence_num + 1, total_chunks, msg_chunk.content[0].len());

        let mut buffer_guard = self.buffer.lock();

        let (buffered_total_chunks, ref mut received_count, ref mut chunks) = buffer_guard
            .entry(message_id.clone())
            .or_insert_with(|| {
                debug!("AddChunk: Initializing buffer for message_id: {} with total_chunks: {}", message_id, total_chunks);
                (total_chunks, 0, vec![None; total_chunks])
            });

        // Ensure consistency if a message_id is reused with different total_chunks
        // Or if an invalid chunk is received for an already existing message_id
        if *buffered_total_chunks != total_chunks {
            debug!("AddChunk: Inconsistent total_chunks for message_id {}. Expected {}, got {}", message_id, *buffered_total_chunks, total_chunks);
            buffer_guard.remove(&message_id);
            return None;
        }

        if sequence_num < total_chunks {
            if chunks[sequence_num].is_none() {
                chunks[sequence_num] = Some(msg_chunk.clone()); // Clone msg_chunk here
                *received_count += 1;
                debug!("AddChunk: Chunk {} received for message_id: {}. Total received: {}/{}", sequence_num, message_id, *received_count, total_chunks);
            } else {
                debug!("AddChunk: Duplicate chunk received for message_id {} sequence {}", message_id, sequence_num);
            }
        } else {
            debug!("AddChunk: Invalid sequence_num {} for message_id {} (total_chunks {})", sequence_num, message_id, total_chunks);
            return None;
        }

        if *received_count == total_chunks {
            debug!("AddChunk: All chunks received for message_id: {}. Attempting reassembly.", message_id);
            // All chunks received, reassemble
            let mut full_content = String::new();
            let mut reassembled_msg = Msg::default();

            for (i, chunk_option) in chunks.iter().enumerate() {
                if let Some(chunk) = chunk_option {
                    if i == 0 {
                        // Use the first chunk's metadata for the reassembled message
                        reassembled_msg.from = chunk.from.clone();
                        reassembled_msg.kind = chunk.kind;
                        reassembled_msg.commit_id = chunk.commit_id;
                        reassembled_msg.nostr_event = chunk.nostr_event.clone();
                        // Reset sequencing info as it's now a single complete message
                        reassembled_msg.message_id = None;
                        reassembled_msg.sequence_num = None;
                        reassembled_msg.total_chunks = None;
                    }
                    full_content.push_str(&chunk.content[0]);
                } else {
                    // This should not happen if received_count == total_chunks
                    debug!("AddChunk: Critical error - Missing chunk for message_id {} at sequence {} during reassembly despite received_count matching total_chunks.", message_id, i);
                    buffer_guard.remove(&message_id); // Clear incomplete message
                    return None;
                }
            }
            reassembled_msg.content = vec![full_content];
            buffer_guard.remove(&message_id);
            debug!("AddChunk: Successfully reassembled message for message_id: {}.", message_id);
            Some(reassembled_msg)
        } else {
            None
        }
    }
}

#[derive(NetworkBehaviour)]
pub struct MyBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
    pub identify: identify::Behaviour,
    pub kademlia: kad::Behaviour<kad::store::MemoryStore>,
    pub ping: ping::Behaviour,
    pub request_response: request_response::cbor::Behaviour<FileRequest, FileResponse>,
}

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

///// fetch_data_async
//async fn fetch_data_async<T>(url: String) -> Result<ureq::Response<T>, ureq::Error> {
//    task::spawn_blocking(move || {
//        let response = ureq::get(&url).call();
//        response
//    })
//    .await
//    .unwrap() // Handle potential join errors
//}

/// evt_loop
pub async fn evt_loop(
    mut send: tokio::sync::mpsc::Receiver<crate::queue::InternalEvent>,
    recv: tokio::sync::mpsc::Sender<crate::queue::InternalEvent>,
    topic: gossipsub::IdentTopic,
) -> Result<(), Box<dyn Error>> {
    let reassembler = Arc::new(MessageReassembler::new()); // Create reassembler here

    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_quic()
        .with_behaviour(|key| {
            // NOTE: To content-address message,
            // we can take the hash of message
            // and use it as an ID.
            // This is used to deduplicate messages.
            //
            let _message_id_fn = |message: &gossipsub::Message| {
                use std::hash::DefaultHasher;
                use std::hash::Hash;
                use std::hash::Hasher;
                let mut s = DefaultHasher::new();
                message.data.hash(&mut s);
                gossipsub::MessageId::from(s.finish().to_string())
            };

            // Set a custom gossipsub configuration
            let gossipsub_config = gossipsub::ConfigBuilder::default()
                .heartbeat_interval(Duration::from_secs(10))
                // This is set to aid debugging by not cluttering the log space
                .validation_mode(gossipsub::ValidationMode::Strict)
                // This sets the kind of message validation.
                // The default is Strict (enforce message signing)
                // .message_id_fn(message_id_fn)
                // content-address messages.
                // No two messages of the same content will be propagated.
                .build()
                .map_err(|msg| io::Error::other(msg))?;
            // Temporary hack because `build` does not return a proper `std::error::Error`.

            // build a gossipsub network behaviour
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

            let request_response = request_response::cbor::Behaviour::new(
                [(
                    StreamProtocol::new("/file-exchange/1"),
                    ProtocolSupport::Full,
                )],
                request_response::Config::default(),
            );

            Ok(crate::p2p::chat::p2p::MyBehaviour {
                gossipsub,
                mdns,
                identify,
                kademlia,
                ping,
                request_response,
            })
        })?
        .with_swarm_config(|c: libp2p::swarm::Config| c.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();

    // subscribes to our topic
    swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

    // Listen on all interfaces and whatever port the OS assigns
    swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    debug!("Enter messages via STDIN and they will be sent to connected peers using Gossipsub");

    // Helper function for text wrapping
    fn apply_text_wrapping(msg: &mut Msg, terminal_width: usize) {
        if msg.content.is_empty() {
            return;
        }

        match msg.kind {
            MsgKind::Chat | MsgKind::OneShot => { // OneShot without --diff will be handled here
                let wrapped_content = textwrap::fill(&msg.content[0], Options::new(terminal_width));
                msg.content = wrapped_content.lines().map(String::from).collect();
            }
            MsgKind::GitDiff => {
                let mut wrapped_lines = Vec::new();
                for line in msg.content[0].lines() {
                    let (prefix, content) = if line.starts_with('+') {
                        ("+", &line[1..])
                    } else if line.starts_with('-') {
                        ("-", &line[1..])
                    } else if line.starts_with(' ') { // context line
                        (" ", &line[1..])
                    } else if line.starts_with("diff --git") || line.starts_with("index") || line.starts_with("--- a/") || line.starts_with("+++ b/") {
                        // Header lines, no wrapping for now, just add as is
                        wrapped_lines.push(line.to_string());
                        continue;
                    } else if line.starts_with("@@") {
                        ("@@", &line[2..]) // Handle @@ line
                    }
                    else {
                        // No specific diff prefix, treat as regular content
                        ("", line)
                    };

                    // Ensure the width calculation accounts for the prefix length
                    let wrap_width = if terminal_width > prefix.len() {
                        terminal_width - prefix.len()
                    } else {
                        terminal_width // Fallback if prefix is too long
                    };

                    let wrapped_segments = textwrap::fill(content, Options::new(wrap_width));
                    for (i, segment) in wrapped_segments.lines().enumerate() {
                        if i == 0 { // First segment gets the prefix
                            wrapped_lines.push(format!("{}{}", prefix, segment));
                        } else { // Subsequent segments are indented if prefix was not empty
                            wrapped_lines.push(format!("{:indent$}{}", "", segment, indent = prefix.len()));
                        }
                    }
                }
                msg.content = wrapped_lines;
            }
            _ => { /* No wrapping for other message kinds */ }
        }
    }

    // Kick it off
    loop {
        select! {
            Some(event) = send.recv() => {
                if let crate::queue::InternalEvent::ChatMessage(m) = event {
                    if let Err(e) = swarm
                        .behaviour_mut().gossipsub
                        .publish(topic.clone(), serde_json::to_vec(&m)?) {
                        debug!("Publish error: {e:?}");
                        let m = crate::p2p::chat::msg::Msg::default().set_content(format!("publish error: {e:?}"), 0).set_kind(crate::p2p::chat::msg::MsgKind::System);
                        recv.send(crate::queue::InternalEvent::ShowErrorMsg(m.to_string())).await?;
                    }
                }
            }
            event = swarm.select_next_some() => match event {
                SwarmEvent::Behaviour(crate::p2p::chat::p2p::MyBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                    for (peer_id, _multiaddr) in list {
                        debug!("mDNS discovered a new peer: {peer_id}");
                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                        let m = crate::p2p::chat::msg::Msg::default().set_content(format!("discovered new peer: {peer_id}"), 0).set_kind(crate::p2p::chat::msg::MsgKind::System);
                        recv.send(crate::queue::InternalEvent::ShowInfoMsg(m.to_string())).await?;
                    }
                },
                SwarmEvent::Behaviour(crate::p2p::chat::p2p::MyBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                    for (peer_id, _multiaddr) in list {
                        debug!("mDNS discover peer has expired: {peer_id}");
                        swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                        let m = crate::p2p::chat::msg::Msg::default().set_content(format!("peer expired: {peer_id}"), 0).set_kind(crate::p2p::chat::msg::MsgKind::System);
                        recv.send(crate::queue::InternalEvent::ShowInfoMsg(m.to_string())).await?;
                    }
                },
                SwarmEvent::Behaviour(crate::p2p::chat::p2p::MyBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                    propagation_source: peer_id,
                    message_id: id,
                    message,
                })) => {
                    debug!(
                        "Got message: '{}' with id: {id} from peer: {peer_id}",
                        String::from_utf8_lossy(&message.data),
                    );
                    match serde_json::from_slice::<Msg>(&message.data) {
                        Ok(msg) => {
                            if msg.message_id.is_some() && msg.sequence_num.is_some() && msg.total_chunks.is_some() {
                                                                if let Some(mut reassembled_msg) = reassembler.add_chunk_and_reassemble(msg) {
                                                                    let terminal_width = terminal_size().map(|(Width(w), _)| w as usize).unwrap_or(80);
                                                                    apply_text_wrapping(&mut reassembled_msg, terminal_width);
                                                                    recv.send(crate::queue::InternalEvent::ChatMessage(reassembled_msg)).await?;
                                                                }
                            } else {
                                // It's a single-part message, send directly
                                let mut processed_msg = msg;
                                let terminal_width = terminal_size().map(|(Width(w), _)| w as usize).unwrap_or(80);
                                apply_text_wrapping(&mut processed_msg, terminal_width);
                                recv.send(crate::queue::InternalEvent::ChatMessage(processed_msg)).await?;
                            }
                        },
                        Err(e) => {
                            debug!("Error deserializing message: {e:?}");
                            let m = Msg::default().set_content(format!("Error deserializing message: {e:?}"), 0).set_kind(MsgKind::System);
                            recv.send(crate::queue::InternalEvent::ShowErrorMsg(m.to_string())).await?;
                        }
                    }
                },
                SwarmEvent::NewListenAddr { address, .. } => {
                    debug!("Local node is listening on {address}");
                    let m = crate::p2p::chat::msg::Msg::default().set_content(format!("Local node is listening on {address}"), 0).set_kind(crate::p2p::chat::msg::MsgKind::System);
                    recv.send(crate::queue::InternalEvent::ShowInfoMsg(m.to_string())).await?;
                }
                _ => {}
            }
        }
    }
}
