//! Recursive packet framing and parity helpers for the `perfect_ip` protocol.
//!
//! `perfect_ip` is a repair-oriented tree protocol. It does not try to be a
//! general transport; instead it turns a byte buffer into a deterministic tree
//! of MTU-safe data slices and parity slices that can be reconstructed when a
//! sibling packet is missing.
//!
//! ## Packet model
//!
//! - [`process_slice`] recursively splits payloads until each emitted leaf fits
//!   within the packet budget.
//! - Internal nodes emit a left child, right child, and parity frame.
//! - Packet ids form a stable recursive path such as `ROOT.1.0.P`.
//! - [`packetize`] finalizes the batch by filling `total_packets` into every
//!   packet header.
//!
//! ## Wire model
//!
//! On the wire, packet slices are exchanged as JSON-encoded [`ProtocolSlice`]
//! values over libp2p request/response. The repair protocol id is
//! `/fractal/repair/1.0.0/json`, and the behaviour is exposed through
//! [`FractalBehaviour`] and [`run_fractal_engine`].
//!
//! ## Repair model
//!
//! The parity scheme is XOR-based. Given one sibling payload and the matching
//! parity frame, [`recover_missing_data`] and [`recover_missing_slice`] can
//! rebuild the missing sibling payload. [`IntegrityManager`] tracks the
//! manifest, records received slices, persists the partial state to disk, and
//! verifies that parity frames still match their children.
//!
//! The layout intentionally mirrors the packet-tree sketch in the
//! `perfect_ip.rs` gist.

use std::error::Error;
use std::time::Duration;

use futures::StreamExt;
use libp2p::{
    request_response,
    swarm::{NetworkBehaviour, Swarm, SwarmEvent},
    StreamProtocol,
};

pub use gnostr_types::{
    calculate_parity, generate_manifest, list_directory, packetize, process_slice,
    reconstruct_directory, reconstruct_payload, recover_missing_data, recover_missing_slice,
    run_directory_walk, sha256_hex, summarize_packets, DirectoryWalk, Header, IntegrityManager,
    MTU_PAYLOAD, PacketBatch, ProtocolSlice,
};

/// libp2p repair behaviour for exchanging packet slices.
///
/// This is a small request/response behaviour that accepts one
/// [`ProtocolSlice`] and returns one [`ProtocolSlice`]. It uses JSON on the
/// wire so the packets can be inspected or proxied as text if needed.
#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "FractalBehaviourEvent")]
pub struct FractalBehaviour {
    /// JSON request/response channel for packet slice repair.
    pub repair_rpc: request_response::json::Behaviour<ProtocolSlice, ProtocolSlice>,
}

/// Behaviour events emitted by the repair RPC layer.
///
/// The enum stays small because the behaviour exposes one request/response
/// flow.
#[derive(Debug)]
pub enum FractalBehaviourEvent {
    /// A JSON request/response protocol event.
    RepairRpc(request_response::Event<ProtocolSlice, ProtocolSlice>),
}

impl From<request_response::Event<ProtocolSlice, ProtocolSlice>> for FractalBehaviourEvent {
    fn from(event: request_response::Event<ProtocolSlice, ProtocolSlice>) -> Self {
        Self::RepairRpc(event)
    }
}

/// Build the repair swarm used to exchange packet slices with peers.
///
/// The swarm uses the current identity, Tokio, and QUIC transport so the demo
/// can listen on a simple localhost multiaddr without extra transport setup.
pub async fn build_fractal_swarm(
    local_key: libp2p::identity::Keypair,
) -> Result<Swarm<FractalBehaviour>, Box<dyn Error + Send + Sync>> {
    #[cfg(target_os = "tvos")]
    let swarm = libp2p::SwarmBuilder::with_existing_identity(local_key)
        .with_tokio()
        .with_behaviour(|_| {
            Ok::<_, Box<dyn Error + Send + Sync>>(FractalBehaviour {
                repair_rpc: request_response::json::Behaviour::new(
                    [(
                        StreamProtocol::new("/fractal/repair/1.0.0/json"),
                        request_response::ProtocolSupport::Full,
                    )],
                    request_response::Config::default(),
                ),
            })
        })?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();

    #[cfg(not(target_os = "tvos"))]
    let swarm = libp2p::SwarmBuilder::with_existing_identity(local_key)
        .with_tokio()
        .with_quic()
        .with_behaviour(|_| {
            Ok::<_, Box<dyn Error + Send + Sync>>(FractalBehaviour {
                repair_rpc: request_response::json::Behaviour::new(
                    [(
                        StreamProtocol::new("/fractal/repair/1.0.0/json"),
                        request_response::ProtocolSupport::Full,
                    )],
                    request_response::Config::default(),
                ),
            })
        })?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();

    Ok(swarm)
}

