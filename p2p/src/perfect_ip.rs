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

use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::io;
use std::path::Path;

use futures::StreamExt;
use libp2p::{
    request_response,
    swarm::{NetworkBehaviour, Swarm, SwarmEvent},
    StreamProtocol,
};
use serde::{Deserialize, Serialize};

/// Packet header metadata shared by all packet types in the tree.
///
/// `seq_num` is assigned in the order packets are emitted by the recursive
/// packetizer. `total_packets` is populated only after packetization has
/// finished, so callers can treat the batch as self-describing once finalized.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Header {
    /// Monotonic sequence number assigned during packetization.
    pub seq_num: u32,
    /// Total number of packets in the finalized batch.
    pub total_packets: u32,
}

/// A single packet produced by the recursive packetizer.
///
/// `id` carries the recursive path for the packet, such as `ROOT.0.1.P`.
/// `is_parity` marks frames that store XOR parity rather than original user
/// data. `data` always contains the raw bytes that would be transmitted on the
/// wire inside the JSON-encoded repair message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolSlice {
    /// Stable recursive packet identifier.
    pub id: String,
    /// Packet sequencing metadata.
    pub header: Header,
    /// Raw payload bytes for the packet or parity frame.
    pub data: Vec<u8>,
    /// `true` when this slice is a parity frame.
    pub is_parity: bool,
}

/// Finalized packet tree output.
///
/// `total_packets` is duplicated here so callers can inspect batch metadata
/// without walking the packet list. The packet list is stable and already has
/// each packet header patched to reflect the finalized count.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PacketBatch {
    /// Number of packets in the batch.
    pub total_packets: u32,
    /// Finalized packets with matching `total_packets` headers.
    pub packets: Vec<ProtocolSlice>,
}

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

/// Tracks the expected manifest and the packets that have arrived so far.
///
/// The manager is deliberately simple: it stores the expected ids, the received
/// slices, and exposes helpers for missing-node inspection, parity validation,
/// and persistence.
#[derive(Debug, Clone, Default)]
pub struct IntegrityManager {
    manifest: HashSet<String>,
    pub received_slices: HashMap<String, ProtocolSlice>,
}

impl IntegrityManager {
    /// Create a manager from the set of expected packet ids.
    ///
    /// The manifest is stored as a set so lookups stay cheap even for larger
    /// packet trees.
    pub fn new(expected_ids: Vec<String>) -> Self {
        Self {
            manifest: expected_ids.into_iter().collect(),
            received_slices: HashMap::new(),
        }
    }

    /// Record a received packet by id, replacing any prior packet with the same id.
    ///
    /// Re-recording a slice is allowed and simply overwrites the previous
    /// entry. That makes repair retries and late arrivals idempotent.
    pub fn record_slice(&mut self, slice: ProtocolSlice) {
        self.received_slices.insert(slice.id.clone(), slice);
    }

    /// Persist the received packet map to disk using bincode.
    ///
    /// This writes only the received slice map. The caller supplies the
    /// manifest again on load so persisted state stays lightweight and portable.
    pub fn persist(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let encoded = bincode::serialize(&self.received_slices)
            .map_err(|error| io::Error::new(io::ErrorKind::Other, error))?;
        std::fs::write(path, encoded)
    }

    /// Load a persisted packet map from disk and attach it to the given manifest.
    ///
    /// The manifest is not persisted with the slice data; callers are expected
    /// to regenerate or reload it independently.
    pub fn load_from_disk(
        path: impl AsRef<Path>,
        manifest: HashSet<String>,
    ) -> io::Result<Self> {
        let bytes = std::fs::read(path)?;
        let received_slices: HashMap<String, ProtocolSlice> =
            bincode::deserialize(&bytes).map_err(|error| io::Error::new(io::ErrorKind::Other, error))?;
        Ok(Self {
            manifest,
            received_slices,
        })
    }

    /// Return the packet ids from the manifest that are still missing.
    ///
    /// The returned ids preserve the manifest set's iteration order only in the
    /// sense that they are collected into a vector; callers should sort if they
    /// need deterministic display order.
    pub fn get_missing_nodes(&self) -> Vec<String> {
        self.manifest
            .iter()
            .filter(|id| !self.received_slices.contains_key(*id))
            .cloned()
            .collect()
    }

