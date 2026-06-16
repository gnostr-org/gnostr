use serde::{Deserialize, Serialize};

/// Packet header metadata shared by all packet types in the tree (PIP).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PacketHeader {
    /// Monotonic sequence number assigned during packetization.
    pub seq_num: u64,
    /// Total number of packets in the finalized batch.
    pub total_packets: u64,
}

/// A single packet slice produced by the recursive packetizer (PIP).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolSlice {
    /// Stable recursive packet identifier.
    pub id: String,
    /// Packet sequencing metadata.
    pub header: PacketHeader,
    /// Raw payload bytes for the packet or parity frame.
    pub data: Vec<u8>,
    /// `true` when this slice is a parity frame.
    pub is_parity: bool,
}

/// A manifest event describing the whole packet tree (PIP).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PacketManifest {
    pub root: String,
    pub sha256: String,
    pub size: u64,
    pub packets: u64,
    pub depth: u32,
    pub mtu: u64,
    pub encoding: String,
    pub path: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_protocol_slice_serde() {
        let slice = ProtocolSlice {
            id: "ROOT.0.0.0.0.0".to_string(),
            header: PacketHeader {
                seq_num: 0,
                total_packets: 63,
            },
            data: vec![171, 171, 171],
            is_parity: false,
        };

        // Wire example from spec
        let expected_json = json!({
            "id":"ROOT.0.0.0.0.0",
            "header":{"seq_num":0,"total_packets":63},
            "data":[171,171,171],
            "is_parity":false
        });

        let json_val = serde_json::to_value(&slice).unwrap();
        assert_eq!(json_val, expected_json);

        let deserialized: ProtocolSlice = serde_json::from_value(json_val).unwrap();
        assert_eq!(slice, deserialized);
    }

    #[test]
    fn test_packet_manifest_serde() {
        let manifest = PacketManifest {
            root: "ROOT".to_string(),
            sha256: "50f3...00f4".to_string(),
            size: 3000,
            packets: 63,
            depth: 5,
            mtu: 1460,
            encoding: "json".to_string(),
            path: "docs/example.bin".to_string(),
        };

        // Wire example from spec
        let expected_json = json!({
            "root":"ROOT",
            "sha256":"50f3...00f4",
            "size":3000,
            "packets":63,
            "depth":5,
            "mtu":1460,
            "encoding":"json",
            "path":"docs/example.bin"
        });

        let json_val = serde_json::to_value(&manifest).unwrap();
        assert_eq!(json_val, expected_json);

        let deserialized: PacketManifest = serde_json::from_value(json_val).unwrap();
        assert_eq!(manifest, deserialized);
    }
}