/// Start the repair swarm and forward incoming repair requests to the packet map.
///
/// This is the runtime repair loop for the protocol. It listens for
/// `ProtocolSlice` requests over JSON, looks up the requested id in
/// [`IntegrityManager::received_slices`], and replies with the stored slice or
/// an empty placeholder when the packet is absent.
pub async fn run_fractal_engine(
    local_key: libp2p::identity::Keypair,
    listen_address: libp2p::Multiaddr,
    manager: IntegrityManager,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut swarm = build_fractal_swarm(local_key).await?;
    swarm.listen_on(listen_address)?;

    loop {
        match swarm.select_next_some().await {
            SwarmEvent::Behaviour(FractalBehaviourEvent::RepairRpc(event)) => {
                if let request_response::Event::Message { message, .. } = event {
                    if let request_response::Message::Request {
                        request, channel, ..
                    } = message
                    {
                        let response = manager
                            .received_slices
                            .get(&request.id)
                            .cloned()
                            .unwrap_or_else(|| ProtocolSlice {
                                id: request.id.clone(),
                                header: request.header.clone(),
                                data: Vec::new(),
                                is_parity: false,
                            });
                        let _ = swarm.behaviour_mut().repair_rpc.send_response(channel, response);
                    }
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::Path;
    use std::process::Command;
    use std::time::Duration;

    use libp2p::{request_response, swarm::SwarmEvent, Multiaddr};
    use tempfile::TempDir;
    use tokio::time::timeout;

    use super::*;
    use crate::keypair_from_seed;

    #[tokio::test]
    async fn build_fractal_swarm_accepts_quic_listen_address() {
        let keypair = keypair_from_seed(Some(
            gnostr_asyncgit::default_gnostr_private_key_hex(),
        ));
        let mut swarm = build_fractal_swarm(keypair).await.expect("swarm");
        let addr: Multiaddr = "/ip4/127.0.0.1/udp/0/quic-v1"
            .parse()
            .expect("quic multiaddr");

        swarm.listen_on(addr).expect("quic listen");
    }

    #[tokio::test]
    #[ignore = "networked p2p sync test: run with --ignored --nocapture"]
    async fn fractal_p2p_sync_reconstructs_git_repo() {
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../web/crlf.git")
            .canonicalize()
            .expect("locate web/crlf.git fixture");
        println!("fixture: {}", fixture.display());

        let walk = run_directory_walk(&fixture, 3, false).expect("packetize fixture");
        let all_slices: Vec<ProtocolSlice> =
            walk.files.values().flat_map(|packets| packets.clone()).collect();
        let all_ids: Vec<String> = all_slices.iter().map(|s| s.id.clone()).collect();
        println!("total slices to sync: {}", all_slices.len());

        let mut sender_manager = IntegrityManager::new(all_ids.clone());
        for slice in &all_slices {
            sender_manager.record_slice(slice.clone());
        }

        // Sender: listen and answer repair requests.
        let (addr_tx, addr_rx) = tokio::sync::oneshot::channel();
        let sender_handle = tokio::spawn(async move {
            let keypair = keypair_from_seed(Some("sender".to_string()));
            let mut swarm = build_fractal_swarm(keypair).await.expect("sender swarm");
            swarm
                .listen_on("/ip4/127.0.0.1/udp/0/quic-v1".parse().unwrap())
                .unwrap();
            println!("[sender] listening");

            let mut addr_tx = Some(addr_tx);
            let mut requests_served = 0usize;
            loop {
                match swarm.select_next_some().await {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("[sender] new listen addr: {address}");
                        if let Some(tx) = addr_tx.take() {
                            let _ = tx.send(address);
                        }
                    }
                    SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        println!("[sender] connection established with {peer_id}");
                    }
                    SwarmEvent::Behaviour(FractalBehaviourEvent::RepairRpc(event)) => {
                        if let request_response::Event::Message { message, .. } = event {
                            if let request_response::Message::Request {
                                request, channel, ..
                            } = message
                            {
                                requests_served += 1;
                                println!(
                                    "[sender] request {requests_served}: {} -> lookup",
                                    request.id
                                );
                                let response = sender_manager
                                    .received_slices
                                    .get(&request.id)
                                    .cloned()
                                    .unwrap_or_else(|| ProtocolSlice {
                                        id: request.id.clone(),
                                        header: request.header.clone(),
                                        data: Vec::new(),
                                        is_parity: false,
                                    });
                                println!(
                                    "[sender] response: {} ({} bytes)",
                                    response.id,
                                    response.data.len()
                                );
                                let _ = swarm
                                    .behaviour_mut()
                                    .repair_rpc
                                    .send_response(channel, response);
                            }
                        }
                    }
                    other => {
                        println!("[sender] event: {other:?}");
                    }
                }
            }
        });

        let sender_addr: Multiaddr = timeout(Duration::from_secs(5), addr_rx)
            .await
            .expect("sender address timeout")
            .expect("sender address channel");
        println!("[receiver] sender addr: {sender_addr}");

        // Receiver: dial sender, request every slice, and collect responses.
        let keypair = keypair_from_seed(Some("receiver".to_string()));
        let mut swarm = build_fractal_swarm(keypair).await.expect("receiver swarm");
        swarm
            .listen_on("/ip4/127.0.0.1/udp/0/quic-v1".parse().unwrap())
            .unwrap();
        swarm.dial(sender_addr).unwrap();
        println!("[receiver] dialing sender");

        let mut received_slices = HashMap::new();

        timeout(Duration::from_secs(30), async {
            loop {
                match swarm.select_next_some().await {
                    SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        println!("[receiver] connected to {peer_id}");
                        tokio::time::sleep(Duration::from_millis(100)).await;
                        println!("[receiver] sending {} requests", all_slices.len());
                        for slice in &all_slices {
                            let request = ProtocolSlice {
                                id: slice.id.clone(),
                                header: slice.header.clone(),
                                data: Vec::new(),
                                is_parity: false,
                            };
                            swarm
                                .behaviour_mut()
                                .repair_rpc
                                .send_request(&peer_id, request);
                        }
                        println!("[receiver] all requests sent");
                    }
                    SwarmEvent::Behaviour(FractalBehaviourEvent::RepairRpc(event)) => {
                        if let request_response::Event::Message { message, .. } = event {
                            if let request_response::Message::Response { response, .. } = message {
                                let prev = received_slices.len();
                                let id = response.id.clone();
                                let len = response.data.len();
                                received_slices.insert(id.clone(), response);
                                if received_slices.len() > prev {
                                    println!(
                                        "[receiver] response {}/{}: {} ({} bytes)",
                                        received_slices.len(),
                                        all_slices.len(),
                                        id,
                                        len
                                    );
                                }
                                if received_slices.len() == all_slices.len() {
                                    println!("[receiver] all responses received");
                                    break;
                                }
                            }
                        }
                    }
                    other => {
                        println!("[receiver] event: {other:?}");
                    }
                }
            }
        })
        .await
        .expect("p2p sync timed out");

        println!("[test] aborting sender task");
        sender_handle.abort();

        // Verify every expected slice was received.
        for expected in &all_slices {
            assert!(
                received_slices.contains_key(&expected.id),
                "missing slice {}",
                expected.id
            );
        }

        // Reconstruct the directory tree from the received slices.
        let temp = TempDir::new().expect("temp dir");
        let reconstructed = temp.path().join("gnostr_web_crlf");
        let clone = temp.path().join("gnostr_web_crlf_repo");

        let mut files = std::collections::BTreeMap::new();
        for (rel_path, original_packets) in &walk.files {
            let mut packets = Vec::new();
            for original in original_packets {
                let received = received_slices
                    .get(&original.id)
                    .unwrap_or_else(|| panic!("missing slice {}", original.id));
                packets.push(received.clone());
            }
            files.insert(rel_path.clone(), packets);
        }

        let receiver_walk = DirectoryWalk {
            files,
            empty_dirs: walk.empty_dirs.clone(),
        };
        reconstruct_directory(receiver_walk, &reconstructed).expect("reconstruct directory");

        // Verify the reconstructed repo can be cloned.
        let status = Command::new("git")
            .arg("clone")
            .arg(&reconstructed)
            .arg(&clone)
            .status()
            .expect("spawn git clone");
        assert!(status.success(), "git clone from reconstructed directory failed");
        assert!(clone.join(".git").exists(), "cloned repo missing .git");

        let reconstructed_head = git_output(&reconstructed, &["rev-parse", "HEAD"]);
        let clone_head = git_output(&clone, &["rev-parse", "HEAD"]);
        assert_eq!(reconstructed_head, clone_head, "clone HEAD mismatch");

        let status_output = git_output(&clone, &["status", "--porcelain"]);
        assert!(
            status_output.is_empty(),
            "cloned working tree is not clean:\n{status_output}"
        );
    }

    #[tokio::test]
    #[ignore = "networked p2p bidirectional sync test: run with --ignored --nocapture"]
    async fn fractal_p2p_sync_modify_and_send_back() {
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../web/crlf.git")
            .canonicalize()
            .expect("locate web/crlf.git fixture");
        println!("fixture: {}", fixture.display());

        // Phase 1: sender -> receiver (initial sync).
        let walk = run_directory_walk(&fixture, 3, false).expect("packetize fixture");
        let all_slices: Vec<ProtocolSlice> =
            walk.files.values().flat_map(|packets| packets.clone()).collect();
        let all_ids: Vec<String> = all_slices.iter().map(|s| s.id.clone()).collect();

        let mut sender_manager = IntegrityManager::new(all_ids.clone());
        for slice in &all_slices {
            sender_manager.record_slice(slice.clone());
        }

        let (sender_handle, sender_addr) = spawn_responder(sender_manager, "sender").await;
        let received_slices = request_all_slices(sender_addr, "receiver1", all_ids.clone()).await;

        println!("[test] phase 1 complete: {} slices received", received_slices.len());
        sender_handle.abort();

        let temp = TempDir::new().expect("temp dir");
        let receiver_bare = temp.path().join("receiver_bare");
        let receiver_working = temp.path().join("receiver_working");

        let mut files = std::collections::BTreeMap::new();
        for (rel_path, original_packets) in &walk.files {
            let mut packets = Vec::new();
            for original in original_packets {
                let received = received_slices
                    .get(&original.id)
                    .unwrap_or_else(|| panic!("missing slice {}", original.id));
                packets.push(received.clone());
            }
            files.insert(rel_path.clone(), packets);
        }
        reconstruct_directory(
            DirectoryWalk {
                files,
                empty_dirs: walk.empty_dirs.clone(),
            },
            &receiver_bare,
        )
        .expect("reconstruct receiver bare repo");

        // Clone the reconstructed bare repo into a working copy.
        let clone_status = Command::new("git")
            .arg("clone")
            .arg(&receiver_bare)
            .arg(&receiver_working)
            .status()
            .expect("spawn git clone");
        assert!(clone_status.success(), "initial git clone failed");

        // Phase 2: receiver modifies the repo.
        git_output(&receiver_working, &["config", "user.email", "test@example.com"]);
        git_output(&receiver_working, &["config", "user.name", "Test User"]);

        let new_file = receiver_working.join("NEW_FILE.md");
        std::fs::write(&new_file, "# Added by receiver\n").expect("write new file");
        println!("[test] created {}", new_file.display());

        git_output(&receiver_working, &["add", "."]);
        git_output(&receiver_working, &["commit", "-m", "receiver update"]);
        println!("[test] receiver committed changes");

        // Phase 3: receiver -> sender (send modified repo back).
        let receiver_working_git = receiver_working.join(".git");
        let modified_walk = run_directory_walk(&receiver_working_git, 3, false).expect("packetize modified .git");
        let modified_slices: Vec<ProtocolSlice> = modified_walk
            .files
            .values()
            .flat_map(|packets| packets.clone())
            .collect();
        let modified_ids: Vec<String> = modified_slices.iter().map(|s| s.id.clone()).collect();

        let mut receiver_manager = IntegrityManager::new(modified_ids.clone());
        for slice in &modified_slices {
            receiver_manager.record_slice(slice.clone());
        }

        let (receiver_handle, receiver_addr) = spawn_responder(receiver_manager, "receiver2").await;
        let sender_received_slices =
            request_all_slices(receiver_addr, "sender2", modified_ids.clone()).await;

        println!(
            "[test] phase 3 complete: {} slices received by sender",
            sender_received_slices.len()
        );
        receiver_handle.abort();

        // Phase 4: sender reconstructs modified repo and shows diff.
        let sender_modified_repo = temp.path().join("sender_modified_repo");
        let mut modified_files = std::collections::BTreeMap::new();
        for (rel_path, original_packets) in &modified_walk.files {
            let mut packets = Vec::new();
            for original in original_packets {
                let received = sender_received_slices
                    .get(&original.id)
                    .unwrap_or_else(|| panic!("missing slice {}", original.id));
                packets.push(received.clone());
            }
            modified_files.insert(rel_path.clone(), packets);
        }
        reconstruct_directory(
            DirectoryWalk {
                files: modified_files,
                empty_dirs: modified_walk.empty_dirs.clone(),
            },
            &sender_modified_repo,
        )
        .expect("reconstruct sender modified repo");

        println!("\n=== modified repo log ===");
        println!("{}", git_output(&sender_modified_repo, &["log", "--oneline"]));

        println!("\n=== modified repo diff (HEAD~1..HEAD) ===");
        println!("{}", git_output(&sender_modified_repo, &["diff", "HEAD~1..HEAD"]));

        println!("\n=== modified repo show ===");
        println!("{}", git_output(&sender_modified_repo, &["show", "--stat"]));

        // Assertions.
        let original_commit_count: usize = git_output(&fixture, &["rev-list", "--count", "HEAD"])
            .parse()
            .expect("parse original commit count");
        let commit_count: usize = git_output(&sender_modified_repo, &["rev-list", "--count", "HEAD"])
            .parse()
            .expect("parse commit count");
        assert_eq!(
            commit_count,
            original_commit_count + 1,
            "expected one new commit in modified repo"
        );

        let diff = git_output(&sender_modified_repo, &["diff", "HEAD~1..HEAD"]);
        assert!(diff.contains("NEW_FILE.md"), "diff should mention NEW_FILE.md");
        assert!(
            diff.contains("# Added by receiver"),
            "diff should contain added content"
        );
    }

    async fn spawn_responder(
        manager: IntegrityManager,
        seed: &str,
    ) -> (tokio::task::JoinHandle<()>, Multiaddr) {
        let (addr_tx, addr_rx) = tokio::sync::oneshot::channel();
        let handle = tokio::spawn({
            let seed = seed.to_string();
            async move {
                let keypair = keypair_from_seed(Some(seed.clone()));
                let mut swarm = build_fractal_swarm(keypair).await.expect("responder swarm");
                swarm
                    .listen_on("/ip4/127.0.0.1/udp/0/quic-v1".parse().unwrap())
                    .unwrap();
                println!("[responder {seed}] listening");

                let mut addr_tx = Some(addr_tx);
                let mut requests_served = 0usize;
                loop {
                    match swarm.select_next_some().await {
                        SwarmEvent::NewListenAddr { address, .. } => {
                            println!("[responder {seed}] addr: {address}");
                            if let Some(tx) = addr_tx.take() {
                                let _ = tx.send(address);
                            }
                        }
                        SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                            println!("[responder {seed}] connected to {peer_id}");
                        }
                        SwarmEvent::Behaviour(FractalBehaviourEvent::RepairRpc(event)) => {
                            if let request_response::Event::Message { message, .. } = event {
                                if let request_response::Message::Request {
                                    request, channel, ..
                                } = message
                                {
                                    requests_served += 1;
                                    println!(
                                        "[responder {seed}] request {requests_served}: {}",
                                        request.id
                                    );
                                    let response = manager
                                        .received_slices
                                        .get(&request.id)
                                        .cloned()
                                        .unwrap_or_else(|| ProtocolSlice {
                                            id: request.id.clone(),
                                            header: request.header.clone(),
                                            data: Vec::new(),
                                            is_parity: false,
                                        });
                                    println!(
                                        "[responder {seed}] response: {} ({} bytes)",
                                        response.id,
                                        response.data.len()
                                    );
                                    let _ = swarm
                                        .behaviour_mut()
                                        .repair_rpc
                                        .send_response(channel, response);
                                }
                            }
                        }
                        other => {
                            println!("[responder {seed}] event: {other:?}");
                        }
                    }
                }
            }
        });

        let addr = timeout(Duration::from_secs(5), addr_rx)
            .await
            .expect("responder address timeout")
            .expect("responder address channel");
        (handle, addr)
    }

    async fn request_all_slices(
        responder_addr: Multiaddr,
        seed: &str,
        slice_ids: Vec<String>,
    ) -> HashMap<String, ProtocolSlice> {
        let keypair = keypair_from_seed(Some(seed.to_string()));
        let mut swarm = build_fractal_swarm(keypair).await.expect("requester swarm");
        swarm
            .listen_on("/ip4/127.0.0.1/udp/0/quic-v1".parse().unwrap())
            .unwrap();
        swarm.dial(responder_addr).unwrap();
        println!("[requester {seed}] dialing responder");

        let mut received_slices = HashMap::new();

        timeout(Duration::from_secs(30), async {
            loop {
                match swarm.select_next_some().await {
                    SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        println!(
                            "[requester {seed}] connected to {peer_id}; sending {} requests",
                            slice_ids.len()
                        );
                        tokio::time::sleep(Duration::from_millis(100)).await;
                        for id in &slice_ids {
                            let request = ProtocolSlice {
                                id: id.clone(),
                                header: Header {
                                    seq_num: 0,
                                    total_packets: 0,
                                },
                                data: Vec::new(),
                                is_parity: false,
                            };
                            swarm
                                .behaviour_mut()
                                .repair_rpc
                                .send_request(&peer_id, request);
                        }
                        println!("[requester {seed}] all requests sent");
                    }
                    SwarmEvent::Behaviour(FractalBehaviourEvent::RepairRpc(event)) => {
                        if let request_response::Event::Message { message, .. } = event {
                            if let request_response::Message::Response { response, .. } = message {
                                let prev = received_slices.len();
                                let id = response.id.clone();
                                let len = response.data.len();
                                received_slices.insert(id.clone(), response);
                                if received_slices.len() > prev {
                                    println!(
                                        "[requester {seed}] response {}/{}: {} ({} bytes)",
                                        received_slices.len(),
                                        slice_ids.len(),
                                        id,
                                        len
                                    );
                                }
                                if received_slices.len() == slice_ids.len() {
                                    println!("[requester {seed}] all responses received");
                                    break;
                                }
                            }
                        }
                    }
                    other => {
                        println!("[requester {seed}] event: {other:?}");
                    }
                }
            }
        })
        .await
        .expect("requester timed out");

        received_slices
    }

    fn git_output(repo: &Path, args: &[&str]) -> String {
        let output = Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(args)
            .output()
            .expect("spawn git");
        assert!(
            output.status.success(),
            "git -C {} {} failed",
            repo.display(),
            args.join(" ")
        );
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }
}
