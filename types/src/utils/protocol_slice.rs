//! Primitive packet types for the `perfect_ip` protocol.
//!
//! These are the pure data types and helpers exchanged on the wire (serialized
//! as JSON). Networking/behaviour code lives in `gnostr-p2p`.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs;
use std::io;
use std::path::Path;

use bincode;
use log::{Level, debug, info, log_enabled};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Packet header metadata shared by all packet types in the tree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Header {
    pub seq_num: u32,
    pub total_packets: u32,
}

/// A single packet produced by the recursive packetizer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolSlice {
    pub id: String,
    pub header: Header,
    pub data: Vec<u8>,
    pub is_parity: bool,
}

/// Finalized packet tree output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PacketBatch {
    pub total_packets: u32,
    pub packets: Vec<ProtocolSlice>,
}

/// XOR two payloads into a parity buffer.
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

/// Tracks the expected manifest and the packets that have arrived so far.
#[derive(Debug, Clone, Default)]
pub struct IntegrityManager {
    manifest: HashSet<String>,
    pub received_slices: HashMap<String, ProtocolSlice>,
}

impl IntegrityManager {
    pub fn new(expected_ids: Vec<String>) -> Self {
        Self {
            manifest: expected_ids.into_iter().collect(),
            received_slices: HashMap::new(),
        }
    }

    pub fn record_slice(&mut self, slice: ProtocolSlice) {
        self.received_slices.insert(slice.id.clone(), slice);
    }

    pub fn persist(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let encoded = bincode::serialize(&self.received_slices)
            .map_err(|error| io::Error::new(io::ErrorKind::Other, error))?;
        std::fs::write(path, encoded)
    }

    pub fn load_from_disk(path: impl AsRef<Path>, manifest: HashSet<String>) -> io::Result<Self> {
        let bytes = std::fs::read(path)?;
        let received_slices: HashMap<String, ProtocolSlice> = bincode::deserialize(&bytes)
            .map_err(|error| io::Error::new(io::ErrorKind::Other, error))?;
        Ok(Self {
            manifest,
            received_slices,
        })
    }

    pub fn get_missing_nodes(&self) -> Vec<String> {
        self.manifest
            .iter()
            .filter(|id| !self.received_slices.contains_key(*id))
            .cloned()
            .collect()
    }

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

/// Maximum payload size this packet protocol is allowed to emit.
///
/// The tree uses half-sized leaves so that parity slices for sibling pairs stay
/// under the same transport ceiling as the data leaves.
pub const MTU_PAYLOAD: usize = 1460;
const MAX_LEAF_PAYLOAD: usize = MTU_PAYLOAD / 2;

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
        total_packets: sibling
            .header
            .total_packets
            .max(parity.header.total_packets),
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

/// Reconstruct the original payload from a collection of non-parity packets.
///
/// Leaves are sorted by sequence number and concatenated. This is the inverse
/// of [`packetize`] for callers that already have every data slice.
pub fn reconstruct_payload(packets: &[ProtocolSlice]) -> Vec<u8> {
    let mut leaves: Vec<_> = packets.iter().filter(|packet| !packet.is_parity).collect();
    leaves.sort_by_key(|packet| packet.header.seq_num);
    leaves
        .into_iter()
        .flat_map(|packet| packet.data.clone())
        .collect()
}

/// Hex-encode a SHA-256 digest of the given bytes.
pub fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// Result of packetizing a directory tree.
///
/// `files` maps relative file paths to their slices. `empty_dirs` lists
/// directories whose entire subtree contained no files; these must be recreated
/// on the receiving side to produce a byte-for-byte directory match.
#[derive(Debug, Clone)]
pub struct DirectoryWalk {
    pub files: BTreeMap<String, Vec<ProtocolSlice>>,
    pub empty_dirs: BTreeSet<String>,
}

/// Recursively list a directory tree and packetize every file found.
///
/// `lines` receives a human-readable tree representation, while `all_packets`
/// receives a map from relative file path to the [`ProtocolSlice`]s produced for
/// that file. Empty directories are recorded in `empty_dirs`. Directories are
/// visited unconditionally up to `depth`.
///
/// Returns `true` if the visited subtree contains at least one file.
pub fn list_directory(
    root: &Path,
    current: &Path,
    depth: usize,
    level: usize,
    logging: bool,
    lines: &mut Vec<String>,
    all_packets: &mut BTreeMap<String, Vec<ProtocolSlice>>,
    empty_dirs: &mut BTreeSet<String>,
) -> io::Result<bool> {
    if level == 0 {
        lines.push(".".to_string());
    }

    let mut has_file_in_subtree = false;
    let mut entries = Vec::new();
    for entry in fs::read_dir(current)? {
        entries.push(entry?);
    }
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let path = entry.path();
        let relative = path.strip_prefix(root).unwrap_or(&path).to_path_buf();
        let indent = "  ".repeat(level + 1);
        let metadata = entry.metadata()?;

        if metadata.is_dir() {
            lines.push(format!("{indent}{}/", relative.display()));
            let child_has_file = list_directory(
                root,
                &path,
                depth,
                level + 1,
                logging,
                lines,
                all_packets,
                empty_dirs,
            )?;
            has_file_in_subtree = has_file_in_subtree || child_has_file;
        } else if metadata.is_file() {
            has_file_in_subtree = true;
            let bytes = fs::read(&path)?;
            lines.push(format!(
                "{indent}{} ({}B sha256:{})",
                relative.display(),
                bytes.len(),
                sha256_hex(&bytes)
            ));
            let rel_path = relative.to_string_lossy().into_owned();
            let batch = packetize(rel_path.clone(), bytes);
            all_packets
                .entry(rel_path)
                .or_default()
                .extend(batch.packets.clone());
            if logging {
                if !log_enabled!(Level::Debug) {
                    info!("File: {}", relative.display());
                    info!("Batch size: {} packets", batch.total_packets);
                }

                debug!(
                    "{indent}  perfect_ip packet batch: {} packets",
                    batch.total_packets
                );
                for line in summarize_packets(&batch.packets) {
                    debug!("{indent}  {line}");
                }
                for slice in &batch.packets {
                    let json = serde_json::to_string(slice).unwrap_or_default();
                    debug!("{indent}  payload: {json}");
                }
            }
        } else {
            lines.push(format!("{indent}{} [special]", relative.display()));
        }
    }

