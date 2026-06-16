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
    use serial_test::serial;
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;

    #[test]
    #[serial]
    fn test_protocol_slice_serde() {
        println!("\n>>> START: test_protocol_slice_serde");
        let slice = ProtocolSlice {
            id: "ROOT.0.0.0.0.0".to_string(),
            header: PacketHeader { seq_num: 0, total_packets: 63 },
            data: vec![171, 171, 171],
            is_parity: false,
        };
        let expected_json = json!({"id":"ROOT.0.0.0.0.0","header":{"seq_num":0,"total_packets":63},"data":[171,171,171],"is_parity":false});
        let json_val = serde_json::to_value(&slice).unwrap();
        println!("Event JSON: {}", json_val);
        let deserialized: ProtocolSlice = serde_json::from_value(json_val).unwrap();
        assert_eq!(slice, deserialized);
        println!("<<< END: test_protocol_slice_serde\n");
    }

    #[test]
    #[serial]
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
        let expected_json = json!({"root":"ROOT","sha256":"50f3...00f4","size":3000,"packets":63,"depth":5,"mtu":1460,"encoding":"json","path":"docs/example.bin"});
        let json_val = serde_json::to_value(&manifest).unwrap();
        println!("Event JSON: {}", json_val);
        let deserialized: PacketManifest = serde_json::from_value(json_val).unwrap();
        assert_eq!(manifest, deserialized);
        println!("<<< END: test_packet_manifest_serde\n");
    }

    #[test]
    #[serial]
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
        let json_val = serde_json::to_value(&batch).unwrap();
        println!("Event JSON: {}", json_val);
        let deserialized: PacketBatch = serde_json::from_value(json_val).unwrap();
        assert_eq!(batch, deserialized);
        println!("<<< END: test_packet_batch_serde\n");
    }

    #[test]
    #[serial]
    fn test_calculate_parity() {
        println!("\n>>> START: test_calculate_parity");
        let left = [0xDE, 0xAD, 0xBE];
        let right = [0x01, 0x02, 0x03];
        let parity = calculate_parity(&left, &right);
        println!("Parity calculation: {:?} XOR {:?} = {:?}", left, right, parity);
        assert_eq!(parity, vec![0xDF, 0xAF, 0xBD]);
        assert_eq!(calculate_parity(&right, &parity), left);
        assert_eq!(calculate_parity(&left, &parity), right);
        println!("<<< END: test_calculate_parity\n");
    }

    #[test]
    #[serial]
    fn test_recursive_packetization() {
        println!("\n>>> START: test_recursive_packetization");
        fn recursive_process(id: String, data: Vec<u8>, seq: &mut u64) -> Vec<ProtocolSlice> {
            if data.len() <= 10 {
                let slice = ProtocolSlice { id: id.clone(), header: PacketHeader { seq_num: *seq, total_packets: 0 }, data, is_parity: false, };
                *seq += 1;
                return vec![slice];
            }
            let half = data.len() / 2;
            let left_data = data[..half].to_vec();
            let right_data = data[half..].to_vec();
            let parity_data = calculate_parity(&left_data, &right_data);
            let mut slices = recursive_process(format!("{}.0", id), left_data, seq);
            slices.append(&mut recursive_process(format!("{}.1", id), right_data, seq));
            slices.push(ProtocolSlice { id: format!("{}.P", id), header: PacketHeader { seq_num: *seq, total_packets: 0 }, data: parity_data, is_parity: true, });
            *seq += 1;
            slices
        }
        let data = vec![0xAB; 30];
        let mut seq = 0;
        let mut packets = recursive_process("ROOT".to_string(), data, &mut seq);
        let total = packets.len() as u64;
        for p in &mut packets { p.header.total_packets = total; }
        
        for (i, p) in packets.iter().enumerate() {
            println!("Packet {}: {}", i, serde_json::to_string(p).unwrap());
        }

        let batch = PacketBatch { total_packets: total, packets };
        println!("Batch JSON: {}", serde_json::to_string(&batch).unwrap());
        
        assert!(batch.total_packets == 7);
        println!("<<< END: test_recursive_packetization\n");
    }

    #[test]
    #[serial]
    fn test_large_file_packetization() {
        println!("\n>>> START: test_large_file_packetization");
        let large_data = vec![0xAB; 1024 * 10];
        let mut seq = 0;
        fn recursive_process(id: String, data: Vec<u8>, seq: &mut u64) -> Vec<ProtocolSlice> {
            if data.len() <= 1024 {
                let slice = ProtocolSlice { id: id.clone(), header: PacketHeader { seq_num: *seq, total_packets: 0 }, data, is_parity: false, };
                *seq += 1;
                return vec![slice];
            }
            let half = data.len() / 2;
            let left_data = data[..half].to_vec();
            let right_data = data[half..].to_vec();
            let parity_data = calculate_parity(&left_data, &right_data);
            let mut slices = recursive_process(format!("{}.0", id), left_data, seq);
            slices.append(&mut recursive_process(format!("{}.1", id), right_data, seq));
            slices.push(ProtocolSlice { id: format!("{}.P", id), header: PacketHeader { seq_num: *seq, total_packets: 0 }, data: parity_data, is_parity: true, });
            *seq += 1;
            slices
        }
        let mut packets = recursive_process("ROOT".to_string(), large_data, &mut seq);
        let total = packets.len() as u64;
        for p in &mut packets { p.header.total_packets = total; }
        
        for (i, p) in packets.iter().take(3).enumerate() {
            println!("Sample Packet {}: {}", i, serde_json::to_string(p).unwrap());
        }

        let batch = PacketBatch { total_packets: total, packets };
        assert!(batch.total_packets > 10);
        println!("<<< END: test_large_file_packetization\n");
    }

    #[test]
    #[serial]
    fn test_parity_recovery() {
        println!("\n>>> START: test_parity_recovery");
        let left_data = vec![0xDE, 0xAD, 0xBE];
        let right_data = vec![0x01, 0x02, 0x03];
        let parity_data = calculate_parity(&left_data, &right_data);
        let left = ProtocolSlice { id: "ROOT.0".to_string(), header: PacketHeader { seq_num: 0, total_packets: 3 }, data: left_data.clone(), is_parity: false, };
        let right = ProtocolSlice { id: "ROOT.1".to_string(), header: PacketHeader { seq_num: 1, total_packets: 3 }, data: right_data.clone(), is_parity: false, };
        let parity = ProtocolSlice { id: "ROOT.P".to_string(), header: PacketHeader { seq_num: 2, total_packets: 3 }, data: parity_data, is_parity: true, };
        
        println!("Recovering: Left={:?}, Right={:?}, Parity={:?}", left.data, right.data, parity.data);
        assert_eq!(calculate_parity(&right.data, &parity.data), left_data);
        assert_eq!(calculate_parity(&left.data, &parity.data), right_data);
        println!("<<< END: test_parity_recovery\n");
    }

    #[test]
    #[serial]
    fn test_real_pip_manifest_event() {
        use sha2::{Digest, Sha256};
        println!("\n>>> START: test_real_pip_manifest_event");
        let payload = vec![0xAB; 3000];
        let hash = Sha256::digest(&payload);
        let sha256_hex = format!("{:x}", hash);
        let manifest = PacketManifest {
            root: "ROOT".to_string(),
            sha256: sha256_hex.clone(),
            size: 3000,
            packets: 63,
            depth: 5,
            mtu: 1460,
            encoding: "json".to_string(),
            path: "docs/example.bin".to_string(),
        };
        let content = serde_json::to_string(&manifest).unwrap();
        let event = json!({
            "kind": 39078,
            "content": content,
            "tags": [
                ["d", "ROOT"],
                ["sha256", sha256_hex],
                ["size", "3000"],
                ["packets", "63"],
                ["depth", "5"],
                ["mtu", "1460"],
                ["encoding", "json"],
                ["path", "docs/example.bin"],
                ["t", "pip"],
                ["t", "manifest"]
            ]
        });
        println!("Event JSON: {}", event.to_string());
        assert_eq!(event["kind"], 39078);
        println!("<<< END: test_real_pip_manifest_event\n");
    }

    #[test]
    #[serial]
    fn test_packetize_git_repo_with_file_copy() {
        println!("\n>>> START: test_packetize_git_repo_with_file_copy");
        let temp_dir = std::env::temp_dir().join("gnostr_pip_test_repo_copy");
        if temp_dir.exists() { fs::remove_dir_all(&temp_dir).unwrap(); }
        fs::create_dir_all(&temp_dir).unwrap();
        let _ = Command::new("git").arg("init").current_dir(&temp_dir).status().unwrap();
        
        fs::write(temp_dir.join("README.md"), "# PIP Test Repo").unwrap();
        
        let pip_rs_path = PathBuf::from("src/nostr/pip.rs");
        let dest_path = temp_dir.join("pip.rs");
        let _ = fs::copy(&pip_rs_path, &dest_path).expect("Failed to copy pip.rs");

        let mut all_data = Vec::new();
        let mut entries: Vec<PathBuf> = fs::read_dir(&temp_dir).unwrap().filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.is_file()).collect();
        entries.sort();

        for entry in entries {
            let data = fs::read(&entry).unwrap();
            println!("Collected file: {:?}, size: {} bytes", entry.file_name(), data.len());
            all_data.extend(data);
        }
        
        // Recursive packetize
        fn recursive_process(id: String, data: Vec<u8>, seq: &mut u64) -> Vec<ProtocolSlice> {
            if data.len() <= 10 {
                let slice = ProtocolSlice { id: id.clone(), header: PacketHeader { seq_num: *seq, total_packets: 0 }, data, is_parity: false, };
                *seq += 1;
                return vec![slice];
            }
            let half = data.len() / 2;
            let left_data = data[..half].to_vec();
            let right_data = data[half..].to_vec();
            let parity_data = calculate_parity(&left_data, &right_data);
            let mut slices = recursive_process(format!("{}.0", id), left_data, seq);
            slices.append(&mut recursive_process(format!("{}.1", id), right_data, seq));
            slices.push(ProtocolSlice { id: format!("{}.P", id), header: PacketHeader { seq_num: *seq, total_packets: 0 }, data: parity_data, is_parity: true, });
            *seq += 1;
            slices
        }

        let mut seq = 0;
        let mut packets = recursive_process("ROOT".to_string(), all_data, &mut seq);
        let total = packets.len() as u64;
        for p in &mut packets { p.header.total_packets = total; }
        let batch = PacketBatch { total_packets: total, packets };
        println!("Git Repo Batch JSON: {}", serde_json::to_string(&batch).unwrap());
        
        assert!(batch.total_packets > 0);
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_git_repo_with_file_copy\n");
    }

    use crate::nostr::client::{Client, Options};
    use crate::nostr::event_builder::EventBuilder;
    use crate::nostr::event_kind::EventKind;
    use crate::nostr::Tag;
    use crate::nostr::keys::Keys;

    #[test]
    #[serial]
    fn test_packetize_and_broadcast_git_repo() {
        println!("\n>>> START: test_packetize_and_broadcast_git_repo");
        
        // 1. Setup temporary git repository
        let temp_dir = std::env::temp_dir().join("gnostr_git_test_clone");
        if temp_dir.exists() { fs::remove_dir_all(&temp_dir).unwrap(); }
        
        println!("Cloning repository...");
        let _ = Command::new("git").args(["clone", "https://github.com/gnostr-org/git-test.git", &temp_dir.to_string_lossy()]).status().unwrap();

        // 2. Mock packetization for broadcast
        let manifest = PacketManifest {
            root: "GIT-TEST-ROOT".to_string(),
            sha256: "fake-sha256".to_string(),
            size: 1024,
            packets: 1,
            depth: 1,
            mtu: 1460,
            encoding: "json".to_string(),
            path: "git-test".to_string(),
        };
        let content = serde_json::to_string(&manifest).unwrap();
        let tags = vec![Tag::new_identifier("GIT-TEST-ROOT")];
        
        // 3. Construct and broadcast
        let keys = crate::nostr::default_gnostr_private_key();
        let event = EventBuilder::new(EventKind::PipManifest, content, tags)
            .to_event(&keys.into_private_key())
            .unwrap();
        
        println!("Event to broadcast: {:?}", event);
        
        let client = Client::new(&Keys::new(keys.public_key()), Options::new());
        
        // Final live broadcast step (wrapped in tokio)
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let result = rt.block_on(client.send_event(event));
        println!("Broadcast result: {:?}", result);

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_and_broadcast_git_repo\n");
    }


    #[test]
    #[serial]
    fn test_packet_size_comparison() {
        println!("\n>>> START: test_packet_size_comparison");
        fn recursive_process(id: String, data: Vec<u8>, seq: &mut u64) -> Vec<ProtocolSlice> {
            if data.len() <= 1024 {
                let slice = ProtocolSlice { id: id.clone(), header: PacketHeader { seq_num: *seq, total_packets: 0 }, data, is_parity: false, };
                *seq += 1;
                return vec![slice];
            }
            let half = data.len() / 2;
            let left_data = data[..half].to_vec();
            let right_data = data[half..].to_vec();
            let parity_data = calculate_parity(&left_data, &right_data);
            let mut slices = recursive_process(format!("{}.0", id), left_data, seq);
            slices.append(&mut recursive_process(format!("{}.1", id), right_data, seq));
            slices.push(ProtocolSlice { id: format!("{}.P", id), header: PacketHeader { seq_num: *seq, total_packets: 0 }, data: parity_data, is_parity: true, });
            *seq += 1;
            slices
        }

        fn get_batch_json(data: Vec<u8>) -> String {
            let mut seq = 0;
            let packets = recursive_process("ROOT".to_string(), data, &mut seq);
            let total = packets.len() as u64;
            let batch = PacketBatch { total_packets: total, packets };
            serde_json::to_string(&batch).unwrap()
        }

        // Single file (1KB)
        let single_file_data = vec![0xAB; 1024];
        let single_json = get_batch_json(single_file_data);
        println!("Single file batch JSON: {}", single_json);
        println!("Single file batch size (JSON chars): {}", single_json.len());

        // Folder (10 files of 100 bytes)
        let mut folder_data = Vec::new();
        for _ in 0..10 { folder_data.extend(vec![0xAB; 100]); }
        let folder_json = get_batch_json(folder_data);
        println!("Folder batch JSON: {}", folder_json);
        println!("Folder batch size (JSON chars): {}", folder_json.len());

        assert!(single_json.len() > 0);
        assert!(folder_json.len() > 0);
        println!("<<< END: test_packet_size_comparison\n");
    }
}

