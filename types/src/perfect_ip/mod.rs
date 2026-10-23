//! Primitive packet types for the `perfect_ip` protocol.
//!
//! These are the pure data types exchanged on the wire (serialized as JSON).
//! Networking/behaviour code lives in `gnostr-p2p`.

use serde::{Deserialize, Serialize};

/// Packet header metadata shared by all packet types in the tree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Header {
    /// Monotonic sequence number assigned during packetization.
    pub seq_num: u32,
    /// Total number of packets in the finalized batch.
    pub total_packets: u32,
}

/// A single packet produced by the recursive packetizer.
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PacketBatch {
    /// Number of packets in the batch.
    pub total_packets: u32,
    /// Finalized packets with matching `total_packets` headers.
    pub packets: Vec<ProtocolSlice>,
}