    /// Verify that every parity slice matches the XOR of its sibling data slices.
    ///
    /// If a parity node's siblings are both present, the check recomputes the
    /// XOR and compares it with the stored parity bytes. Missing siblings are
    /// ignored so partial downloads can still pass for the fragments they have.
    pub fn verify_integrity(&self) -> bool {
        for (id, slice) in &self.received_slices {
            if !slice.is_parity {
                continue;
            }

            let Some(base_id) = id.strip_suffix(".P") else {
                return false;
            };
            let left = self.received_slices.get(&format!("{}.0", base_id));
            let right = self.received_slices.get(&format!("{}.1", base_id));

            if let (Some(left), Some(right)) = (left, right) {
                if calculate_parity(&left.data, &right.data) != slice.data {
                    return false;
                }
            }
        }

        true
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

/// Maximum payload size this packet protocol is allowed to emit.
///
/// The tree uses half-sized leaves so that parity slices for sibling pairs stay
/// under the same transport ceiling as the data leaves.
pub const MTU_PAYLOAD: usize = 1460;
const MAX_LEAF_PAYLOAD: usize = MTU_PAYLOAD / 2;

/// XOR two payloads into a parity buffer.
///
/// Missing bytes are treated as zero so the returned buffer is as long as the
/// larger input. This is the core repair primitive used at every branch in the
/// recursive packet tree.
pub fn calculate_parity(left: &[u8], right: &[u8]) -> Vec<u8> {
    let max_len = left.len().max(right.len());
    let mut parity = vec![0; max_len];
    for i in 0..max_len {
        let l = if i < left.len() { left[i] } else { 0 };
        let r = if i < right.len() { right[i] } else { 0 };
        parity[i] = l ^ r;
    }
    parity
}

/// Generate the expected packet ids for a recursive packet tree.
///
/// The returned manifest mirrors [`process_slice`] exactly: leaf nodes are
/// emitted when the payload fits within [`MAX_LEAF_PAYLOAD`], and internal
/// nodes append `.0`, `.1`, and `.P` entries in tree order. This is what lets
/// [`IntegrityManager`] tell which packets are still missing.
pub fn generate_manifest(id: String, len: usize) -> Vec<String> {
    if len <= MAX_LEAF_PAYLOAD {
        return vec![id];
    }

    let half = len / 2;
    let mut ids = generate_manifest(format!("{}.0", id), half);
    ids.append(&mut generate_manifest(format!("{}.1", id), len - half));
    ids.push(format!("{}.P", id));
    ids
}

/// Recover the missing payload by XORing the sibling payload and parity frame.
///
/// `expected_len` trims the recovered buffer back to the original payload
/// length. If the sibling and parity bytes are from the same branch, the XOR
/// yields the missing sibling exactly.
pub fn recover_missing_data(expected_len: usize, sibling: &[u8], parity: &[u8]) -> Vec<u8> {
    let recovered = calculate_parity(sibling, parity);
    recovered.into_iter().take(expected_len).collect()
}

/// Rebuild a missing packet using a sibling packet and parity packet.
///
/// The returned slice is marked as data and receives the next sequence number.
/// This is a convenience wrapper for repair handlers that need to synthesize a
/// complete [`ProtocolSlice`] from branch-local evidence.
pub fn recover_missing_slice(
    id: String,
    expected_len: usize,
    sibling: &ProtocolSlice,
    parity: &ProtocolSlice,
    seq: &mut u32,
) -> ProtocolSlice {
    let header = Header {
        seq_num: *seq,
        total_packets: sibling.header.total_packets.max(parity.header.total_packets),
    };
    *seq += 1;

    ProtocolSlice {
        id,
        header,
        data: recover_missing_data(expected_len, &sibling.data, &parity.data),
        is_parity: false,
    }
}

/// Recursively split a payload into MTU-safe slices and parity frames.
///
/// Leaf packets are emitted when the payload is at or below
/// [`MAX_LEAF_PAYLOAD`]. Internal nodes emit left, right, and parity frames in
/// a deterministic order so the manifest and sequence numbers line up exactly.
pub fn process_slice(id: String, data: Vec<u8>, seq: &mut u32) -> Vec<ProtocolSlice> {
    if data.len() <= MAX_LEAF_PAYLOAD {
        let slice = ProtocolSlice {
            id,
            header: Header {
                seq_num: *seq,
                total_packets: 0,
            },
            data,
            is_parity: false,
        };
        *seq += 1;
        return vec![slice];
    }

    let half = data.len() / 2;
    let left_data = data[..half].to_vec();
    let right_data = data[half..].to_vec();

    let parity = calculate_parity(&left_data, &right_data);
    let mut slices = process_slice(format!("{}.0", id), left_data, seq);
    slices.append(&mut process_slice(format!("{}.1", id), right_data, seq));

    slices.push(ProtocolSlice {
        id: format!("{}.P", id),
        header: Header {
            seq_num: *seq,
            total_packets: 0,
        },
        data: parity,
        is_parity: true,
    });
    *seq += 1;

    slices
}

/// Packetize a payload and finalize the batch metadata.
///
/// This is the preferred entrypoint for callers that want a ready-to-send
/// packet tree with `total_packets` filled in. The returned batch is ready for
/// logging, persistence, or network repair.
pub fn packetize(id: String, data: Vec<u8>) -> PacketBatch {
    let mut seq = 0;
    let mut packets = process_slice(id, data, &mut seq);
    let total_packets = packets.len() as u32;
    for packet in &mut packets {
        packet.header.total_packets = total_packets;
    }
    PacketBatch {
        total_packets,
        packets,
    }
}

/// Render packet summaries for logs or diagnostics.
///
/// This is intentionally human-facing output for terminal demos and debugging.
/// It keeps the packet ids, sequence numbers, types, and sizes aligned in a
/// single line per packet.
pub fn summarize_packets(packets: &[ProtocolSlice]) -> Vec<String> {
    packets
        .iter()
        .map(|packet| {
            format!(
                "ID: {:<8} | Seq: {:>2}/{} | Type: {:<6} | Size: {}B",
                packet.id,
                packet.header.seq_num,
                packet.header.total_packets,
                if packet.is_parity { "PARITY" } else { "DATA" },
                packet.data.len()
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keypair_from_seed;
    use libp2p::Multiaddr;
    use tempfile::NamedTempFile;

    #[test]
    fn process_slice_emits_parity_and_data() {
        let raw_data = vec![0xAB; 2000];
        let batch = packetize("ROOT".to_string(), raw_data);
        let packets = batch.packets;

        assert!(!packets.is_empty());
        assert!(packets.iter().any(|packet| packet.is_parity));
        assert!(packets.iter().any(|packet| packet.id == "ROOT.P"));
        assert!(packets.iter().any(|packet| packet.id.starts_with("ROOT.0.")));
        assert!(packets.iter().any(|packet| packet.id.starts_with("ROOT.1.")));
        assert!(packets.iter().all(|packet| packet.data.len() <= MTU_PAYLOAD));
        assert!(packets
            .iter()
            .all(|packet| packet.header.total_packets == batch.total_packets));
    }

    #[test]
    fn calculate_parity_xors_equal_length_blocks() {
        let left = [0xDE, 0xAD, 0xBE];
        let right = [0x01, 0x02, 0x03];
        assert_eq!(calculate_parity(&left, &right), vec![0xDF, 0xAF, 0xBD]);
    }

    #[test]
    fn generate_manifest_matches_recursive_packet_tree() {
        let manifest = generate_manifest("ROOT".to_string(), 2000);

        assert!(manifest.contains(&"ROOT.P".to_string()));
        assert!(manifest.iter().any(|id| id.starts_with("ROOT.0.")));
        assert!(manifest.iter().any(|id| id.starts_with("ROOT.1.")));
        assert_eq!(manifest.len(), packetize("ROOT".to_string(), vec![0xAB; 2000]).total_packets as usize);
    }

    #[test]
    fn recover_missing_data_restores_xor_partner() {
        let left = vec![0xDE, 0xAD, 0xBE];
        let right = vec![0x01, 0x02, 0x03];
        let parity = calculate_parity(&left, &right);

        assert_eq!(recover_missing_data(left.len(), &right, &parity), left);
        assert_eq!(recover_missing_data(right.len(), &left, &parity), right);
    }

    #[test]
    fn recover_missing_slice_rebuilds_header_and_payload() {
        let sibling = ProtocolSlice {
            id: "ROOT.1".to_string(),
            header: Header {
                seq_num: 1,
                total_packets: 3,
            },
            data: vec![0x01, 0x02, 0x03],
            is_parity: false,
        };
        let parity = ProtocolSlice {
            id: "ROOT.P".to_string(),
            header: Header {
                seq_num: 2,
                total_packets: 3,
            },
            data: vec![0xDF, 0xAF, 0xBD],
            is_parity: true,
        };
        let mut seq = 3;

        let recovered = recover_missing_slice("ROOT.0".to_string(), 3, &sibling, &parity, &mut seq);

        assert_eq!(recovered.id, "ROOT.0");
        assert_eq!(recovered.data, vec![0xDE, 0xAD, 0xBE]);
        assert_eq!(recovered.header.seq_num, 3);
        assert_eq!(recovered.header.total_packets, 3);
        assert!(!recovered.is_parity);
    }

    #[test]
    fn get_missing_nodes_returns_unseen_manifest_entries() {
        let mut manager = IntegrityManager::new(vec![
            "ROOT.0".to_string(),
            "ROOT.1".to_string(),
            "ROOT.P".to_string(),
        ]);
        manager.record_slice(ProtocolSlice {
            id: "ROOT.0".to_string(),
            header: Header {
                seq_num: 0,
                total_packets: 3,
            },
            data: vec![0xDE],
            is_parity: false,
        });

        let missing = manager.get_missing_nodes();
        assert_eq!(missing.len(), 2);
        assert!(missing.contains(&"ROOT.1".to_string()));
        assert!(missing.contains(&"ROOT.P".to_string()));
    }

    #[test]
    fn verify_integrity_checks_parity_frames() {
        let mut manager = IntegrityManager::new(vec![
            "ROOT.0".to_string(),
            "ROOT.1".to_string(),
            "ROOT.P".to_string(),
        ]);
        let left = ProtocolSlice {
            id: "ROOT.0".to_string(),
            header: Header {
                seq_num: 0,
                total_packets: 3,
            },
            data: vec![0xDE, 0xAD, 0xBE],
            is_parity: false,
        };
        let right = ProtocolSlice {
            id: "ROOT.1".to_string(),
            header: Header {
                seq_num: 1,
                total_packets: 3,
            },
            data: vec![0x01, 0x02, 0x03],
            is_parity: false,
        };
        let parity = ProtocolSlice {
            id: "ROOT.P".to_string(),
            header: Header {
                seq_num: 2,
                total_packets: 3,
            },
            data: calculate_parity(&left.data, &right.data),
            is_parity: true,
        };

        manager.record_slice(left);
        manager.record_slice(right);
        manager.record_slice(parity);
        assert!(manager.verify_integrity());

        let mut corrupted = manager.clone();
        corrupted
            .received_slices
            .get_mut("ROOT.P")
            .expect("parity slice")
            .data[0] ^= 0xFF;
        assert!(!corrupted.verify_integrity());
    }

    #[test]
    fn persist_and_load_from_disk_round_trips_received_slices() {
        let temp = NamedTempFile::new().expect("temp file");
        let mut manager = IntegrityManager::new(vec![
            "ROOT.0".to_string(),
            "ROOT.1".to_string(),
            "ROOT.P".to_string(),
        ]);
        manager.record_slice(ProtocolSlice {
            id: "ROOT.0".to_string(),
            header: Header {
                seq_num: 0,
                total_packets: 3,
            },
            data: vec![0xDE, 0xAD, 0xBE],
            is_parity: false,
        });
        manager.record_slice(ProtocolSlice {
            id: "ROOT.P".to_string(),
            header: Header {
                seq_num: 1,
                total_packets: 3,
            },
            data: vec![0xAA],
            is_parity: true,
        });

        manager.persist(temp.path()).expect("persist");
        let loaded = IntegrityManager::load_from_disk(
            temp.path(),
            vec![
                "ROOT.0".to_string(),
                "ROOT.1".to_string(),
                "ROOT.P".to_string(),
            ]
            .into_iter()
            .collect(),
        )
        .expect("load");

        assert_eq!(loaded.received_slices.len(), 2);
        assert!(loaded.received_slices.contains_key("ROOT.0"));
        assert!(loaded.received_slices.contains_key("ROOT.P"));
        assert!(loaded.get_missing_nodes().contains(&"ROOT.1".to_string()));
    }

    #[test]
    fn protocol_slice_json_round_trips_bytes_and_flags() {
        let slice = ProtocolSlice {
            id: "ROOT.0.P".to_string(),
            header: Header {
                seq_num: 7,
                total_packets: 63,
            },
            data: vec![0x00, 0xFF, 0x10, 0x20],
            is_parity: true,
        };

        let encoded = serde_json::to_string(&slice).expect("encode slice");
        assert!(encoded.contains("\"id\":\"ROOT.0.P\""));
        assert!(encoded.contains("\"is_parity\":true"));
        assert!(encoded.contains("\"data\":[0,255,16,32]"));

        let decoded: ProtocolSlice = serde_json::from_str(&encoded).expect("decode slice");
        assert_eq!(decoded, slice);
    }

    #[test]
    fn summarize_packets_shows_packet_inventory_details() {
        let batch = packetize("ROOT".to_string(), vec![0xAB; 2000]);
        let lines = summarize_packets(&batch.packets);

        assert_eq!(lines.len(), batch.total_packets as usize);
        assert!(lines.iter().any(|line| line.contains("ID: ROOT.P")));
        assert!(lines
            .iter()
            .any(|line| line.contains(&format!("Seq: {:>2}/{}", 0, batch.total_packets))));
        assert!(lines.iter().any(|line| line.contains("Type: DATA")));
        assert!(lines.iter().any(|line| line.contains("Type: PARITY")));
    }

    #[test]
    fn dump_full_protocol_for_nocapture() {
        use sha2::{Digest, Sha256};

        fn reconstruct_payload_for_test(packets: &[ProtocolSlice]) -> Vec<u8> {
            let mut leaves: Vec<_> = packets.iter().filter(|packet| !packet.is_parity).collect();
            leaves.sort_by_key(|packet| packet.header.seq_num);
            leaves
                .into_iter()
                .flat_map(|packet| packet.data.clone())
                .collect()
        }

        let payload = vec![0xAB; 3000];
        let batch = packetize("ROOT".to_string(), payload.clone());
        let manifest = generate_manifest("ROOT".to_string(), payload.len());
        let mut manager = IntegrityManager::new(manifest.clone());

        println!("perfect_ip manifest ({} ids):", manifest.len());
        for id in &manifest {
            println!("  {id}");
        }

        println!("perfect_ip packet inventory ({} packets):", batch.total_packets);
        for line in summarize_packets(&batch.packets) {
            println!("{line}");
        }

        println!("perfect_ip packet batch json:");
        println!(
            "{}",
            serde_json::to_string(&batch).expect("serialize packet batch")
        );

        println!("perfect_ip first packet json:");
        println!(
            "{}",
            serde_json::to_string(&batch.packets.first().expect("first packet"))
                .expect("serialize first packet")
        );

        println!("perfect_ip last packet json:");
        println!(
            "{}",
            serde_json::to_string(&batch.packets.last().expect("last packet"))
                .expect("serialize last packet")
        );

        for slice in batch.packets.clone() {
            manager.record_slice(slice);
        }
        println!("perfect_ip missing nodes: {:?}", manager.get_missing_nodes());
        println!("perfect_ip integrity verified: {}", manager.verify_integrity());

        let reconstructed = reconstruct_payload_for_test(&batch.packets);
        let reconstructed_sha256 = format!("{:x}", Sha256::digest(&reconstructed));
        let sender_sha256 = format!("{:x}", Sha256::digest(&payload));
        println!("perfect_ip reconstructed sha256: {reconstructed_sha256}");
        println!("perfect_ip sender sha256: {sender_sha256}");
        assert_eq!(reconstructed, payload);
    }

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
}
