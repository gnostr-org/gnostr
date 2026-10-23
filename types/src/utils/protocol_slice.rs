//! Primitive packet types for the `perfect_ip` protocol.

use std::collections::{HashMap, HashSet};
use std::io;
use std::path::Path;

use bincode;
use serde::{Deserialize, Serialize};

use crate::utils::file_slice::FileSliceReader as _; // keep utils module linked (no-op)

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