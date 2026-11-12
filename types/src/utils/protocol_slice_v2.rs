//! v2 of the primitive packet types for the `perfect_ip` protocol.
//!
//! This is the data-plane half of the `p2p/TODO.md` gap-fill, implemented as
//! a parallel module so the v1 wire and data types stay untouched. The plans
//! implemented here:
//!
//! - **Plan 1 — explicit absence.** `Header` v2 carries a `len` field so a
//!   packet is self-describing, and the wire layer answers a missing slice
//!   with `Err("not held: <id>")` instead of v1's empty-placeholder
//!   `ProtocolSlice`, which was indistinguishable from a zero-length leaf.
//! - **Plan 3 — manifest v2 `{id, offset, len, sha256}`.** Every slice is
//!   self-verifying: [`IntegrityManagerV2::record_slice`] checks the sha256
//!   against the manifest entry and *rejects* mismatches into a `rejected`
//!   list without storing them.
//! - **Plan 4 — one-loss-per-pair ceiling.** The parity scheme is XOR-based:
//!   each parity frame recovers exactly one missing sibling per pair. Two
//!   losses in one pair make the subtree unrecoverable; this is a
//!   repair-assist layer, not a primary transport. [`IntegrityManagerV2::recover_missing`]
//!   reports such losses as unrecoverable instead of producing silent garbage.
//! - **Plan 5 — no `seq_num` reliance.** Reconstruction orders data leaves by
//!   manifest offset and asserts the assembled length equals the sum of the
//!   data-leaf lengths.
//! - **Plan 7 — persist format versioning.** [`IntegrityManagerV2::persist`]
//!   writes JSON with a `"v": 1` marker; [`IntegrityManagerV2::load_from_disk`]
//!   refuses any version it does not know and re-verifies every loaded slice,
//!   so tampered state is rejected rather than trusted.
//! - **Raw-leaf CIDs.** Every manifest entry's sha256 IS the multihash
//!   digest, so [`ManifestEntry::cid`] derives a `CIDv1(raw, sha2-256)` —
//!   the same shape `ipfs add --raw-leaves` produces — with no new
//!   dependencies and no manifest format change. A slice can be served and
//!   fetched back through any trustless gateway by CID and verified against
//!   the manifest on record ([`verify_slice_against_cid`]).
//!
//! Wire/behaviour code lives in `gnostr-p2p` (`fractal/fractal_ip_core_v2.rs`).

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

use log::{debug, log_enabled, trace, Level};

use super::protocol_slice::{MTU_PAYLOAD, calculate_parity, sha256_hex};

/// v2 leaf ceiling, mirroring v1. v1's `MAX_LEAF_PAYLOAD` is private, so the
/// same value is defined here to keep v1 untouched.
pub const MAX_LEAF_PAYLOAD_V2: usize = MTU_PAYLOAD / 2;

/// Persisted-state format version written by [`IntegrityManagerV2::persist`].
pub const PERSIST_FORMAT_VERSION_V2: u32 = 1;

/// CIDv1 version marker.
pub const CID_VERSION_V2: u8 = 0x01;
/// CIDv1 codec for an untyped raw block (`ipfs add --raw-leaves` shape).
pub const CID_CODEC_RAW_V2: u8 = 0x55;
/// Multihash code for sha2-256.
pub const CID_SHA2_256_V2: u8 = 0x12;
/// Multibase code for lowercase base32 (the `b` in `bafk...`).
pub const CID_MULTIBASE_BASE32_V2: char = 'b';

/// RFC 4648 lowercase base32 alphabet (multibase base32, no padding).
const BASE32_ALPHABET_LOWER: &[u8; 32] = b"abcdefghijklmnopqrstuvwxyz234567";

/// Encode bytes as lowercase base32 without padding (multibase base32).
fn base32_lower_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 8 / 5 + 8);
    let mut buffer: u32 = 0;
    let mut bits = 0u32;
    for &byte in bytes {
        buffer = (buffer << 8) | byte as u32;
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            let index = ((buffer >> bits) & 0x1F) as usize;
            out.push(BASE32_ALPHABET_LOWER[index] as char);
        }
    }
    if bits > 0 {
        let index = ((buffer << (5 - bits)) & 0x1F) as usize;
        out.push(BASE32_ALPHABET_LOWER[index] as char);
    }
    out
}

fn hex_decode_32(bytes: &[u8]) -> Option<[u8; 32]> {
    if bytes.len() != 64 {
        return None;
    }
    fn nibble(byte: u8) -> Option<u8> {
        match byte {
            b'0'..=b'9' => Some(byte - b'0'),
            b'a'..=b'f' => Some(byte - b'a' + 10),
            b'A'..=b'F' => Some(byte - b'A' + 10),
            _ => None,
        }
    }
    let (pairs, _) = bytes.as_chunks::<2>();
    let mut digest = [0u8; 32];
    for (index, pair) in pairs.iter().enumerate() {
        digest[index] = (nibble(pair[0])? << 4) | nibble(pair[1])?;
    }
    Some(digest)
}

/// Derive a `CIDv1(raw, sha2-256)` string from a sha256 hex digest.
///
/// The manifest sha256 is the raw multihash digest, so a slice's CID is
/// purely a function of bytes the manifest already carries — no format
/// change, no new dependencies. This is the exact shape Kubo produces for a
/// small file with `ipfs add --raw-leaves`, so each slice is independently
/// addressable through any IPFS gateway. Returns `None` for a malformed
/// digest (not 64 hex chars).
pub fn raw_leaf_cid_from_sha256(sha256_hex_digest: &str) -> Option<String> {
    let digest = hex_decode_32(sha256_hex_digest.as_bytes())?;
    // CIDv1: version varint || codec varint || multihash(code, len, digest).
    // All values are < 128, so every unsigned-varint is a single byte.
    let mut body = Vec::with_capacity(4 + 32);
    body.push(CID_VERSION_V2);
    body.push(CID_CODEC_RAW_V2);
    body.push(CID_SHA2_256_V2);
    body.push(32);
    body.extend_from_slice(&digest);
    let mut cid = String::with_capacity(1 + 60);
    cid.push(CID_MULTIBASE_BASE32_V2);
    cid.push_str(&base32_lower_encode(&body));
    Some(cid)
}

