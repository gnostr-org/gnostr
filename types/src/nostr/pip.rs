use serde::{Deserialize, Serialize};

/// Packet header metadata shared by all packet types in the tree (PIP).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

/// A finalized packet tree output (PIP).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PacketBatch {
    /// Number of packets in the batch.
    pub total_packets: u64,
    /// Finalized packets.
    pub packets: Vec<ProtocolSlice>,
}

/// XOR two payloads into a parity buffer (PIP helper).
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_protocol_slice_serde() {
        println!("\n>>> START: test_protocol_slice_serde");
        let slice = ProtocolSlice {
            id: "ROOT.0.0.0.0.0".to_string(),
            header: PacketHeader {
                seq_num: 0,
                total_packets: 63,
            },
            data: vec![171, 171, 171],
            is_parity: false,
        };

        println!("Testing ProtocolSlice: {:?}", slice);

        let expected_json = json!({
            "id":"ROOT.0.0.0.0.0",
            "header":{"seq_num":0,"total_packets":63},
            "data":[171,171,171],
            "is_parity":false
        });

        let json_val = serde_json::to_value(&slice).unwrap();
        println!("ProtocolSlice JSON: {}", json_val);
        assert_eq!(json_val, expected_json);

        let deserialized: ProtocolSlice = serde_json::from_value(json_val).unwrap();
        println!("Deserialized: {:?}", deserialized);
        assert_eq!(slice, deserialized);
        println!("<<< END: test_protocol_slice_serde\n");
    }

    #[test]
    fn test_packet_manifest_serde() {
        println!("\n>>> START: test_packet_manifest_serde");
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

        println!("Testing PacketManifest: {:?}", manifest);

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
        println!("PacketManifest JSON: {}", json_val);
        assert_eq!(json_val, expected_json);

        let deserialized: PacketManifest = serde_json::from_value(json_val).unwrap();
        println!("Deserialized: {:?}", deserialized);
        assert_eq!(manifest, deserialized);
        println!("<<< END: test_packet_manifest_serde\n");
    }

    #[test]
    fn test_packet_batch_serde() {
        println!("\n>>> START: test_packet_batch_serde");
        let batch = PacketBatch {
            total_packets: 1,
            packets: vec![ProtocolSlice {
                id: "ROOT".to_string(),
                header: PacketHeader { seq_num: 0, total_packets: 1 },
                data: vec![1, 2, 3],
                is_parity: false,
            }],
        };
        
        println!("Testing PacketBatch: {:?}", batch);

        let json_val = serde_json::to_value(&batch).unwrap();
        println!("PacketBatch JSON: {}", json_val);
        let deserialized: PacketBatch = serde_json::from_value(json_val).unwrap();
        println!("Deserialized: {:?}", deserialized);
        assert_eq!(batch, deserialized);
        println!("<<< END: test_packet_batch_serde\n");
    }

    #[test]
    fn test_calculate_parity() {
        println!("\n>>> START: test_calculate_parity");
        let left = [0xDE, 0xAD, 0xBE];
        let right = [0x01, 0x02, 0x03];
        println!("Parity Input - Left: {:?}, Right: {:?}", left, right);
        
        let parity = calculate_parity(&left, &right);
        println!("Calculated Parity: {:?}", parity);
        assert_eq!(parity, vec![0xDF, 0xAF, 0xBD]);
        
        // Recover one side
        let recovered_left = calculate_parity(&right, &parity);
        let recovered_right = calculate_parity(&left, &parity);
        println!("Recovered - Left: {:?}, Right: {:?}", recovered_left, recovered_right);
        assert_eq!(recovered_left, left);
        assert_eq!(recovered_right, right);
        println!("<<< END: test_calculate_parity\n");
    }

    #[test]
    fn test_recursive_packetization() {
        println!("\n>>> START: test_recursive_packetization");
        // ... (rest of recursive test)

        fn recursive_process(id: String, data: Vec<u8>, seq: &mut u64) -> Vec<ProtocolSlice> {
            if data.len() <= 10 {
                let slice = ProtocolSlice {
                    id: id.clone(),
                    header: PacketHeader { seq_num: *seq, total_packets: 0 },
                    data,
                    is_parity: false,
                };
                *seq += 1;
                return vec![slice];
            }

            let half = data.len() / 2;
            let left_data = data[..half].to_vec();
            let right_data = data[half..].to_vec();
            let parity_data = calculate_parity(&left_data, &right_data);

            let mut slices = recursive_process(format!("{}.0", id), left_data, seq);
            slices.append(&mut recursive_process(format!("{}.1", id), right_data, seq));
            
            slices.push(ProtocolSlice {
                id: format!("{}.P", id),
                header: PacketHeader { seq_num: *seq, total_packets: 0 },
                data: parity_data,
                is_parity: true,
            });
            *seq += 1;
            slices
        }

        let data = vec![0xAB; 30];
        let mut seq = 0;
        let mut packets = recursive_process("ROOT".to_string(), data, &mut seq);
        let total = packets.len() as u64;
        
        for p in &mut packets {
            p.header.total_packets = total;
        }

        let batch = PacketBatch { total_packets: total, packets };
        println!("Recursive Batch: {:?}", batch);
        
        assert!(batch.packets.iter().any(|p| p.is_parity));
        assert!(batch.packets.iter().any(|p| p.id == "ROOT.P"));
        assert_eq!(batch.total_packets, 7);
    }

    #[test]
    fn test_large_file_packetization() {
        let large_data = vec![0xAB; 1024 * 10]; // 10KB
        println!("Testing Large File Packetization: {} bytes", large_data.len());
        
        let mut seq = 0;
        fn recursive_process(id: String, data: Vec<u8>, seq: &mut u64) -> Vec<ProtocolSlice> {
            if data.len() <= 1024 {
                let slice = ProtocolSlice {
                    id: id.clone(),
                    header: PacketHeader { seq_num: *seq, total_packets: 0 },
                    data,
                    is_parity: false,
                };
                *seq += 1;
                return vec![slice];
            }
            let half = data.len() / 2;
            let left_data = data[..half].to_vec();
            let right_data = data[half..].to_vec();
            let parity_data = calculate_parity(&left_data, &right_data);
            let mut slices = recursive_process(format!("{}.0", id), left_data, seq);
            slices.append(&mut recursive_process(format!("{}.1", id), right_data, seq));
            slices.push(ProtocolSlice {
                id: format!("{}.P", id),
                header: PacketHeader { seq_num: *seq, total_packets: 0 },
                data: parity_data,
                is_parity: true,
            });
            *seq += 1;
            slices
        }

        let mut packets = recursive_process("ROOT".to_string(), large_data, &mut seq);
        let total = packets.len() as u64;
        for p in &mut packets {
            p.header.total_packets = total;
        }

        let batch = PacketBatch { total_packets: total, packets };
        println!("Large Batch: {} packets", batch.total_packets);
        assert!(batch.total_packets > 10);
    }

    #[test]
    fn test_parity_recovery() {
        let left_data = vec![0xDE, 0xAD, 0xBE];
        let right_data = vec![0x01, 0x02, 0x03];
        let parity_data = calculate_parity(&left_data, &right_data);

        let left = ProtocolSlice {
            id: "ROOT.0".to_string(),
            header: PacketHeader { seq_num: 0, total_packets: 3 },
            data: left_data.clone(),
            is_parity: false,
        };
        let right = ProtocolSlice {
            id: "ROOT.1".to_string(),
            header: PacketHeader { seq_num: 1, total_packets: 3 },
            data: right_data.clone(),
            is_parity: false,
        };
        let parity = ProtocolSlice {
            id: "ROOT.P".to_string(),
            header: PacketHeader { seq_num: 2, total_packets: 3 },
            data: parity_data,
            is_parity: true,
        };

        println!("Simulating Parity Recovery...");
        
        // Simulate missing left slice
        let recovered_left = calculate_parity(&right.data, &parity.data);
        println!("Recovered Left: {:?}", recovered_left);
        assert_eq!(recovered_left, left_data);

        // Simulate missing right slice
        let recovered_right = calculate_parity(&left.data, &parity.data);
        println!("Recovered Right: {:?}", recovered_right);
        assert_eq!(recovered_right, right_data);
    }
}
