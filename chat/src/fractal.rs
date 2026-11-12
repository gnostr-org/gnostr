//! Fractal / `perfect_ip` repair sidecar for `gnostr chat`.
//!
//! This module runs a dedicated libp2p request/response swarm on
//! `/fractal/repair/1.0.0/json` so chat peers can exchange `ProtocolSlice`
//! packets without disturbing the main gossipsub chat transport.

use std::{
    collections::{BTreeMap, HashMap, HashSet},
    path::Path,
    sync::Arc,
    time::Duration,
};

use anyhow::{anyhow, Context, Result};
use futures::StreamExt;
use gnostr_p2p::{
    keypair_from_seed,
    perfect_ip_core::{
        build_fractal_swarm, reconstruct_directory, reconstruct_payload, run_directory_walk,
        sha256_hex, DirectoryWalk, FractalBehaviourEvent, IntegrityManager, ProtocolSlice,
    },
};
use libp2p::{request_response, swarm::SwarmEvent, Multiaddr, PeerId};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use tokio::time::timeout;
use tracing::debug;

/// Handle for the local fractal repair service started by chat.
pub struct LocalFractalRepairService {
    peer_id: PeerId,
    listen_addr: Multiaddr,
    manager: Arc<Mutex<IntegrityManager>>,
    join_handle: tokio::task::JoinHandle<()>,
}

impl Drop for LocalFractalRepairService {
    fn drop(&mut self) {
        self.join_handle.abort();
    }
}

impl LocalFractalRepairService {
    pub async fn new(topic: &str) -> Result<Self> {
        let keypair = keypair_from_seed(Some(format!("gnostr-chat-fractal-{topic}")));
        let peer_id = keypair.public().to_peer_id();
        let manager = Arc::new(Mutex::new(IntegrityManager::new(Vec::new())));

        let (addr_tx, addr_rx) = tokio::sync::oneshot::channel::<Multiaddr>();
        let manager_clone = manager.clone();
        let join_handle = tokio::spawn(async move {
            if let Err(error) = run_repair_service(keypair, manager_clone, addr_tx).await {
                tracing::warn!("local fractal repair service exited with error: {error}");
            }
        });

        let listen_addr = timeout(Duration::from_secs(5), addr_rx)
            .await
            .context("local fractal repair service did not report a listen address")?
            .context("local fractal repair service address channel closed")?;

        Ok(Self {
            peer_id,
            listen_addr,
            manager,
            join_handle,
        })
    }

    pub fn peer_id(&self) -> PeerId {
        self.peer_id
    }

    pub fn listen_addr(&self) -> &Multiaddr {
        &self.listen_addr
    }

    /// Multiaddr that includes the repair peer id so remote peers can dial it.
    pub fn repair_addr(&self) -> Multiaddr {
        let mut addr = self.listen_addr.clone();
        addr.push(libp2p::multiaddr::Protocol::P2p(self.peer_id));
        addr
    }

    pub fn manager(&self) -> Arc<Mutex<IntegrityManager>> {
        self.manager.clone()
    }

    /// Replace the current slice inventory with `slices` and update the manifest.
    pub fn publish_slices(&self, slices: Vec<ProtocolSlice>) {
        let mut manager = self.manager.lock();
        let ids: Vec<String> = slices.iter().map(|s| s.id.clone()).collect();
        *manager = IntegrityManager::new(ids);
        for slice in slices {
            manager.record_slice(slice);
        }
    }
}

async fn run_repair_service(
    keypair: libp2p::identity::Keypair,
    manager: Arc<Mutex<IntegrityManager>>,
    addr_tx: tokio::sync::oneshot::Sender<Multiaddr>,
) -> Result<()> {
    let mut swarm = build_fractal_swarm(keypair)
        .await
        .map_err(|error| anyhow!("{error}"))?;
    swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;

    let mut addr_tx = Some(addr_tx);

    while let Some(event) = swarm.next().await {
        match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                debug!("local fractal repair service listening on {address}");
                if let Some(tx) = addr_tx.take() {
                    let _ = tx.send(address);
                }
            }
            SwarmEvent::Behaviour(FractalBehaviourEvent::RepairRpc(event)) => {
                if let request_response::Event::Message { message, .. } = event {
                    if let request_response::Message::Request {
                        request, channel, ..
                    } = message
                    {
                        let response = manager
                            .lock()
                            .received_slices
                            .get(&request.id)
                            .cloned()
                            .unwrap_or_else(|| ProtocolSlice {
                                id: request.id.clone(),
                                header: request.header.clone(),
                                data: Vec::new(),
                                is_parity: false,
                            });
                        let _ = swarm
                            .behaviour_mut()
                            .repair_rpc
                            .send_response(channel, response);
                    }
                }
            }
            _ => {}
        }
    }

    Ok(())
}