/// Verify that `data` is the exact block a `CIDv1(raw, sha2-256)` addresses.
///
/// This is the trustless-gateway seam: bytes fetched back by CID from any
/// gateway (or any untrusted peer) are checked locally against the digest
/// encoded in the CID itself, with no manifest needed. Returns `false` for
/// CIDs of any other shape — this deliberately understands exactly one CID
/// form rather than guessing.
pub fn verify_slice_against_cid(cid: &str, data: &[u8]) -> bool {
    let Some(encoded) = cid.strip_prefix(CID_MULTIBASE_BASE32_V2) else {
        return false;
    };
    let body = match base32_lower_decode(encoded.as_bytes()) {
        Some(bytes) => bytes,
        None => return false,
    };
    // version || codec || multihash code || multihash len || digest
    if body.len() != 4 + 32
        || body[0] != CID_VERSION_V2
        || body[1] != CID_CODEC_RAW_V2
        || body[2] != CID_SHA2_256_V2
        || body[3] != 32
    {
        return false;
    }
    sha256_hex(data) == hex::encode(&body[4..])
}

fn base32_lower_decode(encoded: &[u8]) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(encoded.len() * 5 / 8 + 1);
    let mut buffer: u32 = 0;
    let mut bits = 0u32;
    for &byte in encoded {
        let value = match byte {
            b'a'..=b'z' => byte - b'a',
            b'2'..=b'7' => byte - b'2' + 26,
            _ => return None,
        } as u32;
        buffer = (buffer << 5) | value;
        bits += 5;
        if bits >= 8 {
            bits -= 8;
            out.push((buffer >> bits) as u8);
        }
    }
    Some(out)
}

/// Packet header metadata shared by all packet types in the v2 tree.
///
/// Unlike v1, the header carries the payload length (`len`) so a packet is
/// self-describing and a receiver can sanity-check header against data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeaderV2 {
    pub seq_num: u32,
    pub total_packets: u32,
    /// Length of `data` in bytes. v1 headers carried no length field, so a
    /// receiver could not tell "peer does not have this slice" apart from "a
    /// zero-length leaf". v2 pairs this field with an explicit `Err` repair
    /// response on the wire; the placeholder ambiguity is gone.
    pub len: u32,
}

/// A single v2 packet produced by the recursive packetizer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolSliceV2 {
    pub id: String,
    pub header: HeaderV2,
    pub data: Vec<u8>,
    pub is_parity: bool,
}

/// Finalized v2 packet tree output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PacketBatchV2 {
    pub total_packets: u32,
    pub packets: Vec<ProtocolSliceV2>,
}

/// Manifest v2 entry (TODO.md plan 3).
///
/// Every slice is self-verifying by sha256, and byte order comes from
/// `(offset, len)` instead of `seq_num` (plan 5). `offset` is the byte offset
/// of the slice's payload within the original payload; parity slices reuse
/// their parent node's offset because they cover the whole pair.
///
/// Parity membership is derived from the `.P` id suffix, so tree root ids
/// must not end in `.P` unless they are parity frames (generated tree ids
/// never do; directory-walk roots are relative file paths, so a file named
/// `*.P` would be misclassified — same constraint as v1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestEntry {
    pub id: String,
    /// Byte offset of this slice's payload within the original payload.
    pub offset: u64,
    pub len: u64,
    pub sha256: String,
}

impl ManifestEntry {
    /// The `CIDv1(raw, sha2-256)` addressing this slice's data.
    ///
    /// Derived from the manifest sha256 (which is the raw multihash digest),
    /// so a fetcher needs only the manifest to pull any slice through an
    /// IPFS gateway by CID and verify it with [`verify_slice_against_cid`].
    /// Returns `None` only if the entry's sha256 is malformed, which
    /// generated manifests never produce.
    pub fn cid(&self) -> Option<String> {
        raw_leaf_cid_from_sha256(&self.sha256)
    }
}

/// A recorded slice that failed verification. Counted, never stored.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RejectedSlice {
    pub id: String,
    pub reason: String,
}

/// Outcome of an XOR repair pass.
///
/// The parity scheme recovers exactly one missing sibling per pair (TODO.md
/// finding 4). A pair that lost two or three of `{left, right, parity}` is
/// reported in `unrecoverable`; recovery never fabricates silent garbage.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RecoveryReport {
    pub recovered: Vec<String>,
    pub unrecoverable: Vec<String>,
}

/// Why a payload could not be reassembled from the recorded slices.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReconstructErrorV2 {
    /// One or more data leaves from the manifest were never recorded.
    MissingSlices(Vec<String>),
    /// The assembled payload length does not equal the sum of the manifest
    /// data-leaf lengths — the tree walk and the slices disagree.
    LengthMismatch { expected: u64, actual: u64 },
}

impl fmt::Display for ReconstructErrorV2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingSlices(ids) => write!(f, "missing slices: {}", ids.join(", ")),
            Self::LengthMismatch { expected, actual } => {
                write!(
                    f,
                    "length mismatch: expected {expected} bytes, assembled {actual}"
                )
            }
        }
    }
}

impl std::error::Error for ReconstructErrorV2 {}

/// Tracks the v2 manifest and the hash-verified packets received so far.
#[derive(Debug, Clone, Default)]
pub struct IntegrityManagerV2 {
    /// Manifest keyed by id. A `BTreeMap` keeps manifest iteration
    /// deterministic; reconstruction sorts by offset, not by this map.
    pub manifest: BTreeMap<String, ManifestEntry>,
    pub received_slices: HashMap<String, ProtocolSliceV2>,
    /// Slices that failed verification, counted and never stored.
    pub rejected: Vec<RejectedSlice>,
}

impl IntegrityManagerV2 {
    pub fn new(manifest: Vec<ManifestEntry>) -> Self {
        Self {
            manifest: manifest
                .into_iter()
                .map(|entry| (entry.id.clone(), entry))
                .collect(),
            received_slices: HashMap::new(),
            rejected: Vec::new(),
        }
    }