    if !has_file_in_subtree && level > 0 {
        let rel_path = current
            .strip_prefix(root)
            .unwrap_or(current)
            .to_string_lossy()
            .into_owned();
        empty_dirs.insert(rel_path);
    }

    Ok(has_file_in_subtree)
}

/// Walk a directory and return all packetized file slices grouped by path.
///
/// This is a convenience wrapper around [`list_directory`] that prints the
/// generated tree lines and returns a [`DirectoryWalk`] describing the files
/// and empty directories found.
pub fn run_directory_walk(path: &Path, depth: usize, logging: bool) -> io::Result<DirectoryWalk> {
    let mut lines = Vec::new();
    let mut files = BTreeMap::new();
    let mut empty_dirs = BTreeSet::new();
    list_directory(
        path,
        path,
        depth,
        0,
        logging,
        &mut lines,
        &mut files,
        &mut empty_dirs,
    )?;
    for line in lines {
        if log_enabled!(Level::Debug) {
            debug!("{line}");
        } else {
            info!("{line}");
        }
    }
    Ok(DirectoryWalk { files, empty_dirs })
}

/// Reconstruct a packetized directory tree under `out_path`.
///
/// Files are written to `out_path/<key>` and empty directories are recreated.
/// Missing parent directories are created automatically.
pub fn reconstruct_directory(walk: DirectoryWalk, out_path: &Path) -> io::Result<()> {
    for (rel_path, packets) in walk.files {
        let file_path = out_path.join(&rel_path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let reconstructed = reconstruct_payload(&packets);
        fs::write(&file_path, &reconstructed)?;
    }

    for rel_path in walk.empty_dirs {
        let dir_path = out_path.join(&rel_path);
        fs::create_dir_all(&dir_path)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

    #[test]
    fn process_slice_emits_parity_and_data() {
        let raw_data = vec![0xAB; 2000];
        let batch = packetize("ROOT".to_string(), raw_data);
        let packets = batch.packets;

        assert!(!packets.is_empty());
        assert!(packets.iter().any(|packet| packet.is_parity));
        assert!(packets.iter().any(|packet| packet.id == "ROOT.P"));
        assert!(
            packets
                .iter()
                .any(|packet| packet.id.starts_with("ROOT.0."))
        );
        assert!(
            packets
                .iter()
                .any(|packet| packet.id.starts_with("ROOT.1."))
        );
        assert!(
            packets
                .iter()
                .all(|packet| packet.data.len() <= MTU_PAYLOAD)
        );
        assert!(
            packets
                .iter()
                .all(|packet| packet.header.total_packets == batch.total_packets)
        );
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
        assert_eq!(
            manifest.len(),
            packetize("ROOT".to_string(), vec![0xAB; 2000]).total_packets as usize
        );
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
        use tempfile::NamedTempFile;
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
        assert!(
            lines
                .iter()
                .any(|line| line.contains(&format!("Seq: {:>2}/{}", 0, batch.total_packets)))
        );
        assert!(lines.iter().any(|line| line.contains("Type: DATA")));
        assert!(lines.iter().any(|line| line.contains("Type: PARITY")));
    }

    #[test]
    fn dump_full_protocol_for_nocapture() {
        let payload = vec![0xAB; 3000];
        let batch = packetize("ROOT".to_string(), payload.clone());
        let manifest = generate_manifest("ROOT".to_string(), payload.len());
        let mut manager = IntegrityManager::new(manifest.clone());

        println!("perfect_ip manifest ({} ids):", manifest.len());
        for id in &manifest {
            println!("  {id}");
        }

        println!(
            "perfect_ip packet inventory ({} packets):",
            batch.total_packets
        );
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
        println!(
            "perfect_ip missing nodes: {:?}",
            manager.get_missing_nodes()
        );
        println!(
            "perfect_ip integrity verified: {}",
            manager.verify_integrity()
        );

        let reconstructed = reconstruct_payload(&batch.packets);
        let reconstructed_sha256 = format!("{:x}", Sha256::digest(&reconstructed));
        let sender_sha256 = format!("{:x}", Sha256::digest(&payload));
        println!("perfect_ip reconstructed sha256: {reconstructed_sha256}");
        println!("perfect_ip sender sha256: {sender_sha256}");
        assert_eq!(reconstructed, payload);
    }
}