/// Dial `repair_addr` and request every slice id in `manifest`.
///
/// Retries missing slices on a 2-second interval until the full manifest is
/// received or `timeout_duration` elapses.
pub async fn request_slices(
    repair_addr: Multiaddr,
    manifest: Vec<String>,
    timeout_duration: Duration,
) -> Result<IntegrityManager> {
    let keypair = keypair_from_seed(None);
    let mut swarm = build_fractal_swarm(keypair)
        .await
        .map_err(|error| anyhow!("{error}"))?;
    swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;
    swarm.dial(repair_addr)?;

    let mut received_slices = HashMap::<String, ProtocolSlice>::new();
    let mut pending: HashSet<String> = HashSet::new();
    let mut peer_id: Option<PeerId> = None;
    let mut retry_interval = tokio::time::interval(Duration::from_secs(2));
    retry_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    timeout(timeout_duration, async {
        loop {
            tokio::select! {
                event = swarm.select_next_some() => {
                    match event {
                        SwarmEvent::ConnectionEstablished { peer_id: pid, .. } => {
                            peer_id = Some(pid);
                            pending = manifest.iter().cloned().collect();
                            tokio::time::sleep(Duration::from_millis(100)).await;
                            send_requests(&mut swarm, &pid, &pending);
                        }
                        SwarmEvent::Behaviour(FractalBehaviourEvent::RepairRpc(event)) => {
                            if let request_response::Event::Message { message, .. } = event {
                                if let request_response::Message::Response { response, .. } = message {
                                    pending.remove(&response.id);
                                    received_slices.insert(response.id.clone(), response);
                                    if received_slices.len() == manifest.len() {
                                        break;
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                _ = retry_interval.tick() => {
                    if let Some(pid) = peer_id {
                        if !pending.is_empty() && received_slices.len() < manifest.len() {
                            let missing: HashSet<String> = pending.iter().take(10).cloned().collect();
                            send_requests(&mut swarm, &pid, &missing);
                        }
                    }
                }
            }
        }
    })
    .await
    .context("timed out waiting for fractal slices")?;

    if received_slices.len() != manifest.len() {
        return Err(anyhow!(
            "incomplete slice download: {}/{} received",
            received_slices.len(),
            manifest.len()
        ));
    }

    let mut manager = IntegrityManager::new(manifest);
    for slice in received_slices.into_values() {
        manager.record_slice(slice);
    }
    Ok(manager)
}

fn send_requests(
    swarm: &mut libp2p::Swarm<gnostr_p2p::perfect_ip_core::FractalBehaviour>,
    peer_id: &PeerId,
    ids: &HashSet<String>,
) {
    for id in ids {
        let request = ProtocolSlice {
            id: id.clone(),
            header: gnostr_p2p::perfect_ip_core::Header {
                seq_num: 0,
                total_packets: 0,
            },
            data: Vec::new(),
            is_parity: false,
        };
        swarm
            .behaviour_mut()
            .repair_rpc
            .send_request(peer_id, request);
    }
}

/// Wire manifest describing a packetized payload.
///
/// `files` maps the original relative path to the ordered slice ids that
/// reconstruct that file. `empty_dirs` preserves empty directory entries.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PerfectIpManifest {
    pub root: String,
    pub files: BTreeMap<String, Vec<String>>,
    pub empty_dirs: std::collections::BTreeSet<String>,
    pub repair_peer: String,
    pub repair_addr: String,
    pub sha256: String,
}

impl PerfectIpManifest {
    pub fn all_slice_ids(&self) -> Vec<String> {
        self.files
            .values()
            .flat_map(|ids| ids.iter().cloned())
            .collect()
    }

    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    pub fn total_slices(&self) -> usize {
        self.files.values().map(|ids| ids.len()).sum()
    }
}

/// Packetize a single file or directory tree.
pub fn packetize_path(
    path: &Path,
    recursive: bool,
    depth: usize,
) -> Result<(DirectoryWalk, Vec<ProtocolSlice>, String)> {
    let (walk, sha256) = if recursive {
        let walk = run_directory_walk(path, depth, false)
            .context("failed to walk directory for perfect_ip")?;
        (walk, String::new())
    } else {
        let bytes = std::fs::read(path).context("failed to read file for perfect_ip")?;
        let sha256 = sha256_hex(&bytes);
        let batch = gnostr_p2p::perfect_ip_core::packetize("ROOT".to_string(), bytes);
        let mut files = BTreeMap::new();
        files.insert(path.to_string_lossy().into_owned(), batch.packets);
        let walk = DirectoryWalk {
            files,
            empty_dirs: std::collections::BTreeSet::new(),
        };
        (walk, sha256)
    };

    let all_slices: Vec<ProtocolSlice> = walk
        .files
        .values()
        .flat_map(|packets| packets.iter().cloned())
        .collect();

    Ok((walk, all_slices, sha256))
}

/// Build a `PerfectIpManifest` from a `DirectoryWalk`.
pub fn build_manifest(
    walk: &DirectoryWalk,
    repair_peer: String,
    repair_addr: String,
    sha256: String,
) -> PerfectIpManifest {
    let files = walk
        .files
        .iter()
        .map(|(path, slices)| (path.clone(), slices.iter().map(|s| s.id.clone()).collect()))
        .collect();
    PerfectIpManifest {
        root: "ROOT".to_string(),
        files,
        empty_dirs: walk.empty_dirs.clone(),
        repair_peer,
        repair_addr,
        sha256,
    }
}

/// Reconstruct a payload from a populated `IntegrityManager` and manifest.
pub fn reconstruct_from_manager(
    manager: &IntegrityManager,
    manifest: &PerfectIpManifest,
    output_dir: &Path,
) -> Result<()> {
    if manifest.files.len() == 1 && manifest.empty_dirs.is_empty() {
        let slices = manifest.files.values().next().expect("one file");
        let packets: Vec<ProtocolSlice> = slices
            .iter()
            .map(|id| {
                manager
                    .received_slices
                    .get(id)
                    .cloned()
                    .ok_or_else(|| anyhow!("missing slice {id} during reconstruction"))
            })
            .collect::<Result<Vec<_>>>()?;
        let payload = reconstruct_payload(&packets);
        std::fs::create_dir_all(output_dir.parent().unwrap_or(output_dir.as_ref()))?;
        std::fs::write(output_dir, payload)?;
        return Ok(());
    }

    let mut files = BTreeMap::new();
    for (path, ids) in &manifest.files {
        let slices: Vec<ProtocolSlice> = ids
            .iter()
            .map(|id| {
                manager
                    .received_slices
                    .get(id)
                    .cloned()
                    .ok_or_else(|| anyhow!("missing slice {id} during reconstruction"))
            })
            .collect::<Result<Vec<_>>>()?;
        files.insert(path.clone(), slices);
    }
    let walk = DirectoryWalk {
        files,
        empty_dirs: manifest.empty_dirs.clone(),
    };
    reconstruct_directory(walk, output_dir).context("failed to reconstruct directory")
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn packetize_single_file_roundtrips() {
        let mut file = NamedTempFile::new().expect("temp file");
        let payload = b"perfect_ip chat test payload\n".repeat(64);
        file.write_all(&payload).expect("write payload");

        let (walk, _slices, sha256) = packetize_path(file.path(), false, 3).expect("packetize");
        assert_eq!(walk.files.len(), 1);
        assert!(!sha256.is_empty());

        let manifest = build_manifest(
            &walk,
            "test-peer".to_string(),
            "/ip4/127.0.0.1/udp/0/quic-v1".to_string(),
            sha256.clone(),
        );
        assert_eq!(manifest.file_count(), 1);
        assert!(!manifest.all_slice_ids().is_empty());
        assert_eq!(manifest.sha256, sha256);

        let json = serde_json::to_string(&manifest).expect("serialize manifest");
        let decoded: PerfectIpManifest = serde_json::from_str(&json).expect("deserialize manifest");
        assert_eq!(manifest, decoded);
    }

    #[test]
    fn reconstruct_single_file_from_manager() {
        let mut file = NamedTempFile::new().expect("temp file");
        let payload = b"reconstruct from manager\n".repeat(32);
        file.write_all(&payload).expect("write payload");

        let (walk, _slices, sha256) = packetize_path(file.path(), false, 3).expect("packetize");
        let manifest = build_manifest(
            &walk,
            "peer".to_string(),
            "/ip4/127.0.0.1/udp/0/quic-v1".to_string(),
            sha256,
        );

        let mut manager = IntegrityManager::new(manifest.all_slice_ids());
        for slices in walk.files.values() {
            for slice in slices {
                manager.record_slice(slice.clone());
            }
        }

        let output = NamedTempFile::new().expect("output temp file");
        reconstruct_from_manager(&manager, &manifest, output.path()).expect("reconstruct");

        let reconstructed = std::fs::read(output.path()).expect("read reconstructed");
        assert_eq!(reconstructed, payload);
    }

    #[tokio::test]
    #[ignore = "networked p2p repair test: run with --ignored --nocapture"]
    async fn local_repair_service_answers_requests() {
        let payload = b"repair service test payload".to_vec();
        let batch = gnostr_p2p::perfect_ip_core::packetize("ROOT".to_string(), payload.clone());
        let service = LocalFractalRepairService::new("test-topic")
            .await
            .expect("start service");
        service.publish_slices(batch.packets.clone());

        let ids: Vec<String> = batch.packets.iter().map(|s| s.id.clone()).collect();
        let manager = request_slices(service.repair_addr(), ids, Duration::from_secs(10))
            .await
            .expect("request slices");

        let reconstructed = reconstruct_payload(
            &manager
                .received_slices
                .values()
                .cloned()
                .collect::<Vec<_>>(),
        );
        assert_eq!(reconstructed, payload);
    }
}