    /// Record a slice after verification (TODO.md plan 3).
    ///
    /// The id must be in the manifest, the header length must match the data
    /// length, and `sha256_hex(&slice.data)` must match the manifest entry.
    /// Any mismatch is pushed to [`IntegrityManagerV2::rejected`] and the
    /// slice is *not* stored.
    pub fn record_slice(&mut self, slice: ProtocolSliceV2) {
        let id = slice.id.clone();
        let Some(entry) = self.manifest.get(&id) else {
            self.rejected.push(RejectedSlice {
                id,
                reason: "id not in manifest".to_string(),
            });
            return;
        };
        if slice.header.len as usize != slice.data.len() || slice.header.len as u64 != entry.len {
            self.rejected.push(RejectedSlice {
                id,
                reason: format!(
                    "length mismatch: header len {}, data len {}, manifest len {}",
                    slice.header.len,
                    slice.data.len(),
                    entry.len
                ),
            });
            return;
        }
        let digest = sha256_hex(&slice.data);
        if digest != entry.sha256 {
            self.rejected.push(RejectedSlice {
                id,
                reason: format!("sha256 mismatch: got {digest}, expected {}", entry.sha256),
            });
            return;
        }
        self.received_slices.insert(id, slice);
    }

    /// Ids from the manifest that have not been recorded yet.
    pub fn get_missing_nodes(&self) -> Vec<String> {
        self.manifest
            .keys()
            .filter(|id| !self.received_slices.contains_key(*id))
            .cloned()
            .collect()
    }

    /// Verify every recorded slice against the manifest and check parity
    /// frames against their children when both are present.
    ///
    /// Fails closed: unknown ids, length inconsistencies, sha256 mismatches,
    /// and bad parities all return `false`.
    pub fn verify_integrity(&self) -> bool {
        for (id, slice) in &self.received_slices {
            let Some(entry) = self.manifest.get(id) else {
                return false;
            };
            if slice.header.len as usize != slice.data.len() || slice.header.len as u64 != entry.len
            {
                return false;
            }
            if sha256_hex(&slice.data) != entry.sha256 {
                return false;
            }
            if slice.is_parity != id.ends_with(".P") {
                return false;
            }
        }

        for parity_id in self.manifest.keys().filter(|id| id.ends_with(".P")) {
            let Some(parity) = self.received_slices.get(parity_id) else {
                continue;
            };
            let base = &parity_id[..parity_id.len() - 2];
            let (Some(left), Some(right)) = (
                self.received_slices.get(&format!("{base}.0")),
                self.received_slices.get(&format!("{base}.1")),
            ) else {
                continue;
            };
            if calculate_parity(&left.data, &right.data) != parity.data {
                return false;
            }
        }

        true
    }

    /// XOR repair pass over the manifest.
    ///
    /// Each parity frame covers exactly one sibling pair, so at most one loss
    /// per pair can be rebuilt; a pair missing two or three members is
    /// reported unrecoverable (TODO.md finding 4). Rebuilt slices go through
    /// [`IntegrityManagerV2::record_slice`], so a rebuild that fails hash
    /// verification is counted unrecoverable rather than stored.
    pub fn recover_missing(&mut self) -> RecoveryReport {
        let mut report = RecoveryReport::default();
        let parity_ids: Vec<String> = self
            .manifest
            .keys()
            .filter(|id| id.ends_with(".P"))
            .cloned()
            .collect();

        for parity_id in parity_ids {
            let base = &parity_id[..parity_id.len() - 2];
            let left_id = format!("{base}.0");
            let right_id = format!("{base}.1");
            // The parity frame covers the children's *payloads*. When a child
            // is itself an internal node it has no slice, so there is no
            // slice-level pair to repair at this level — deeper leaves are
            // covered by their own closer parities.
            let trio: Vec<String> = [left_id, right_id, parity_id.clone()]
                .into_iter()
                .filter(|id| self.manifest.contains_key(id))
                .collect();
            if trio.len() < 3 {
                continue;
            }
            let missing: Vec<String> = trio
                .iter()
                .filter(|id| !self.received_slices.contains_key(*id))
                .cloned()
                .collect();

            match missing.len() {
                0 => {}
                1 => {
                    let missing_id = missing[0].clone();
                    let total_packets = self.manifest.len() as u32;
                    let recovered = if missing_id == parity_id {
                        let left = self
                            .received_slices
                            .get(&format!("{base}.0"))
                            .expect("sibling present")
                            .clone();
                        let right = self
                            .received_slices
                            .get(&format!("{base}.1"))
                            .expect("sibling present")
                            .clone();
                        let data = calculate_parity(&left.data, &right.data);
                        ProtocolSliceV2 {
                            id: parity_id.clone(),
                            header: HeaderV2 {
                                seq_num: 0,
                                total_packets,
                                len: data.len() as u32,
                            },
                            data,
                            is_parity: true,
                        }
                    } else {
                        let sibling_id = if missing_id == format!("{base}.0") {
                            format!("{base}.1")
                        } else {
                            format!("{base}.0")
                        };
                        let sibling = self
                            .received_slices
                            .get(&sibling_id)
                            .expect("sibling present")
                            .clone();
                        let parity = self
                            .received_slices
                            .get(&parity_id)
                            .expect("parity present")
                            .clone();
                        let expected_len = self
                            .manifest
                            .get(&missing_id)
                            .map(|entry| entry.len as usize)
                            .unwrap_or(sibling.data.len());
                        let data: Vec<u8> = calculate_parity(&sibling.data, &parity.data)
                            .into_iter()
                            .take(expected_len)
                            .collect();
                        ProtocolSliceV2 {
                            id: missing_id.clone(),
                            header: HeaderV2 {
                                seq_num: 0,
                                total_packets,
                                len: data.len() as u32,
                            },
                            data,
                            is_parity: false,
                        }
                    };
                    self.record_slice(recovered);
                    if self.received_slices.contains_key(&missing_id) {
                        report.recovered.push(missing_id);
                    } else {
                        report.unrecoverable.push(missing_id);
                    }
                }
                _ => report.unrecoverable.extend(missing),
            }
        }

        report
    }

    /// Rebuild the payload from the recorded data leaves.
    ///
    /// Ordering comes from the manifest offsets (TODO.md plan 5) — `seq_num`
    /// is never consulted — and the assembled length must equal the sum of
    /// the data-leaf lengths.
    pub fn reconstruct_payload(&self) -> Result<Vec<u8>, ReconstructErrorV2> {
        let mut leaves: Vec<&ManifestEntry> = self
            .manifest
            .values()
            .filter(|entry| !entry.id.ends_with(".P"))
            .collect();
        leaves.sort_by(|a, b| (a.offset, &a.id).cmp(&(b.offset, &b.id)));

        let missing: Vec<String> = leaves
            .iter()
            .filter(|entry| !self.received_slices.contains_key(&entry.id))
            .map(|entry| entry.id.clone())
            .collect();
        if !missing.is_empty() {
            return Err(ReconstructErrorV2::MissingSlices(missing));
        }

        let expected: u64 = leaves.iter().map(|entry| entry.len).sum();
        let mut payload = Vec::with_capacity(expected as usize);
        for entry in leaves {
            let slice = self
                .received_slices
                .get(&entry.id)
                .expect("missing checked above");
            payload.extend_from_slice(&slice.data);
        }
        if payload.len() as u64 != expected {
            return Err(ReconstructErrorV2::LengthMismatch {
                expected,
                actual: payload.len() as u64,
            });
        }
        Ok(payload)
    }

    /// Persist the manager as versioned JSON (TODO.md plan 7).
    ///
    /// The envelope carries `"v": 1`; loaders refuse any version they do not
    /// know so the format can evolve without silent misreads.
    pub fn persist(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let state = PersistedIntegrityV2 {
            v: PERSIST_FORMAT_VERSION_V2,
            manifest: self.manifest.values().cloned().collect(),
            received_slices: self.received_slices.clone(),
        };
        let encoded = serde_json::to_vec_pretty(&state)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        std::fs::write(path, encoded)
    }

    /// Load a persisted manager and re-verify every slice.
    ///
    /// Unknown format versions are refused with a clear error, and slices that
    /// no longer match the manifest (tampered state) are routed through
    /// [`IntegrityManagerV2::record_slice`] and land in `rejected` instead of
    /// being trusted.
    pub fn load_from_disk(path: impl AsRef<Path>) -> io::Result<Self> {
        let bytes = std::fs::read(path)?;
        let state: PersistedIntegrityV2 = serde_json::from_slice(&bytes)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        if state.v != PERSIST_FORMAT_VERSION_V2 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "unsupported v2 persist format version {} (expected {})",
                    state.v, PERSIST_FORMAT_VERSION_V2
                ),
            ));
        }
        let mut manager = Self::new(state.manifest);
        for slice in state.received_slices.into_values() {
            manager.record_slice(slice);
        }
        Ok(manager)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct PersistedIntegrityV2 {
    v: u32,
    manifest: Vec<ManifestEntry>,
    received_slices: HashMap<String, ProtocolSliceV2>,
}

/// Combined output of the v2 packetizer: the wire batch plus its manifest.
#[derive(Debug, Clone)]
pub struct PacketizeV2Result {
    pub batch: PacketBatchV2,
    pub manifest: Vec<ManifestEntry>,
}

/// Recursively split a payload into MTU-safe v2 slices and parity frames,
/// recording each emitted slice's manifest entry.
fn walk_slice_v2(
    id: String,
    data: &[u8],
    seq: &mut u32,
    offset: u64,
    slices: &mut Vec<ProtocolSliceV2>,
    entries: &mut Vec<ManifestEntry>,
) {
    if data.len() <= MAX_LEAF_PAYLOAD_V2 {
        entries.push(ManifestEntry {
            id: id.clone(),
            offset,
            len: data.len() as u64,
            sha256: sha256_hex(data),
        });
        slices.push(ProtocolSliceV2 {
            id,
            header: HeaderV2 {
                seq_num: *seq,
                total_packets: 0,
                len: data.len() as u32,
            },
            data: data.to_vec(),
            is_parity: false,
        });
        *seq += 1;
        return;
    }

    let half = data.len() / 2;
    let (left, right) = data.split_at(half);
    let parity = calculate_parity(left, right);

    walk_slice_v2(format!("{id}.0"), left, seq, offset, slices, entries);
    walk_slice_v2(
        format!("{id}.1"),
        right,
        seq,
        offset + half as u64,
        slices,
        entries,
    );

    entries.push(ManifestEntry {
        id: format!("{id}.P"),
        offset,
        len: parity.len() as u64,
        sha256: sha256_hex(&parity),
    });
    slices.push(ProtocolSliceV2 {
        id: format!("{id}.P"),
        header: HeaderV2 {
            seq_num: *seq,
            total_packets: 0,
            len: parity.len() as u32,
        },
        data: parity,
        is_parity: true,
    });
    *seq += 1;
}

/// Generate the v2 manifest for a payload: one `{id, offset, len, sha256}`
/// entry per emitted slice, offsets derived from the same tree walk that
/// emits the ids (TODO.md plans 3 and 5).
pub fn generate_manifest_v2(id: String, data: &[u8]) -> Vec<ManifestEntry> {
    let mut seq = 0;
    let mut slices = Vec::new();
    let mut entries = Vec::new();
    walk_slice_v2(id, data, &mut seq, 0, &mut slices, &mut entries);
    entries
}

/// Packetize a payload and finalize the batch metadata, returning both the
/// wire batch and the self-verifying manifest.
pub fn packetize_v2(id: String, data: Vec<u8>) -> PacketizeV2Result {
    let mut seq = 0;
    let mut packets = Vec::new();
    let mut manifest = Vec::new();
    walk_slice_v2(id, &data, &mut seq, 0, &mut packets, &mut manifest);
    let total_packets = packets.len() as u32;
    for packet in &mut packets {
        packet.header.total_packets = total_packets;
    }
    PacketizeV2Result {
        batch: PacketBatchV2 {
            total_packets,
            packets,
        },
        manifest,
    }
}

/// Packets and manifest for a single file within a directory walk.
#[derive(Debug, Clone, Default)]
pub struct FileEntryV2 {
    pub packets: Vec<ProtocolSliceV2>,
    pub manifest: Vec<ManifestEntry>,
}

/// Result of packetizing a directory tree with the v2 data plane.
#[derive(Debug, Clone, Default)]
pub struct DirectoryWalkV2 {
    pub files: BTreeMap<String, FileEntryV2>,
    pub empty_dirs: BTreeSet<String>,
}

impl DirectoryWalkV2 {
    /// Flattened manifest across every file in the walk.
    pub fn manifest(&self) -> Vec<ManifestEntry> {
        self.files
            .values()
            .flat_map(|entry| entry.manifest.iter().cloned())
            .collect()
    }

    /// Total packet count across every file in the walk.
    pub fn total_packets(&self) -> usize {
        self.files.values().map(|entry| entry.packets.len()).sum()
    }
}

/// One-line-per-packet summary, mirroring v1's `summarize_packets`.
pub fn summarize_packets_v2(packets: &[ProtocolSliceV2]) -> Vec<String> {
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

/// Recursively list a directory tree and packetize every file found with the
/// v2 data plane. Empty directories are recorded in `empty_dirs`.
pub fn list_directory_v2(
    root: &Path,
    current: &Path,
    depth: usize,
    level: usize,
    logging: bool,
    lines: &mut Vec<String>,
    all_files: &mut BTreeMap<String, FileEntryV2>,
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
            let child_has_file = list_directory_v2(
                root,
                &path,
                depth,
                level + 1,
                logging,
                lines,
                all_files,
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
            let result = packetize_v2(rel_path.clone(), bytes);
            if logging {
                // trace suppresses debug: --logging trace must not emit
                // debug records (same rule as v1's list_directory).
                if log_enabled!(Level::Debug) && !log_enabled!(Level::Trace) {
                    debug!("File: {}", relative.display());
                    debug!("Batch size: {} packets", result.batch.total_packets);
                    debug!(
                        "{indent}  perfect_ip v2 packet batch: {} packets",
                        result.batch.total_packets
                    );
                }
                for line in summarize_packets_v2(&result.batch.packets) {
                    trace!("\n{line}");
                }
            }
            all_files.insert(
                rel_path,
                FileEntryV2 {
                    packets: result.batch.packets,
                    manifest: result.manifest,
                },
            );
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

/// Walk a directory and return all v2 packetized file slices grouped by path.
pub fn run_directory_walk_v2(
    path: &Path,
    depth: usize,
    logging: bool,
) -> io::Result<DirectoryWalkV2> {
    let mut lines = Vec::new();
    let mut files = BTreeMap::new();
    let mut empty_dirs = BTreeSet::new();
    list_directory_v2(
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
        // trace suppresses debug: tree lines surface only at debug level,
        // never at info — same adjusted rule as v1's run_directory_walk.
        if log_enabled!(Level::Debug) && !log_enabled!(Level::Trace) {
            debug!("{line}");
        }
    }
    Ok(DirectoryWalkV2 { files, empty_dirs })
}

/// Reconstruct a v2 packetized directory tree under `out_path`.
///
/// Each file is reassembled through an [`IntegrityManagerV2`], so every
/// packet is hash-verified against the manifest before it is written and a
/// missing or tampered packet aborts the reconstruction with an error.
pub fn reconstruct_directory_v2(walk: &DirectoryWalkV2, out_path: &Path) -> io::Result<()> {
    for (rel_path, entry) in &walk.files {
        let mut manager = IntegrityManagerV2::new(entry.manifest.clone());
        for packet in &entry.packets {
            manager.record_slice(packet.clone());
        }
        if !manager.rejected.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("rejected slices for {rel_path}: {:?}", manager.rejected),
            ));
        }
        let reconstructed = manager
            .reconstruct_payload()
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        let file_path = out_path.join(rel_path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&file_path, reconstructed)?;
    }

    for rel_path in &walk.empty_dirs {
        let dir_path = out_path.join(rel_path);
        fs::create_dir_all(dir_path)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_payload(len: usize) -> Vec<u8> {
        (0..len).map(|i| (i % 251) as u8).collect()
    }

    #[test]
    fn packetize_v2_manifest_matches_tree_and_offsets_are_ordered() {
        // 2500 bytes keeps every parity frame within MTU_PAYLOAD: the tree
        // only guarantees the parity ceiling while each half stays under the
        // transport limit (same constraint as v1).
        let payload = sample_payload(2500);
        let result = packetize_v2("ROOT".to_string(), payload.clone());

        assert_eq!(result.manifest.len(), result.batch.total_packets as usize);
        assert!(result.manifest.iter().any(|entry| entry.id == "ROOT.P"));
        assert!(
            result
                .manifest
                .iter()
                .any(|entry| entry.id.starts_with("ROOT.0."))
        );
        assert!(
            result
                .manifest
                .iter()
                .any(|entry| entry.id.starts_with("ROOT.1."))
        );
        assert!(
            result
                .batch
                .packets
                .iter()
                .all(|packet| packet.data.len() <= MTU_PAYLOAD)
        );
        assert!(
            result
                .batch
                .packets
                .iter()
                .all(|packet| packet.header.total_packets == result.batch.total_packets)
        );

        // Data leaves partition the payload: offsets strictly increasing and
        // contiguous, lengths sum to the payload length.
        let mut leaves: Vec<&ManifestEntry> = result
            .manifest
            .iter()
            .filter(|entry| !entry.id.ends_with(".P"))
            .collect();
        leaves.sort_by_key(|entry| entry.offset);
        let mut cursor = 0u64;
        for entry in &leaves {
            assert_eq!(entry.offset, cursor, "leaf {} is not contiguous", entry.id);
            cursor += entry.len;
        }
        assert_eq!(cursor, payload.len() as u64);

        // Every manifest hash matches its slice.
        for entry in &result.manifest {
            let slice = result
                .batch
                .packets
                .iter()
                .find(|packet| packet.id == entry.id)
                .expect("slice for manifest entry");
            assert_eq!(entry.sha256, sha256_hex(&slice.data));
            assert_eq!(entry.len, slice.data.len() as u64);
        }
    }

    #[test]
    fn record_slice_rejects_bad_hash_unknown_id_and_bad_length() {
        let payload = sample_payload(2000);
        let result = packetize_v2("ROOT".to_string(), payload);
        let mut manager = IntegrityManagerV2::new(result.manifest.clone());

        let mut corrupted = result.batch.packets[0].clone();
        corrupted.data[0] ^= 0xFF;
        let corrupted_id = corrupted.id.clone();
        manager.record_slice(corrupted);
        assert!(!manager.received_slices.contains_key(&corrupted_id));
        assert_eq!(manager.rejected.len(), 1);
        assert!(manager.rejected[0].reason.contains("sha256 mismatch"));

        manager.record_slice(ProtocolSliceV2 {
            id: "ROOT.unknown".to_string(),
            header: HeaderV2 {
                seq_num: 0,
                total_packets: 0,
                len: 1,
            },
            data: vec![0xAA],
            is_parity: false,
        });
        assert!(manager.rejected[1].reason.contains("not in manifest"));

        let mut bad_len = result.batch.packets[1].clone();
        bad_len.header.len = bad_len.data.len() as u32 + 1;
        let bad_len_id = bad_len.id.clone();
        manager.record_slice(bad_len);
        assert!(!manager.received_slices.contains_key(&bad_len_id));
        assert!(manager.rejected[2].reason.contains("length mismatch"));

        // Good slices still record fine.
        for packet in &result.batch.packets {
            manager.record_slice(packet.clone());
        }
        assert!(manager.verify_integrity());
    }

    #[test]
    fn verify_integrity_fails_when_slice_is_tampered_after_recording() {
        let payload = sample_payload(2000);
        let result = packetize_v2("ROOT".to_string(), payload);
        let mut manager = IntegrityManagerV2::new(result.manifest.clone());
        for packet in &result.batch.packets {
            manager.record_slice(packet.clone());
        }
        assert!(manager.verify_integrity());

        // Bypass record_slice to simulate post-verification corruption.
        manager
            .received_slices
            .get_mut("ROOT.P")
            .expect("parity slice")
            .data[0] ^= 0xFF;
        assert!(!manager.verify_integrity());
    }

    #[test]
    fn recover_missing_rebuilds_single_loss_byte_exactly() {
        let payload = sample_payload(3000);
        let result = packetize_v2("ROOT".to_string(), payload.clone());
        let mut manager = IntegrityManagerV2::new(result.manifest.clone());
        // Drop one data leaf; keep its sibling and the parity frames.
        let dropped = result
            .batch
            .packets
            .iter()
            .find(|packet| !packet.is_parity && packet.id != "ROOT.0" && packet.id != "ROOT.1")
            .expect("a nested data leaf")
            .id
            .clone();
        for packet in &result.batch.packets {
            if packet.id != dropped {
                manager.record_slice(packet.clone());
            }
        }
        assert!(manager.get_missing_nodes().contains(&dropped));

        let report = manager.recover_missing();
        assert_eq!(report.recovered, vec![dropped.clone()]);
        assert!(report.unrecoverable.is_empty());
        assert!(manager.rejected.is_empty());
        assert_eq!(manager.reconstruct_payload().expect("payload"), payload);
    }

    #[test]
    fn recover_missing_reports_unrecoverable_when_pair_loses_two() {
        // TODO.md plan 4: one loss per pair is the ceiling. Losing a sibling
        // AND its parity must report unrecoverable, not fabricate garbage.
        let payload = sample_payload(3000);
        let result = packetize_v2("ROOT".to_string(), payload.clone());
        let mut manager = IntegrityManagerV2::new(result.manifest.clone());

        // Pick a parity whose complete slice trio exists (both children are
        // leaves), then drop its left child and the parity itself.
        let pair_base = result
            .manifest
            .iter()
            .filter(|entry| entry.id.ends_with(".P"))
            .map(|entry| entry.id.trim_end_matches(".P").to_string())
            .find(|base| {
                result.manifest.iter().any(|e| e.id == format!("{base}.0"))
                    && result.manifest.iter().any(|e| e.id == format!("{base}.1"))
            })
            .expect("a complete slice trio");
        let left_id = format!("{pair_base}.0");
        let parity_id = format!("{pair_base}.P");
        for packet in &result.batch.packets {
            if packet.id != left_id && packet.id != parity_id {
                manager.record_slice(packet.clone());
            }
        }

        let report = manager.recover_missing();
        assert!(report.recovered.is_empty());
        assert!(report.unrecoverable.contains(&left_id));
        assert!(report.unrecoverable.contains(&parity_id));
        assert!(manager.reconstruct_payload().is_err());
    }

    #[test]
    fn reconstruct_orders_by_offset_regardless_of_arrival_order() {
        let payload = sample_payload(5000);
        let result = packetize_v2("ROOT".to_string(), payload.clone());
        let mut manager = IntegrityManagerV2::new(result.manifest.clone());

        // Record in reverse emission order; manifest offsets must win.
        for packet in result.batch.packets.iter().rev() {
            manager.record_slice(packet.clone());
        }
        let assembled = manager.reconstruct_payload().expect("payload");
        assert_eq!(assembled, payload);
    }

    #[test]
    fn reconstruct_fails_closed_with_missing_slices() {
        let payload = sample_payload(2000);
        let result = packetize_v2("ROOT".to_string(), payload);
        let mut manager = IntegrityManagerV2::new(result.manifest.clone());
        for packet in result.batch.packets.iter().take(2) {
            manager.record_slice(packet.clone());
        }
        match manager.reconstruct_payload() {
            Err(ReconstructErrorV2::MissingSlices(ids)) => assert!(!ids.is_empty()),
            other => panic!("expected MissingSlices, got {other:?}"),
        }
    }

    #[test]
    fn persist_round_trips_and_marks_version() {
        let payload = sample_payload(2000);
        let result = packetize_v2("ROOT".to_string(), payload);
        let mut manager = IntegrityManagerV2::new(result.manifest.clone());
        for packet in &result.batch.packets {
            manager.record_slice(packet.clone());
        }

        let temp = tempfile::NamedTempFile::new().expect("temp file");
        manager.persist(temp.path()).expect("persist");
        let raw = std::fs::read_to_string(temp.path()).expect("read back");
        assert!(raw.contains("\"v\": 1"));

        let loaded = IntegrityManagerV2::load_from_disk(temp.path()).expect("load");
        assert_eq!(loaded.received_slices.len(), manager.received_slices.len());
        assert_eq!(loaded.manifest.len(), manager.manifest.len());
        assert!(loaded.verify_integrity());
        assert!(loaded.rejected.is_empty());
    }

    #[test]
    fn load_from_disk_refuses_unknown_version() {
        let temp = tempfile::NamedTempFile::new().expect("temp file");
        std::fs::write(
            temp.path(),
            r#"{"v": 99, "manifest": [], "received_slices": {}}"#,
        )
        .expect("write");
        let error = IntegrityManagerV2::load_from_disk(temp.path()).expect_err("must refuse");
        assert!(
            error.to_string().contains("version 99"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn load_from_disk_rejects_tampered_persisted_state() {
        let payload = sample_payload(2000);
        let result = packetize_v2("ROOT".to_string(), payload);
        let mut manager = IntegrityManagerV2::new(result.manifest.clone());
        for packet in &result.batch.packets {
            manager.record_slice(packet.clone());
        }

        let temp = tempfile::NamedTempFile::new().expect("temp file");
        manager.persist(temp.path()).expect("persist");

        // Tamper deterministically: flip one data byte of the first slice in
        // the persisted JSON, keeping the envelope itself well-formed.
        let mut value: serde_json::Value =
            serde_json::from_slice(&std::fs::read(temp.path()).expect("read")).expect("json");
        let slices = value["received_slices"]
            .as_object_mut()
            .expect("received_slices object");
        let first_id = &result.batch.packets[0].id;
        let data = slices[first_id]["data"]
            .as_array_mut()
            .expect("slice data array");
        let byte = data[0].as_u64().expect("data byte") as u8;
        data[0] = serde_json::json!((byte ^ 0xFF) as u32);
        std::fs::write(temp.path(), serde_json::to_vec(&value).expect("encode")).expect("rewrite");

        let loaded = IntegrityManagerV2::load_from_disk(temp.path()).expect("load");
        assert!(
            !loaded.rejected.is_empty(),
            "tampered slice must be rejected on load"
        );
        assert!(!loaded.received_slices.contains_key(first_id));
    }

    #[test]
    fn protocol_slice_v2_json_round_trips() {
        let slice = ProtocolSliceV2 {
            id: "ROOT.0.P".to_string(),
            header: HeaderV2 {
                seq_num: 7,
                total_packets: 63,
                len: 4,
            },
            data: vec![0x00, 0xFF, 0x10, 0x20],
            is_parity: true,
        };
        let encoded = serde_json::to_string(&slice).expect("encode");
        assert!(encoded.contains("\"len\":4"));
        let decoded: ProtocolSliceV2 = serde_json::from_str(&encoded).expect("decode");
        assert_eq!(decoded, slice);
    }

    #[test]
    fn raw_leaf_cid_matches_known_vectors() {
        // Empty payload: the well-known CIDv1(raw, sha2-256) of the empty
        // block, independently reproduced from sha256(b"") = e3b0c442...
        let empty = sha256_hex(&[]);
        assert_eq!(
            raw_leaf_cid_from_sha256(&empty).expect("cid"),
            "bafkreihdwdcefgh4dqkjv67uzcmw7ojee6xedzdetojuzjevtenxquvyku"
        );
        // Independently computed vector for b"hello world".
        let hello = sha256_hex(b"hello world");
        assert_eq!(
            raw_leaf_cid_from_sha256(&hello).expect("cid"),
            "bafkreifzjut3te2nhyekklss27nh3k72ysco7y32koao5eei66wof36n5e"
        );

        // Malformed digests are refused, never guessed.
        assert!(raw_leaf_cid_from_sha256("").is_none());
        assert!(raw_leaf_cid_from_sha256("e3b0").is_none());
        assert!(raw_leaf_cid_from_sha256(&"ff".repeat(33)).is_none());
        assert!(raw_leaf_cid_from_sha256(&"zz".repeat(32)).is_none());
    }

    #[test]
    fn manifest_entry_cids_address_their_slices() {
        // The gateway-backfill invariant: for every slice, the CID derived
        // from the manifest entry addresses exactly the slice's bytes, and
        // verification in both directions holds.
        let payload = sample_payload(2500);
        let result = packetize_v2("ROOT".to_string(), payload);
        for entry in &result.manifest {
            let slice = result
                .batch
                .packets
                .iter()
                .find(|packet| packet.id == entry.id)
                .expect("slice for manifest entry");
            let cid = entry.cid().expect("entry cid");
            assert!(cid.starts_with("bafkrei"));
            assert!(
                verify_slice_against_cid(&cid, &slice.data),
                "cid {cid} must verify against slice {}",
                entry.id
            );
            // The same CID derived straight from the data agrees.
            let from_data = raw_leaf_cid_from_sha256(&sha256_hex(&slice.data)).expect("cid");
            assert_eq!(cid, from_data);
        }

        // A single flipped byte anywhere fails verification.
        let entry = &result.manifest[0];
        let cid = entry.cid().expect("entry cid");
        let mut tampered = result.batch.packets[0].data.clone();
        tampered[0] ^= 0xFF;
        assert!(!verify_slice_against_cid(&cid, &tampered));
    }

    #[test]
    fn verify_slice_against_cid_refuses_other_shapes() {
        let data = b"gateway backfill";
        let cid = raw_leaf_cid_from_sha256(&sha256_hex(data)).expect("cid");
        assert!(verify_slice_against_cid(&cid, data));

        // Uppercase multibase char, mutated header bytes, truncated digest:
        // all refused rather than guessed. (Char 1 spans the low bits of the
        // version byte and the high bits of the codec byte, so either
        // mutation breaks the CIDv1(raw, sha2-256) header check.)
        assert!(!verify_slice_against_cid(&cid.to_uppercase(), data));
        let mut wrong_version = cid.clone();
        wrong_version.replace_range(1..2, "c");
        assert!(!verify_slice_against_cid(&wrong_version, data));
        let mut wrong_codec = cid.clone();
        wrong_codec.replace_range(1..2, "g");
        assert!(!verify_slice_against_cid(&wrong_codec, data));
        assert!(!verify_slice_against_cid("bafkreishort", data));
        assert!(!verify_slice_against_cid("not-a-cid", data));
    }

    #[test]
    fn xor_repair_loss_rate_evaluation() {
        // TODO.md open issue: Reed-Solomon/RaptorQ evaluation for lossy
        // links. This is the measurement half — a deterministic Monte Carlo
        // (seeded PRNG, no flakiness) of the XOR tree's repair ceiling under
        // independent per-slice loss, tabulated against a Reed-Solomon
        // baseline at LOWER wire overhead (k + 25% parity vs the XOR tree's
        // ~100%). recover_missing gets exactly one pass, as on the wire.
        use rand::{Rng, SeedableRng};
        let mut rng = rand::rngs::StdRng::seed_from_u64(0xF7AC71A1);
        let sizes = [4_096usize, 65_536, 262_144];
        let rates = [0.01f64, 0.02, 0.05, 0.10, 0.20, 0.30];
        // Trials scale down as slice count grows so the suite stays fast;
        // the seeded PRNG keeps every number reproducible.
        let trials_for = |slices: usize| (60_000 / slices).clamp(40, 120);

        // P(Binom(n, p) <= k), summed iteratively to stay exact.
        fn binom_cdf_at_most(n: u64, p: f64, k: u64) -> f64 {
            if k >= n {
                return 1.0;
            }
            let mut term = (1.0 - p).powf(n as f64); // i = 0
            let mut sum = term;
            for i in 1..=k {
                term *= (n - (i - 1)) as f64 / i as f64 * p / (1.0 - p);
                sum += term;
            }
            sum.min(1.0)
        }

        println!(
            "{:>9} {:>6} {:>6} {:>10} {:>10}",
            "payload", "slices", "loss", "xor_ok", "rs25_th"
        );
        let mut previous_column: Vec<f64> = Vec::new();
        for (size_index, &size) in sizes.iter().enumerate() {
            let result = packetize_v2("ROOT".to_string(), sample_payload(size));
            let slice_count = result.manifest.len();
            let data_leaves = result
                .manifest
                .iter()
                .filter(|entry| !entry.id.ends_with(".P"))
                .count();
            // RS baseline: same data leaves, 25% parity, recovers iff at most
            // 25% of the RS codeword is lost — independent of loss pattern.
            let rs_parity = (data_leaves + 3) / 4;
            let trials = trials_for(slice_count);
            let mut column: Vec<f64> = Vec::new();
            for &rate in &rates {
                let mut recovered_payloads = 0usize;
                for _ in 0..trials {
                    let mut manager = IntegrityManagerV2::new(result.manifest.clone());
                    for packet in &result.batch.packets {
                        if rng.gen_range(0.0..1.0) >= rate {
                            manager.record_slice(packet.clone());
                        }
                    }
                    manager.recover_missing();
                    if manager.reconstruct_payload().is_ok() {
                        recovered_payloads += 1;
                    }
                }
                let xor_rate = recovered_payloads as f64 / trials as f64;
                let rs_rate =
                    binom_cdf_at_most((data_leaves + rs_parity) as u64, rate, rs_parity as u64);
                column.push(xor_rate);
                println!(
                    "{size:>9} {slice_count:>6} {:>5.0}% {xor_rate:>10.3} {rs_rate:>10.3}",
                    rate * 100.0
                );
                if size_index == sizes.len() - 1 {
                    // At 256 KiB (1023 slices) the XOR ceiling is
                    // structural: hundreds of independent leaf pairs make a
                    // clean sweep unlikely even at 5% loss.
                    if rate >= 0.05 {
                        assert!(
                            xor_rate < 0.2,
                            "XOR repair at {rate} loss over {slice_count} slices must collapse, got {xor_rate}"
                        );
                    }
                    // RS at 25% overhead tolerates up to 20% loss; certainty
                    // holds well below the threshold (10%), not at it.
                    if rate <= 0.10 {
                        assert!(
                            rs_rate > 0.99,
                            "RS(25%) at {rate} loss should be ~certain, got {rs_rate}"
                        );
                    }
                }
            }
            if !previous_column.is_empty() {
                // Success is monotone non-increasing in loss rate for a fixed
                // payload size (deterministic seed keeps this stable).
                assert!(
                    column.windows(2).all(|pair| pair[0] + 1e-9 >= pair[1]),
                    "success must not increase with loss rate"
                );
            }
            previous_column = column;
        }
    }

    #[test]
    fn repair_response_v2_result_shape_round_trips_in_json() {
        let ok: Result<ProtocolSliceV2, String> = Ok(ProtocolSliceV2 {
            id: "ROOT.0".to_string(),
            header: HeaderV2 {
                seq_num: 0,
                total_packets: 1,
                len: 3,
            },
            data: vec![1, 2, 3],
            is_parity: false,
        });
        let encoded = serde_json::to_string(&ok).expect("encode ok");
        let decoded: Result<ProtocolSliceV2, String> =
            serde_json::from_str(&encoded).expect("decode ok");
        assert_eq!(decoded, ok);

        let err: Result<ProtocolSliceV2, String> = Err("not held: ROOT.0".to_string());
        let encoded = serde_json::to_string(&err).expect("encode err");
        let decoded: Result<ProtocolSliceV2, String> =
            serde_json::from_str(&encoded).expect("decode err");
        assert_eq!(decoded, err);
    }
}
