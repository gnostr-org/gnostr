use serde::{Deserialize, Serialize};

/// Packet header metadata shared by all packet types in the tree (PIP).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PacketHeader {
    /// Monotonic sequence number assigned during packetization.
    pub seq_num: u64,
    /// Total number of packets in the finalized batch.
    pub total_packets: u64,

    #[test]
    fn test_packetize_git_repo_with_file_copy() {
        println!("\n>>> START: test_packetize_git_repo_with_file_copy");
        
        // 1. Setup temporary git repository
        let temp_dir = std::env::temp_dir().join("gnostr_pip_test_repo_copy");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        fs::create_dir_all(&temp_dir).unwrap();

        let _ = Command::new("git")
            .arg("init")
            .current_dir(&temp_dir)
            .status()
            .unwrap();
        
        // 2. Copy pip.rs into the repo
        let pip_rs_path = PathBuf::from("types/src/nostr/pip.rs");
        let dest_path = temp_dir.join("pip.rs");
        fs::copy(&pip_rs_path, &dest_path).unwrap();

        // 3. Walk directory and collect bytes
        let mut all_data = Vec::new();
        let mut entries: Vec<PathBuf> = fs::read_dir(&temp_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .collect();
        entries.sort();

        for entry in entries {
            let data = fs::read(&entry).unwrap();
            println!("Collected file: {:?}, size: {} bytes", entry.file_name(), data.len());
            all_data.extend(data);
        }

        // 4. Recursive packetize
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

        let mut seq = 0;
        let mut packets = recursive_process("ROOT".to_string(), all_data, &mut seq);
        let total = packets.len() as u64;
        
        for p in &mut packets {
            p.header.total_packets = total;
        }

        let batch = PacketBatch { total_packets: total, packets };
        println!("Git Repo Batch (with pip.rs copy): {} packets", batch.total_packets);

        // 5. Verify
        assert!(batch.total_packets > 0);
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_git_repo_with_file_copy\n");
    }

    #[test]
    fn test_packetize_and_broadcast_git_repo() {
        println!("\n>>> START: test_packetize_and_broadcast_git_repo");
        
        // 1. Clone repo
        let temp_dir = std::env::temp_dir().join("gnostr_git_test_clone");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        
        println!("Cloning repository...");
        let _ = Command::new("git")
            .args(["clone", "https://github.com/gnostr-org/git-test.git", &temp_dir.to_string_lossy()])
            .status()
            .unwrap();

        // 2. Packetize
        // Note: Packetization logic would be same as above.
        // For now, demonstrate preparation for broadcast.

        println!("Git Test Repo cloned and ready for packetization.");
        
        // This is where you would call the broadcast mechanism if approved.
        println!("Broadcasting not implemented: Awaiting explicit command.");
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_and_broadcast_git_repo\n");
    }
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

    #[test]
    fn test_packetize_git_repo_with_file_copy() {
        println!("\n>>> START: test_packetize_git_repo_with_file_copy");
        
        // 1. Setup temporary git repository
        let temp_dir = std::env::temp_dir().join("gnostr_pip_test_repo_copy");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        fs::create_dir_all(&temp_dir).unwrap();

        let _ = Command::new("git")
            .arg("init")
            .current_dir(&temp_dir)
            .status()
            .unwrap();
        
        // 2. Copy pip.rs into the repo
        let pip_rs_path = PathBuf::from("types/src/nostr/pip.rs");
        let dest_path = temp_dir.join("pip.rs");
        fs::copy(&pip_rs_path, &dest_path).unwrap();

        // 3. Walk directory and collect bytes
        let mut all_data = Vec::new();
        let mut entries: Vec<PathBuf> = fs::read_dir(&temp_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .collect();
        entries.sort();

        for entry in entries {
            let data = fs::read(&entry).unwrap();
            println!("Collected file: {:?}, size: {} bytes", entry.file_name(), data.len());
            all_data.extend(data);
        }

        // 4. Recursive packetize
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

        let mut seq = 0;
        let mut packets = recursive_process("ROOT".to_string(), all_data, &mut seq);
        let total = packets.len() as u64;
        
        for p in &mut packets {
            p.header.total_packets = total;
        }

        let batch = PacketBatch { total_packets: total, packets };
        println!("Git Repo Batch (with pip.rs copy): {} packets", batch.total_packets);

        // 5. Verify
        assert!(batch.total_packets > 0);
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_git_repo_with_file_copy\n");
    }

    #[test]
    fn test_packetize_and_broadcast_git_repo() {
        println!("\n>>> START: test_packetize_and_broadcast_git_repo");
        
        // 1. Clone repo
        let temp_dir = std::env::temp_dir().join("gnostr_git_test_clone");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        
        println!("Cloning repository...");
        let _ = Command::new("git")
            .args(["clone", "https://github.com/gnostr-org/git-test.git", &temp_dir.to_string_lossy()])
            .status()
            .unwrap();

        // 2. Packetize
        // Note: Packetization logic would be same as above.
        // For now, demonstrate preparation for broadcast.

        println!("Git Test Repo cloned and ready for packetization.");
        
        // This is where you would call the broadcast mechanism if approved.
        println!("Broadcasting not implemented: Awaiting explicit command.");
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_and_broadcast_git_repo\n");
    }
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

    #[test]
    fn test_packetize_git_repo_with_file_copy() {
        println!("\n>>> START: test_packetize_git_repo_with_file_copy");
        
        // 1. Setup temporary git repository
        let temp_dir = std::env::temp_dir().join("gnostr_pip_test_repo_copy");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        fs::create_dir_all(&temp_dir).unwrap();

        let _ = Command::new("git")
            .arg("init")
            .current_dir(&temp_dir)
            .status()
            .unwrap();
        
        // 2. Copy pip.rs into the repo
        let pip_rs_path = PathBuf::from("types/src/nostr/pip.rs");
        let dest_path = temp_dir.join("pip.rs");
        fs::copy(&pip_rs_path, &dest_path).unwrap();

        // 3. Walk directory and collect bytes
        let mut all_data = Vec::new();
        let mut entries: Vec<PathBuf> = fs::read_dir(&temp_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .collect();
        entries.sort();

        for entry in entries {
            let data = fs::read(&entry).unwrap();
            println!("Collected file: {:?}, size: {} bytes", entry.file_name(), data.len());
            all_data.extend(data);
        }

        // 4. Recursive packetize
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

        let mut seq = 0;
        let mut packets = recursive_process("ROOT".to_string(), all_data, &mut seq);
        let total = packets.len() as u64;
        
        for p in &mut packets {
            p.header.total_packets = total;
        }

        let batch = PacketBatch { total_packets: total, packets };
        println!("Git Repo Batch (with pip.rs copy): {} packets", batch.total_packets);

        // 5. Verify
        assert!(batch.total_packets > 0);
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_git_repo_with_file_copy\n");
    }

    #[test]
    fn test_packetize_and_broadcast_git_repo() {
        println!("\n>>> START: test_packetize_and_broadcast_git_repo");
        
        // 1. Clone repo
        let temp_dir = std::env::temp_dir().join("gnostr_git_test_clone");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        
        println!("Cloning repository...");
        let _ = Command::new("git")
            .args(["clone", "https://github.com/gnostr-org/git-test.git", &temp_dir.to_string_lossy()])
            .status()
            .unwrap();

        // 2. Packetize
        // Note: Packetization logic would be same as above.
        // For now, demonstrate preparation for broadcast.

        println!("Git Test Repo cloned and ready for packetization.");
        
        // This is where you would call the broadcast mechanism if approved.
        println!("Broadcasting not implemented: Awaiting explicit command.");
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_and_broadcast_git_repo\n");
    }
}

/// A finalized packet tree output (PIP).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PacketBatch {
    /// Number of packets in the batch.
    pub total_packets: u64,
    /// Finalized packets.
    pub packets: Vec<ProtocolSlice>,

    #[test]
    fn test_packetize_git_repo_with_file_copy() {
        println!("\n>>> START: test_packetize_git_repo_with_file_copy");
        
        // 1. Setup temporary git repository
        let temp_dir = std::env::temp_dir().join("gnostr_pip_test_repo_copy");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        fs::create_dir_all(&temp_dir).unwrap();

        let _ = Command::new("git")
            .arg("init")
            .current_dir(&temp_dir)
            .status()
            .unwrap();
        
        // 2. Copy pip.rs into the repo
        let pip_rs_path = PathBuf::from("types/src/nostr/pip.rs");
        let dest_path = temp_dir.join("pip.rs");
        fs::copy(&pip_rs_path, &dest_path).unwrap();

        // 3. Walk directory and collect bytes
        let mut all_data = Vec::new();
        let mut entries: Vec<PathBuf> = fs::read_dir(&temp_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .collect();
        entries.sort();

        for entry in entries {
            let data = fs::read(&entry).unwrap();
            println!("Collected file: {:?}, size: {} bytes", entry.file_name(), data.len());
            all_data.extend(data);
        }

        // 4. Recursive packetize
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

        let mut seq = 0;
        let mut packets = recursive_process("ROOT".to_string(), all_data, &mut seq);
        let total = packets.len() as u64;
        
        for p in &mut packets {
            p.header.total_packets = total;
        }

        let batch = PacketBatch { total_packets: total, packets };
        println!("Git Repo Batch (with pip.rs copy): {} packets", batch.total_packets);

        // 5. Verify
        assert!(batch.total_packets > 0);
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_git_repo_with_file_copy\n");
    }

    #[test]
    fn test_packetize_and_broadcast_git_repo() {
        println!("\n>>> START: test_packetize_and_broadcast_git_repo");
        
        // 1. Clone repo
        let temp_dir = std::env::temp_dir().join("gnostr_git_test_clone");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        
        println!("Cloning repository...");
        let _ = Command::new("git")
            .args(["clone", "https://github.com/gnostr-org/git-test.git", &temp_dir.to_string_lossy()])
            .status()
            .unwrap();

        // 2. Packetize
        // Note: Packetization logic would be same as above.
        // For now, demonstrate preparation for broadcast.

        println!("Git Test Repo cloned and ready for packetization.");
        
        // This is where you would call the broadcast mechanism if approved.
        println!("Broadcasting not implemented: Awaiting explicit command.");
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_and_broadcast_git_repo\n");
    }
}

/// XOR two payloads into a parity buffer (PIP helper).
pub fn calculate_parity(left: &[u8], right: &[u8]) -> Vec<u8> {
    let max_len = left.len().max(right.len());
    let mut parity = vec![0; max_len];
    for i in 0..max_len {
        let l = if i < left.len() { left[i] } else { 0 };
        let r = if i < right.len() { right[i] } else { 0 };
        parity[i] = l ^ r;
    
    #[test]
    fn test_packetize_git_repo_with_file_copy() {
        println!("\n>>> START: test_packetize_git_repo_with_file_copy");
        
        // 1. Setup temporary git repository
        let temp_dir = std::env::temp_dir().join("gnostr_pip_test_repo_copy");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        fs::create_dir_all(&temp_dir).unwrap();

        let _ = Command::new("git")
            .arg("init")
            .current_dir(&temp_dir)
            .status()
            .unwrap();
        
        // 2. Copy pip.rs into the repo
        let pip_rs_path = PathBuf::from("types/src/nostr/pip.rs");
        let dest_path = temp_dir.join("pip.rs");
        fs::copy(&pip_rs_path, &dest_path).unwrap();

        // 3. Walk directory and collect bytes
        let mut all_data = Vec::new();
        let mut entries: Vec<PathBuf> = fs::read_dir(&temp_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .collect();
        entries.sort();

        for entry in entries {
            let data = fs::read(&entry).unwrap();
            println!("Collected file: {:?}, size: {} bytes", entry.file_name(), data.len());
            all_data.extend(data);
        }

        // 4. Recursive packetize
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

        let mut seq = 0;
        let mut packets = recursive_process("ROOT".to_string(), all_data, &mut seq);
        let total = packets.len() as u64;
        
        for p in &mut packets {
            p.header.total_packets = total;
        }

        let batch = PacketBatch { total_packets: total, packets };
        println!("Git Repo Batch (with pip.rs copy): {} packets", batch.total_packets);

        // 5. Verify
        assert!(batch.total_packets > 0);
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_git_repo_with_file_copy\n");
    }

    #[test]
    fn test_packetize_and_broadcast_git_repo() {
        println!("\n>>> START: test_packetize_and_broadcast_git_repo");
        
        // 1. Clone repo
        let temp_dir = std::env::temp_dir().join("gnostr_git_test_clone");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        
        println!("Cloning repository...");
        let _ = Command::new("git")
            .args(["clone", "https://github.com/gnostr-org/git-test.git", &temp_dir.to_string_lossy()])
            .status()
            .unwrap();

        // 2. Packetize
        // Note: Packetization logic would be same as above.
        // For now, demonstrate preparation for broadcast.

        println!("Git Test Repo cloned and ready for packetization.");
        
        // This is where you would call the broadcast mechanism if approved.
        println!("Broadcasting not implemented: Awaiting explicit command.");
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_and_broadcast_git_repo\n");
    }
}
    parity

    #[test]
    fn test_packetize_git_repo_with_file_copy() {
        println!("\n>>> START: test_packetize_git_repo_with_file_copy");
        
        // 1. Setup temporary git repository
        let temp_dir = std::env::temp_dir().join("gnostr_pip_test_repo_copy");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        fs::create_dir_all(&temp_dir).unwrap();

        let _ = Command::new("git")
            .arg("init")
            .current_dir(&temp_dir)
            .status()
            .unwrap();
        
        // 2. Copy pip.rs into the repo
        let pip_rs_path = PathBuf::from("types/src/nostr/pip.rs");
        let dest_path = temp_dir.join("pip.rs");
        fs::copy(&pip_rs_path, &dest_path).unwrap();

        // 3. Walk directory and collect bytes
        let mut all_data = Vec::new();
        let mut entries: Vec<PathBuf> = fs::read_dir(&temp_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .collect();
        entries.sort();

        for entry in entries {
            let data = fs::read(&entry).unwrap();
            println!("Collected file: {:?}, size: {} bytes", entry.file_name(), data.len());
            all_data.extend(data);
        }

        // 4. Recursive packetize
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

        let mut seq = 0;
        let mut packets = recursive_process("ROOT".to_string(), all_data, &mut seq);
        let total = packets.len() as u64;
        
        for p in &mut packets {
            p.header.total_packets = total;
        }

        let batch = PacketBatch { total_packets: total, packets };
        println!("Git Repo Batch (with pip.rs copy): {} packets", batch.total_packets);

        // 5. Verify
        assert!(batch.total_packets > 0);
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_git_repo_with_file_copy\n");
    }

    #[test]
    fn test_packetize_and_broadcast_git_repo() {
        println!("\n>>> START: test_packetize_and_broadcast_git_repo");
        
        // 1. Clone repo
        let temp_dir = std::env::temp_dir().join("gnostr_git_test_clone");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        
        println!("Cloning repository...");
        let _ = Command::new("git")
            .args(["clone", "https://github.com/gnostr-org/git-test.git", &temp_dir.to_string_lossy()])
            .status()
            .unwrap();

        // 2. Packetize
        // Note: Packetization logic would be same as above.
        // For now, demonstrate preparation for broadcast.

        println!("Git Test Repo cloned and ready for packetization.");
        
        // This is where you would call the broadcast mechanism if approved.
        println!("Broadcasting not implemented: Awaiting explicit command.");
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_and_broadcast_git_repo\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;

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
    
    #[test]
    fn test_packetize_git_repo_with_file_copy() {
        println!("\n>>> START: test_packetize_git_repo_with_file_copy");
        
        // 1. Setup temporary git repository
        let temp_dir = std::env::temp_dir().join("gnostr_pip_test_repo_copy");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        fs::create_dir_all(&temp_dir).unwrap();

        let _ = Command::new("git")
            .arg("init")
            .current_dir(&temp_dir)
            .status()
            .unwrap();
        
        // 2. Copy pip.rs into the repo
        let pip_rs_path = PathBuf::from("types/src/nostr/pip.rs");
        let dest_path = temp_dir.join("pip.rs");
        fs::copy(&pip_rs_path, &dest_path).unwrap();

        // 3. Walk directory and collect bytes
        let mut all_data = Vec::new();
        let mut entries: Vec<PathBuf> = fs::read_dir(&temp_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .collect();
        entries.sort();

        for entry in entries {
            let data = fs::read(&entry).unwrap();
            println!("Collected file: {:?}, size: {} bytes", entry.file_name(), data.len());
            all_data.extend(data);
        }

        // 4. Recursive packetize
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

        let mut seq = 0;
        let mut packets = recursive_process("ROOT".to_string(), all_data, &mut seq);
        let total = packets.len() as u64;
        
        for p in &mut packets {
            p.header.total_packets = total;
        }

        let batch = PacketBatch { total_packets: total, packets };
        println!("Git Repo Batch (with pip.rs copy): {} packets", batch.total_packets);

        // 5. Verify
        assert!(batch.total_packets > 0);
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_git_repo_with_file_copy\n");
    }

    #[test]
    fn test_packetize_and_broadcast_git_repo() {
        println!("\n>>> START: test_packetize_and_broadcast_git_repo");
        
        // 1. Clone repo
        let temp_dir = std::env::temp_dir().join("gnostr_git_test_clone");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        
        println!("Cloning repository...");
        let _ = Command::new("git")
            .args(["clone", "https://github.com/gnostr-org/git-test.git", &temp_dir.to_string_lossy()])
            .status()
            .unwrap();

        // 2. Packetize
        // Note: Packetization logic would be same as above.
        // For now, demonstrate preparation for broadcast.

        println!("Git Test Repo cloned and ready for packetization.");
        
        // This is where you would call the broadcast mechanism if approved.
        println!("Broadcasting not implemented: Awaiting explicit command.");
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_and_broadcast_git_repo\n");
    }
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
    
    #[test]
    fn test_packetize_git_repo_with_file_copy() {
        println!("\n>>> START: test_packetize_git_repo_with_file_copy");
        
        // 1. Setup temporary git repository
        let temp_dir = std::env::temp_dir().join("gnostr_pip_test_repo_copy");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        fs::create_dir_all(&temp_dir).unwrap();

        let _ = Command::new("git")
            .arg("init")
            .current_dir(&temp_dir)
            .status()
            .unwrap();
        
        // 2. Copy pip.rs into the repo
        let pip_rs_path = PathBuf::from("types/src/nostr/pip.rs");
        let dest_path = temp_dir.join("pip.rs");
        fs::copy(&pip_rs_path, &dest_path).unwrap();

        // 3. Walk directory and collect bytes
        let mut all_data = Vec::new();
        let mut entries: Vec<PathBuf> = fs::read_dir(&temp_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .collect();
        entries.sort();

        for entry in entries {
            let data = fs::read(&entry).unwrap();
            println!("Collected file: {:?}, size: {} bytes", entry.file_name(), data.len());
            all_data.extend(data);
        }

        // 4. Recursive packetize
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

        let mut seq = 0;
        let mut packets = recursive_process("ROOT".to_string(), all_data, &mut seq);
        let total = packets.len() as u64;
        
        for p in &mut packets {
            p.header.total_packets = total;
        }

        let batch = PacketBatch { total_packets: total, packets };
        println!("Git Repo Batch (with pip.rs copy): {} packets", batch.total_packets);

        // 5. Verify
        assert!(batch.total_packets > 0);
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_git_repo_with_file_copy\n");
    }

    #[test]
    fn test_packetize_and_broadcast_git_repo() {
        println!("\n>>> START: test_packetize_and_broadcast_git_repo");
        
        // 1. Clone repo
        let temp_dir = std::env::temp_dir().join("gnostr_git_test_clone");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        
        println!("Cloning repository...");
        let _ = Command::new("git")
            .args(["clone", "https://github.com/gnostr-org/git-test.git", &temp_dir.to_string_lossy()])
            .status()
            .unwrap();

        // 2. Packetize
        // Note: Packetization logic would be same as above.
        // For now, demonstrate preparation for broadcast.

        println!("Git Test Repo cloned and ready for packetization.");
        
        // This is where you would call the broadcast mechanism if approved.
        println!("Broadcasting not implemented: Awaiting explicit command.");
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_and_broadcast_git_repo\n");
    }
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
    
    #[test]
    fn test_packetize_git_repo_with_file_copy() {
        println!("\n>>> START: test_packetize_git_repo_with_file_copy");
        
        // 1. Setup temporary git repository
        let temp_dir = std::env::temp_dir().join("gnostr_pip_test_repo_copy");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        fs::create_dir_all(&temp_dir).unwrap();

        let _ = Command::new("git")
            .arg("init")
            .current_dir(&temp_dir)
            .status()
            .unwrap();
        
        // 2. Copy pip.rs into the repo
        let pip_rs_path = PathBuf::from("types/src/nostr/pip.rs");
        let dest_path = temp_dir.join("pip.rs");
        fs::copy(&pip_rs_path, &dest_path).unwrap();

        // 3. Walk directory and collect bytes
        let mut all_data = Vec::new();
        let mut entries: Vec<PathBuf> = fs::read_dir(&temp_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .collect();
        entries.sort();

        for entry in entries {
            let data = fs::read(&entry).unwrap();
            println!("Collected file: {:?}, size: {} bytes", entry.file_name(), data.len());
            all_data.extend(data);
        }

        // 4. Recursive packetize
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

        let mut seq = 0;
        let mut packets = recursive_process("ROOT".to_string(), all_data, &mut seq);
        let total = packets.len() as u64;
        
        for p in &mut packets {
            p.header.total_packets = total;
        }

        let batch = PacketBatch { total_packets: total, packets };
        println!("Git Repo Batch (with pip.rs copy): {} packets", batch.total_packets);

        // 5. Verify
        assert!(batch.total_packets > 0);
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_git_repo_with_file_copy\n");
    }

    #[test]
    fn test_packetize_and_broadcast_git_repo() {
        println!("\n>>> START: test_packetize_and_broadcast_git_repo");
        
        // 1. Clone repo
        let temp_dir = std::env::temp_dir().join("gnostr_git_test_clone");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        
        println!("Cloning repository...");
        let _ = Command::new("git")
            .args(["clone", "https://github.com/gnostr-org/git-test.git", &temp_dir.to_string_lossy()])
            .status()
            .unwrap();

        // 2. Packetize
        // Note: Packetization logic would be same as above.
        // For now, demonstrate preparation for broadcast.

        println!("Git Test Repo cloned and ready for packetization.");
        
        // This is where you would call the broadcast mechanism if approved.
        println!("Broadcasting not implemented: Awaiting explicit command.");
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_and_broadcast_git_repo\n");
    }
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
    
    #[test]
    fn test_packetize_git_repo_with_file_copy() {
        println!("\n>>> START: test_packetize_git_repo_with_file_copy");
        
        // 1. Setup temporary git repository
        let temp_dir = std::env::temp_dir().join("gnostr_pip_test_repo_copy");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        fs::create_dir_all(&temp_dir).unwrap();

        let _ = Command::new("git")
            .arg("init")
            .current_dir(&temp_dir)
            .status()
            .unwrap();
        
        // 2. Copy pip.rs into the repo
        let pip_rs_path = PathBuf::from("types/src/nostr/pip.rs");
        let dest_path = temp_dir.join("pip.rs");
        fs::copy(&pip_rs_path, &dest_path).unwrap();

        // 3. Walk directory and collect bytes
        let mut all_data = Vec::new();
        let mut entries: Vec<PathBuf> = fs::read_dir(&temp_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .collect();
        entries.sort();

        for entry in entries {
            let data = fs::read(&entry).unwrap();
            println!("Collected file: {:?}, size: {} bytes", entry.file_name(), data.len());
            all_data.extend(data);
        }

        // 4. Recursive packetize
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

        let mut seq = 0;
        let mut packets = recursive_process("ROOT".to_string(), all_data, &mut seq);
        let total = packets.len() as u64;
        
        for p in &mut packets {
            p.header.total_packets = total;
        }

        let batch = PacketBatch { total_packets: total, packets };
        println!("Git Repo Batch (with pip.rs copy): {} packets", batch.total_packets);

        // 5. Verify
        assert!(batch.total_packets > 0);
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_git_repo_with_file_copy\n");
    }

    #[test]
    fn test_packetize_and_broadcast_git_repo() {
        println!("\n>>> START: test_packetize_and_broadcast_git_repo");
        
        // 1. Clone repo
        let temp_dir = std::env::temp_dir().join("gnostr_git_test_clone");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        
        println!("Cloning repository...");
        let _ = Command::new("git")
            .args(["clone", "https://github.com/gnostr-org/git-test.git", &temp_dir.to_string_lossy()])
            .status()
            .unwrap();

        // 2. Packetize
        // Note: Packetization logic would be same as above.
        // For now, demonstrate preparation for broadcast.

        println!("Git Test Repo cloned and ready for packetization.");
        
        // This is where you would call the broadcast mechanism if approved.
        println!("Broadcasting not implemented: Awaiting explicit command.");
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_and_broadcast_git_repo\n");
    }
}

    #[test]
    fn test_recursive_packetization() {
        println!("\n>>> START: test_recursive_packetization");
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
            
    #[test]
    fn test_packetize_git_repo_with_file_copy() {
        println!("\n>>> START: test_packetize_git_repo_with_file_copy");
        
        // 1. Setup temporary git repository
        let temp_dir = std::env::temp_dir().join("gnostr_pip_test_repo_copy");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        fs::create_dir_all(&temp_dir).unwrap();

        let _ = Command::new("git")
            .arg("init")
            .current_dir(&temp_dir)
            .status()
            .unwrap();
        
        // 2. Copy pip.rs into the repo
        let pip_rs_path = PathBuf::from("types/src/nostr/pip.rs");
        let dest_path = temp_dir.join("pip.rs");
        fs::copy(&pip_rs_path, &dest_path).unwrap();

        // 3. Walk directory and collect bytes
        let mut all_data = Vec::new();
        let mut entries: Vec<PathBuf> = fs::read_dir(&temp_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .collect();
        entries.sort();

        for entry in entries {
            let data = fs::read(&entry).unwrap();
            println!("Collected file: {:?}, size: {} bytes", entry.file_name(), data.len());
            all_data.extend(data);
        }

        // 4. Recursive packetize
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

        let mut seq = 0;
        let mut packets = recursive_process("ROOT".to_string(), all_data, &mut seq);
        let total = packets.len() as u64;
        
        for p in &mut packets {
            p.header.total_packets = total;
        }

        let batch = PacketBatch { total_packets: total, packets };
        println!("Git Repo Batch (with pip.rs copy): {} packets", batch.total_packets);

        // 5. Verify
        assert!(batch.total_packets > 0);
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_git_repo_with_file_copy\n");
    }

    #[test]
    fn test_packetize_and_broadcast_git_repo() {
        println!("\n>>> START: test_packetize_and_broadcast_git_repo");
        
        // 1. Clone repo
        let temp_dir = std::env::temp_dir().join("gnostr_git_test_clone");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        
        println!("Cloning repository...");
        let _ = Command::new("git")
            .args(["clone", "https://github.com/gnostr-org/git-test.git", &temp_dir.to_string_lossy()])
            .status()
            .unwrap();

        // 2. Packetize
        // Note: Packetization logic would be same as above.
        // For now, demonstrate preparation for broadcast.

        println!("Git Test Repo cloned and ready for packetization.");
        
        // This is where you would call the broadcast mechanism if approved.
        println!("Broadcasting not implemented: Awaiting explicit command.");
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_and_broadcast_git_repo\n");
    }
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
        
    #[test]
    fn test_packetize_git_repo_with_file_copy() {
        println!("\n>>> START: test_packetize_git_repo_with_file_copy");
        
        // 1. Setup temporary git repository
        let temp_dir = std::env::temp_dir().join("gnostr_pip_test_repo_copy");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        fs::create_dir_all(&temp_dir).unwrap();

        let _ = Command::new("git")
            .arg("init")
            .current_dir(&temp_dir)
            .status()
            .unwrap();
        
        // 2. Copy pip.rs into the repo
        let pip_rs_path = PathBuf::from("types/src/nostr/pip.rs");
        let dest_path = temp_dir.join("pip.rs");
        fs::copy(&pip_rs_path, &dest_path).unwrap();

        // 3. Walk directory and collect bytes
        let mut all_data = Vec::new();
        let mut entries: Vec<PathBuf> = fs::read_dir(&temp_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .collect();
        entries.sort();

        for entry in entries {
            let data = fs::read(&entry).unwrap();
            println!("Collected file: {:?}, size: {} bytes", entry.file_name(), data.len());
            all_data.extend(data);
        }

        // 4. Recursive packetize
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

        let mut seq = 0;
        let mut packets = recursive_process("ROOT".to_string(), all_data, &mut seq);
        let total = packets.len() as u64;
        
        for p in &mut packets {
            p.header.total_packets = total;
        }

        let batch = PacketBatch { total_packets: total, packets };
        println!("Git Repo Batch (with pip.rs copy): {} packets", batch.total_packets);

        // 5. Verify
        assert!(batch.total_packets > 0);
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_git_repo_with_file_copy\n");
    }

    #[test]
    fn test_packetize_and_broadcast_git_repo() {
        println!("\n>>> START: test_packetize_and_broadcast_git_repo");
        
        // 1. Clone repo
        let temp_dir = std::env::temp_dir().join("gnostr_git_test_clone");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        
        println!("Cloning repository...");
        let _ = Command::new("git")
            .args(["clone", "https://github.com/gnostr-org/git-test.git", &temp_dir.to_string_lossy()])
            .status()
            .unwrap();

        // 2. Packetize
        // Note: Packetization logic would be same as above.
        // For now, demonstrate preparation for broadcast.

        println!("Git Test Repo cloned and ready for packetization.");
        
        // This is where you would call the broadcast mechanism if approved.
        println!("Broadcasting not implemented: Awaiting explicit command.");
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_and_broadcast_git_repo\n");
    }
}

        let data = vec![0xAB; 30];
        let mut seq = 0;
        let mut packets = recursive_process("ROOT".to_string(), data, &mut seq);
        let total = packets.len() as u64;
        
        for p in &mut packets {
            p.header.total_packets = total;
        
    #[test]
    fn test_packetize_git_repo_with_file_copy() {
        println!("\n>>> START: test_packetize_git_repo_with_file_copy");
        
        // 1. Setup temporary git repository
        let temp_dir = std::env::temp_dir().join("gnostr_pip_test_repo_copy");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        fs::create_dir_all(&temp_dir).unwrap();

        let _ = Command::new("git")
            .arg("init")
            .current_dir(&temp_dir)
            .status()
            .unwrap();
        
        // 2. Copy pip.rs into the repo
        let pip_rs_path = PathBuf::from("types/src/nostr/pip.rs");
        let dest_path = temp_dir.join("pip.rs");
        fs::copy(&pip_rs_path, &dest_path).unwrap();

        // 3. Walk directory and collect bytes
        let mut all_data = Vec::new();
        let mut entries: Vec<PathBuf> = fs::read_dir(&temp_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .collect();
        entries.sort();

        for entry in entries {
            let data = fs::read(&entry).unwrap();
            println!("Collected file: {:?}, size: {} bytes", entry.file_name(), data.len());
            all_data.extend(data);
        }

        // 4. Recursive packetize
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

        let mut seq = 0;
        let mut packets = recursive_process("ROOT".to_string(), all_data, &mut seq);
        let total = packets.len() as u64;
        
        for p in &mut packets {
            p.header.total_packets = total;
        }

        let batch = PacketBatch { total_packets: total, packets };
        println!("Git Repo Batch (with pip.rs copy): {} packets", batch.total_packets);

        // 5. Verify
        assert!(batch.total_packets > 0);
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_git_repo_with_file_copy\n");
    }

    #[test]
    fn test_packetize_and_broadcast_git_repo() {
        println!("\n>>> START: test_packetize_and_broadcast_git_repo");
        
        // 1. Clone repo
        let temp_dir = std::env::temp_dir().join("gnostr_git_test_clone");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        
        println!("Cloning repository...");
        let _ = Command::new("git")
            .args(["clone", "https://github.com/gnostr-org/git-test.git", &temp_dir.to_string_lossy()])
            .status()
            .unwrap();

        // 2. Packetize
        // Note: Packetization logic would be same as above.
        // For now, demonstrate preparation for broadcast.

        println!("Git Test Repo cloned and ready for packetization.");
        
        // This is where you would call the broadcast mechanism if approved.
        println!("Broadcasting not implemented: Awaiting explicit command.");
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_and_broadcast_git_repo\n");
    }
}

        let batch = PacketBatch { total_packets: total, packets };
        println!("Recursive Batch: {:?}", batch);
        
        assert!(batch.packets.iter().any(|p| p.is_parity));
        assert!(batch.packets.iter().any(|p| p.id == "ROOT.P"));
        assert_eq!(batch.total_packets, 7);
        println!("<<< END: test_recursive_packetization\n");
    
    #[test]
    fn test_packetize_git_repo_with_file_copy() {
        println!("\n>>> START: test_packetize_git_repo_with_file_copy");
        
        // 1. Setup temporary git repository
        let temp_dir = std::env::temp_dir().join("gnostr_pip_test_repo_copy");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        fs::create_dir_all(&temp_dir).unwrap();

        let _ = Command::new("git")
            .arg("init")
            .current_dir(&temp_dir)
            .status()
            .unwrap();
        
        // 2. Copy pip.rs into the repo
        let pip_rs_path = PathBuf::from("types/src/nostr/pip.rs");
        let dest_path = temp_dir.join("pip.rs");
        fs::copy(&pip_rs_path, &dest_path).unwrap();

        // 3. Walk directory and collect bytes
        let mut all_data = Vec::new();
        let mut entries: Vec<PathBuf> = fs::read_dir(&temp_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .collect();
        entries.sort();

        for entry in entries {
            let data = fs::read(&entry).unwrap();
            println!("Collected file: {:?}, size: {} bytes", entry.file_name(), data.len());
            all_data.extend(data);
        }

        // 4. Recursive packetize
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

        let mut seq = 0;
        let mut packets = recursive_process("ROOT".to_string(), all_data, &mut seq);
        let total = packets.len() as u64;
        
        for p in &mut packets {
            p.header.total_packets = total;
        }

        let batch = PacketBatch { total_packets: total, packets };
        println!("Git Repo Batch (with pip.rs copy): {} packets", batch.total_packets);

        // 5. Verify
        assert!(batch.total_packets > 0);
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_git_repo_with_file_copy\n");
    }

    #[test]
    fn test_packetize_and_broadcast_git_repo() {
        println!("\n>>> START: test_packetize_and_broadcast_git_repo");
        
        // 1. Clone repo
        let temp_dir = std::env::temp_dir().join("gnostr_git_test_clone");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        
        println!("Cloning repository...");
        let _ = Command::new("git")
            .args(["clone", "https://github.com/gnostr-org/git-test.git", &temp_dir.to_string_lossy()])
            .status()
            .unwrap();

        // 2. Packetize
        // Note: Packetization logic would be same as above.
        // For now, demonstrate preparation for broadcast.

        println!("Git Test Repo cloned and ready for packetization.");
        
        // This is where you would call the broadcast mechanism if approved.
        println!("Broadcasting not implemented: Awaiting explicit command.");
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_and_broadcast_git_repo\n");
    }
}

    #[test]
    fn test_large_file_packetization() {
        println!("\n>>> START: test_large_file_packetization");
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
            
    #[test]
    fn test_packetize_git_repo_with_file_copy() {
        println!("\n>>> START: test_packetize_git_repo_with_file_copy");
        
        // 1. Setup temporary git repository
        let temp_dir = std::env::temp_dir().join("gnostr_pip_test_repo_copy");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        fs::create_dir_all(&temp_dir).unwrap();

        let _ = Command::new("git")
            .arg("init")
            .current_dir(&temp_dir)
            .status()
            .unwrap();
        
        // 2. Copy pip.rs into the repo
        let pip_rs_path = PathBuf::from("types/src/nostr/pip.rs");
        let dest_path = temp_dir.join("pip.rs");
        fs::copy(&pip_rs_path, &dest_path).unwrap();

        // 3. Walk directory and collect bytes
        let mut all_data = Vec::new();
        let mut entries: Vec<PathBuf> = fs::read_dir(&temp_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .collect();
        entries.sort();

        for entry in entries {
            let data = fs::read(&entry).unwrap();
            println!("Collected file: {:?}, size: {} bytes", entry.file_name(), data.len());
            all_data.extend(data);
        }

        // 4. Recursive packetize
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

        let mut seq = 0;
        let mut packets = recursive_process("ROOT".to_string(), all_data, &mut seq);
        let total = packets.len() as u64;
        
        for p in &mut packets {
            p.header.total_packets = total;
        }

        let batch = PacketBatch { total_packets: total, packets };
        println!("Git Repo Batch (with pip.rs copy): {} packets", batch.total_packets);

        // 5. Verify
        assert!(batch.total_packets > 0);
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_git_repo_with_file_copy\n");
    }

    #[test]
    fn test_packetize_and_broadcast_git_repo() {
        println!("\n>>> START: test_packetize_and_broadcast_git_repo");
        
        // 1. Clone repo
        let temp_dir = std::env::temp_dir().join("gnostr_git_test_clone");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        
        println!("Cloning repository...");
        let _ = Command::new("git")
            .args(["clone", "https://github.com/gnostr-org/git-test.git", &temp_dir.to_string_lossy()])
            .status()
            .unwrap();

        // 2. Packetize
        // Note: Packetization logic would be same as above.
        // For now, demonstrate preparation for broadcast.

        println!("Git Test Repo cloned and ready for packetization.");
        
        // This is where you would call the broadcast mechanism if approved.
        println!("Broadcasting not implemented: Awaiting explicit command.");
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_and_broadcast_git_repo\n");
    }
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
        
    #[test]
    fn test_packetize_git_repo_with_file_copy() {
        println!("\n>>> START: test_packetize_git_repo_with_file_copy");
        
        // 1. Setup temporary git repository
        let temp_dir = std::env::temp_dir().join("gnostr_pip_test_repo_copy");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        fs::create_dir_all(&temp_dir).unwrap();

        let _ = Command::new("git")
            .arg("init")
            .current_dir(&temp_dir)
            .status()
            .unwrap();
        
        // 2. Copy pip.rs into the repo
        let pip_rs_path = PathBuf::from("types/src/nostr/pip.rs");
        let dest_path = temp_dir.join("pip.rs");
        fs::copy(&pip_rs_path, &dest_path).unwrap();

        // 3. Walk directory and collect bytes
        let mut all_data = Vec::new();
        let mut entries: Vec<PathBuf> = fs::read_dir(&temp_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .collect();
        entries.sort();

        for entry in entries {
            let data = fs::read(&entry).unwrap();
            println!("Collected file: {:?}, size: {} bytes", entry.file_name(), data.len());
            all_data.extend(data);
        }

        // 4. Recursive packetize
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

        let mut seq = 0;
        let mut packets = recursive_process("ROOT".to_string(), all_data, &mut seq);
        let total = packets.len() as u64;
        
        for p in &mut packets {
            p.header.total_packets = total;
        }

        let batch = PacketBatch { total_packets: total, packets };
        println!("Git Repo Batch (with pip.rs copy): {} packets", batch.total_packets);

        // 5. Verify
        assert!(batch.total_packets > 0);
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_git_repo_with_file_copy\n");
    }

    #[test]
    fn test_packetize_and_broadcast_git_repo() {
        println!("\n>>> START: test_packetize_and_broadcast_git_repo");
        
        // 1. Clone repo
        let temp_dir = std::env::temp_dir().join("gnostr_git_test_clone");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        
        println!("Cloning repository...");
        let _ = Command::new("git")
            .args(["clone", "https://github.com/gnostr-org/git-test.git", &temp_dir.to_string_lossy()])
            .status()
            .unwrap();

        // 2. Packetize
        // Note: Packetization logic would be same as above.
        // For now, demonstrate preparation for broadcast.

        println!("Git Test Repo cloned and ready for packetization.");
        
        // This is where you would call the broadcast mechanism if approved.
        println!("Broadcasting not implemented: Awaiting explicit command.");
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_and_broadcast_git_repo\n");
    }
}

        let mut packets = recursive_process("ROOT".to_string(), large_data, &mut seq);
        let total = packets.len() as u64;
        for p in &mut packets {
            p.header.total_packets = total;
        
    #[test]
    fn test_packetize_git_repo_with_file_copy() {
        println!("\n>>> START: test_packetize_git_repo_with_file_copy");
        
        // 1. Setup temporary git repository
        let temp_dir = std::env::temp_dir().join("gnostr_pip_test_repo_copy");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        fs::create_dir_all(&temp_dir).unwrap();

        let _ = Command::new("git")
            .arg("init")
            .current_dir(&temp_dir)
            .status()
            .unwrap();
        
        // 2. Copy pip.rs into the repo
        let pip_rs_path = PathBuf::from("types/src/nostr/pip.rs");
        let dest_path = temp_dir.join("pip.rs");
        fs::copy(&pip_rs_path, &dest_path).unwrap();

        // 3. Walk directory and collect bytes
        let mut all_data = Vec::new();
        let mut entries: Vec<PathBuf> = fs::read_dir(&temp_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .collect();
        entries.sort();

        for entry in entries {
            let data = fs::read(&entry).unwrap();
            println!("Collected file: {:?}, size: {} bytes", entry.file_name(), data.len());
            all_data.extend(data);
        }

        // 4. Recursive packetize
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

        let mut seq = 0;
        let mut packets = recursive_process("ROOT".to_string(), all_data, &mut seq);
        let total = packets.len() as u64;
        
        for p in &mut packets {
            p.header.total_packets = total;
        }

        let batch = PacketBatch { total_packets: total, packets };
        println!("Git Repo Batch (with pip.rs copy): {} packets", batch.total_packets);

        // 5. Verify
        assert!(batch.total_packets > 0);
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_git_repo_with_file_copy\n");
    }

    #[test]
    fn test_packetize_and_broadcast_git_repo() {
        println!("\n>>> START: test_packetize_and_broadcast_git_repo");
        
        // 1. Clone repo
        let temp_dir = std::env::temp_dir().join("gnostr_git_test_clone");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        
        println!("Cloning repository...");
        let _ = Command::new("git")
            .args(["clone", "https://github.com/gnostr-org/git-test.git", &temp_dir.to_string_lossy()])
            .status()
            .unwrap();

        // 2. Packetize
        // Note: Packetization logic would be same as above.
        // For now, demonstrate preparation for broadcast.

        println!("Git Test Repo cloned and ready for packetization.");
        
        // This is where you would call the broadcast mechanism if approved.
        println!("Broadcasting not implemented: Awaiting explicit command.");
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_and_broadcast_git_repo\n");
    }
}

        let batch = PacketBatch { total_packets: total, packets };
        println!("Large Batch: {} packets", batch.total_packets);
        assert!(batch.total_packets > 10);
        println!("<<< END: test_large_file_packetization\n");
    
    #[test]
    fn test_packetize_git_repo_with_file_copy() {
        println!("\n>>> START: test_packetize_git_repo_with_file_copy");
        
        // 1. Setup temporary git repository
        let temp_dir = std::env::temp_dir().join("gnostr_pip_test_repo_copy");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        fs::create_dir_all(&temp_dir).unwrap();

        let _ = Command::new("git")
            .arg("init")
            .current_dir(&temp_dir)
            .status()
            .unwrap();
        
        // 2. Copy pip.rs into the repo
        let pip_rs_path = PathBuf::from("types/src/nostr/pip.rs");
        let dest_path = temp_dir.join("pip.rs");
        fs::copy(&pip_rs_path, &dest_path).unwrap();

        // 3. Walk directory and collect bytes
        let mut all_data = Vec::new();
        let mut entries: Vec<PathBuf> = fs::read_dir(&temp_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .collect();
        entries.sort();

        for entry in entries {
            let data = fs::read(&entry).unwrap();
            println!("Collected file: {:?}, size: {} bytes", entry.file_name(), data.len());
            all_data.extend(data);
        }

        // 4. Recursive packetize
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

        let mut seq = 0;
        let mut packets = recursive_process("ROOT".to_string(), all_data, &mut seq);
        let total = packets.len() as u64;
        
        for p in &mut packets {
            p.header.total_packets = total;
        }

        let batch = PacketBatch { total_packets: total, packets };
        println!("Git Repo Batch (with pip.rs copy): {} packets", batch.total_packets);

        // 5. Verify
        assert!(batch.total_packets > 0);
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_git_repo_with_file_copy\n");
    }

    #[test]
    fn test_packetize_and_broadcast_git_repo() {
        println!("\n>>> START: test_packetize_and_broadcast_git_repo");
        
        // 1. Clone repo
        let temp_dir = std::env::temp_dir().join("gnostr_git_test_clone");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        
        println!("Cloning repository...");
        let _ = Command::new("git")
            .args(["clone", "https://github.com/gnostr-org/git-test.git", &temp_dir.to_string_lossy()])
            .status()
            .unwrap();

        // 2. Packetize
        // Note: Packetization logic would be same as above.
        // For now, demonstrate preparation for broadcast.

        println!("Git Test Repo cloned and ready for packetization.");
        
        // This is where you would call the broadcast mechanism if approved.
        println!("Broadcasting not implemented: Awaiting explicit command.");
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_and_broadcast_git_repo\n");
    }
}

    #[test]
    fn test_parity_recovery() {
        println!("\n>>> START: test_parity_recovery");
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
        println!("<<< END: test_parity_recovery\n");
    
    #[test]
    fn test_packetize_git_repo_with_file_copy() {
        println!("\n>>> START: test_packetize_git_repo_with_file_copy");
        
        // 1. Setup temporary git repository
        let temp_dir = std::env::temp_dir().join("gnostr_pip_test_repo_copy");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        fs::create_dir_all(&temp_dir).unwrap();

        let _ = Command::new("git")
            .arg("init")
            .current_dir(&temp_dir)
            .status()
            .unwrap();
        
        // 2. Copy pip.rs into the repo
        let pip_rs_path = PathBuf::from("types/src/nostr/pip.rs");
        let dest_path = temp_dir.join("pip.rs");
        fs::copy(&pip_rs_path, &dest_path).unwrap();

        // 3. Walk directory and collect bytes
        let mut all_data = Vec::new();
        let mut entries: Vec<PathBuf> = fs::read_dir(&temp_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .collect();
        entries.sort();

        for entry in entries {
            let data = fs::read(&entry).unwrap();
            println!("Collected file: {:?}, size: {} bytes", entry.file_name(), data.len());
            all_data.extend(data);
        }

        // 4. Recursive packetize
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

        let mut seq = 0;
        let mut packets = recursive_process("ROOT".to_string(), all_data, &mut seq);
        let total = packets.len() as u64;
        
        for p in &mut packets {
            p.header.total_packets = total;
        }

        let batch = PacketBatch { total_packets: total, packets };
        println!("Git Repo Batch (with pip.rs copy): {} packets", batch.total_packets);

        // 5. Verify
        assert!(batch.total_packets > 0);
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_git_repo_with_file_copy\n");
    }

    #[test]
    fn test_packetize_and_broadcast_git_repo() {
        println!("\n>>> START: test_packetize_and_broadcast_git_repo");
        
        // 1. Clone repo
        let temp_dir = std::env::temp_dir().join("gnostr_git_test_clone");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        
        println!("Cloning repository...");
        let _ = Command::new("git")
            .args(["clone", "https://github.com/gnostr-org/git-test.git", &temp_dir.to_string_lossy()])
            .status()
            .unwrap();

        // 2. Packetize
        // Note: Packetization logic would be same as above.
        // For now, demonstrate preparation for broadcast.

        println!("Git Test Repo cloned and ready for packetization.");
        
        // This is where you would call the broadcast mechanism if approved.
        println!("Broadcasting not implemented: Awaiting explicit command.");
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_and_broadcast_git_repo\n");
    }
}

    #[test]
    fn test_real_pip_manifest_event() {
        use sha2::{Digest, Sha256};
        println!("\n>>> START: test_real_pip_manifest_event");

        let payload = vec![0xAB; 3000];
        let hash = Sha256::digest(&payload);
        let sha256_hex = format!("{:x}", hash);
        println!("Payload hash: {}", sha256_hex);

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
        println!("Content: {}", content);

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

        println!("Real Nostr PIP Manifest Event: {}", event.to_string());
        
        // Assertions
        assert_eq!(event["kind"], 39078);
        assert_eq!(event["tags"][1][1], sha256_hex);
        assert!(event["content"].as_str().unwrap().contains(&sha256_hex));
        println!("<<< END: test_real_pip_manifest_event\n");
    
    #[test]
    fn test_packetize_git_repo_with_file_copy() {
        println!("\n>>> START: test_packetize_git_repo_with_file_copy");
        
        // 1. Setup temporary git repository
        let temp_dir = std::env::temp_dir().join("gnostr_pip_test_repo_copy");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        fs::create_dir_all(&temp_dir).unwrap();

        let _ = Command::new("git")
            .arg("init")
            .current_dir(&temp_dir)
            .status()
            .unwrap();
        
        // 2. Copy pip.rs into the repo
        let pip_rs_path = PathBuf::from("types/src/nostr/pip.rs");
        let dest_path = temp_dir.join("pip.rs");
        fs::copy(&pip_rs_path, &dest_path).unwrap();

        // 3. Walk directory and collect bytes
        let mut all_data = Vec::new();
        let mut entries: Vec<PathBuf> = fs::read_dir(&temp_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .collect();
        entries.sort();

        for entry in entries {
            let data = fs::read(&entry).unwrap();
            println!("Collected file: {:?}, size: {} bytes", entry.file_name(), data.len());
            all_data.extend(data);
        }

        // 4. Recursive packetize
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

        let mut seq = 0;
        let mut packets = recursive_process("ROOT".to_string(), all_data, &mut seq);
        let total = packets.len() as u64;
        
        for p in &mut packets {
            p.header.total_packets = total;
        }

        let batch = PacketBatch { total_packets: total, packets };
        println!("Git Repo Batch (with pip.rs copy): {} packets", batch.total_packets);

        // 5. Verify
        assert!(batch.total_packets > 0);
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_git_repo_with_file_copy\n");
    }

    #[test]
    fn test_packetize_and_broadcast_git_repo() {
        println!("\n>>> START: test_packetize_and_broadcast_git_repo");
        
        // 1. Clone repo
        let temp_dir = std::env::temp_dir().join("gnostr_git_test_clone");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        
        println!("Cloning repository...");
        let _ = Command::new("git")
            .args(["clone", "https://github.com/gnostr-org/git-test.git", &temp_dir.to_string_lossy()])
            .status()
            .unwrap();

        // 2. Packetize
        // Note: Packetization logic would be same as above.
        // For now, demonstrate preparation for broadcast.

        println!("Git Test Repo cloned and ready for packetization.");
        
        // This is where you would call the broadcast mechanism if approved.
        println!("Broadcasting not implemented: Awaiting explicit command.");
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_and_broadcast_git_repo\n");
    }
}

    #[test]
    fn test_packetize_git_repo() {
        println!("\n>>> START: test_packetize_git_repo");
        
        // 1. Setup temporary git repository
        let temp_dir = std::env::temp_dir().join("gnostr_pip_test_repo");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        
    #[test]
    fn test_packetize_git_repo_with_file_copy() {
        println!("\n>>> START: test_packetize_git_repo_with_file_copy");
        
        // 1. Setup temporary git repository
        let temp_dir = std::env::temp_dir().join("gnostr_pip_test_repo_copy");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        fs::create_dir_all(&temp_dir).unwrap();

        let _ = Command::new("git")
            .arg("init")
            .current_dir(&temp_dir)
            .status()
            .unwrap();
        
        // 2. Copy pip.rs into the repo
        let pip_rs_path = PathBuf::from("types/src/nostr/pip.rs");
        let dest_path = temp_dir.join("pip.rs");
        fs::copy(&pip_rs_path, &dest_path).unwrap();

        // 3. Walk directory and collect bytes
        let mut all_data = Vec::new();
        let mut entries: Vec<PathBuf> = fs::read_dir(&temp_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .collect();
        entries.sort();

        for entry in entries {
            let data = fs::read(&entry).unwrap();
            println!("Collected file: {:?}, size: {} bytes", entry.file_name(), data.len());
            all_data.extend(data);
        }

        // 4. Recursive packetize
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

        let mut seq = 0;
        let mut packets = recursive_process("ROOT".to_string(), all_data, &mut seq);
        let total = packets.len() as u64;
        
        for p in &mut packets {
            p.header.total_packets = total;
        }

        let batch = PacketBatch { total_packets: total, packets };
        println!("Git Repo Batch (with pip.rs copy): {} packets", batch.total_packets);

        // 5. Verify
        assert!(batch.total_packets > 0);
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_git_repo_with_file_copy\n");
    }

    #[test]
    fn test_packetize_and_broadcast_git_repo() {
        println!("\n>>> START: test_packetize_and_broadcast_git_repo");
        
        // 1. Clone repo
        let temp_dir = std::env::temp_dir().join("gnostr_git_test_clone");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        
        println!("Cloning repository...");
        let _ = Command::new("git")
            .args(["clone", "https://github.com/gnostr-org/git-test.git", &temp_dir.to_string_lossy()])
            .status()
            .unwrap();

        // 2. Packetize
        // Note: Packetization logic would be same as above.
        // For now, demonstrate preparation for broadcast.

        println!("Git Test Repo cloned and ready for packetization.");
        
        // This is where you would call the broadcast mechanism if approved.
        println!("Broadcasting not implemented: Awaiting explicit command.");
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_and_broadcast_git_repo\n");
    }
}
        fs::create_dir_all(&temp_dir).unwrap();

        let _ = Command::new("git")
            .arg("init")
            .current_dir(&temp_dir)
            .status()
            .unwrap();
        
        // 2. Populate repo
        let file1 = temp_dir.join("file1.txt");
        fs::write(&file1, "Hello Git Repo!").unwrap();
        
        let file2 = temp_dir.join("file2.txt");
        fs::write(&file2, "More content for pip test.").unwrap();

        // 3. Walk directory and collect bytes
        let mut all_data = Vec::new();
        let mut entries: Vec<PathBuf> = fs::read_dir(&temp_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .collect();
        entries.sort();

        for entry in entries {
            let data = fs::read(entry).unwrap();
            all_data.extend(data);
        
    #[test]
    fn test_packetize_git_repo_with_file_copy() {
        println!("\n>>> START: test_packetize_git_repo_with_file_copy");
        
        // 1. Setup temporary git repository
        let temp_dir = std::env::temp_dir().join("gnostr_pip_test_repo_copy");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        fs::create_dir_all(&temp_dir).unwrap();

        let _ = Command::new("git")
            .arg("init")
            .current_dir(&temp_dir)
            .status()
            .unwrap();
        
        // 2. Copy pip.rs into the repo
        let pip_rs_path = PathBuf::from("types/src/nostr/pip.rs");
        let dest_path = temp_dir.join("pip.rs");
        fs::copy(&pip_rs_path, &dest_path).unwrap();

        // 3. Walk directory and collect bytes
        let mut all_data = Vec::new();
        let mut entries: Vec<PathBuf> = fs::read_dir(&temp_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .collect();
        entries.sort();

        for entry in entries {
            let data = fs::read(&entry).unwrap();
            println!("Collected file: {:?}, size: {} bytes", entry.file_name(), data.len());
            all_data.extend(data);
        }

        // 4. Recursive packetize
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

        let mut seq = 0;
        let mut packets = recursive_process("ROOT".to_string(), all_data, &mut seq);
        let total = packets.len() as u64;
        
        for p in &mut packets {
            p.header.total_packets = total;
        }

        let batch = PacketBatch { total_packets: total, packets };
        println!("Git Repo Batch (with pip.rs copy): {} packets", batch.total_packets);

        // 5. Verify
        assert!(batch.total_packets > 0);
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_git_repo_with_file_copy\n");
    }

    #[test]
    fn test_packetize_and_broadcast_git_repo() {
        println!("\n>>> START: test_packetize_and_broadcast_git_repo");
        
        // 1. Clone repo
        let temp_dir = std::env::temp_dir().join("gnostr_git_test_clone");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        
        println!("Cloning repository...");
        let _ = Command::new("git")
            .args(["clone", "https://github.com/gnostr-org/git-test.git", &temp_dir.to_string_lossy()])
            .status()
            .unwrap();

        // 2. Packetize
        // Note: Packetization logic would be same as above.
        // For now, demonstrate preparation for broadcast.

        println!("Git Test Repo cloned and ready for packetization.");
        
        // This is where you would call the broadcast mechanism if approved.
        println!("Broadcasting not implemented: Awaiting explicit command.");
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_and_broadcast_git_repo\n");
    }
}

        // 4. Recursive packetize
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
            
    #[test]
    fn test_packetize_git_repo_with_file_copy() {
        println!("\n>>> START: test_packetize_git_repo_with_file_copy");
        
        // 1. Setup temporary git repository
        let temp_dir = std::env::temp_dir().join("gnostr_pip_test_repo_copy");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        fs::create_dir_all(&temp_dir).unwrap();

        let _ = Command::new("git")
            .arg("init")
            .current_dir(&temp_dir)
            .status()
            .unwrap();
        
        // 2. Copy pip.rs into the repo
        let pip_rs_path = PathBuf::from("types/src/nostr/pip.rs");
        let dest_path = temp_dir.join("pip.rs");
        fs::copy(&pip_rs_path, &dest_path).unwrap();

        // 3. Walk directory and collect bytes
        let mut all_data = Vec::new();
        let mut entries: Vec<PathBuf> = fs::read_dir(&temp_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .collect();
        entries.sort();

        for entry in entries {
            let data = fs::read(&entry).unwrap();
            println!("Collected file: {:?}, size: {} bytes", entry.file_name(), data.len());
            all_data.extend(data);
        }

        // 4. Recursive packetize
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

        let mut seq = 0;
        let mut packets = recursive_process("ROOT".to_string(), all_data, &mut seq);
        let total = packets.len() as u64;
        
        for p in &mut packets {
            p.header.total_packets = total;
        }

        let batch = PacketBatch { total_packets: total, packets };
        println!("Git Repo Batch (with pip.rs copy): {} packets", batch.total_packets);

        // 5. Verify
        assert!(batch.total_packets > 0);
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_git_repo_with_file_copy\n");
    }

    #[test]
    fn test_packetize_and_broadcast_git_repo() {
        println!("\n>>> START: test_packetize_and_broadcast_git_repo");
        
        // 1. Clone repo
        let temp_dir = std::env::temp_dir().join("gnostr_git_test_clone");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        
        println!("Cloning repository...");
        let _ = Command::new("git")
            .args(["clone", "https://github.com/gnostr-org/git-test.git", &temp_dir.to_string_lossy()])
            .status()
            .unwrap();

        // 2. Packetize
        // Note: Packetization logic would be same as above.
        // For now, demonstrate preparation for broadcast.

        println!("Git Test Repo cloned and ready for packetization.");
        
        // This is where you would call the broadcast mechanism if approved.
        println!("Broadcasting not implemented: Awaiting explicit command.");
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_and_broadcast_git_repo\n");
    }
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
        
    #[test]
    fn test_packetize_git_repo_with_file_copy() {
        println!("\n>>> START: test_packetize_git_repo_with_file_copy");
        
        // 1. Setup temporary git repository
        let temp_dir = std::env::temp_dir().join("gnostr_pip_test_repo_copy");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        fs::create_dir_all(&temp_dir).unwrap();

        let _ = Command::new("git")
            .arg("init")
            .current_dir(&temp_dir)
            .status()
            .unwrap();
        
        // 2. Copy pip.rs into the repo
        let pip_rs_path = PathBuf::from("types/src/nostr/pip.rs");
        let dest_path = temp_dir.join("pip.rs");
        fs::copy(&pip_rs_path, &dest_path).unwrap();

        // 3. Walk directory and collect bytes
        let mut all_data = Vec::new();
        let mut entries: Vec<PathBuf> = fs::read_dir(&temp_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .collect();
        entries.sort();

        for entry in entries {
            let data = fs::read(&entry).unwrap();
            println!("Collected file: {:?}, size: {} bytes", entry.file_name(), data.len());
            all_data.extend(data);
        }

        // 4. Recursive packetize
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

        let mut seq = 0;
        let mut packets = recursive_process("ROOT".to_string(), all_data, &mut seq);
        let total = packets.len() as u64;
        
        for p in &mut packets {
            p.header.total_packets = total;
        }

        let batch = PacketBatch { total_packets: total, packets };
        println!("Git Repo Batch (with pip.rs copy): {} packets", batch.total_packets);

        // 5. Verify
        assert!(batch.total_packets > 0);
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_git_repo_with_file_copy\n");
    }

    #[test]
    fn test_packetize_and_broadcast_git_repo() {
        println!("\n>>> START: test_packetize_and_broadcast_git_repo");
        
        // 1. Clone repo
        let temp_dir = std::env::temp_dir().join("gnostr_git_test_clone");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        
        println!("Cloning repository...");
        let _ = Command::new("git")
            .args(["clone", "https://github.com/gnostr-org/git-test.git", &temp_dir.to_string_lossy()])
            .status()
            .unwrap();

        // 2. Packetize
        // Note: Packetization logic would be same as above.
        // For now, demonstrate preparation for broadcast.

        println!("Git Test Repo cloned and ready for packetization.");
        
        // This is where you would call the broadcast mechanism if approved.
        println!("Broadcasting not implemented: Awaiting explicit command.");
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_and_broadcast_git_repo\n");
    }
}

        let mut seq = 0;
        let mut packets = recursive_process("ROOT".to_string(), all_data, &mut seq);
        let total = packets.len() as u64;
        
        for p in &mut packets {
            p.header.total_packets = total;
        
    #[test]
    fn test_packetize_git_repo_with_file_copy() {
        println!("\n>>> START: test_packetize_git_repo_with_file_copy");
        
        // 1. Setup temporary git repository
        let temp_dir = std::env::temp_dir().join("gnostr_pip_test_repo_copy");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        fs::create_dir_all(&temp_dir).unwrap();

        let _ = Command::new("git")
            .arg("init")
            .current_dir(&temp_dir)
            .status()
            .unwrap();
        
        // 2. Copy pip.rs into the repo
        let pip_rs_path = PathBuf::from("types/src/nostr/pip.rs");
        let dest_path = temp_dir.join("pip.rs");
        fs::copy(&pip_rs_path, &dest_path).unwrap();

        // 3. Walk directory and collect bytes
        let mut all_data = Vec::new();
        let mut entries: Vec<PathBuf> = fs::read_dir(&temp_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .collect();
        entries.sort();

        for entry in entries {
            let data = fs::read(&entry).unwrap();
            println!("Collected file: {:?}, size: {} bytes", entry.file_name(), data.len());
            all_data.extend(data);
        }

        // 4. Recursive packetize
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

        let mut seq = 0;
        let mut packets = recursive_process("ROOT".to_string(), all_data, &mut seq);
        let total = packets.len() as u64;
        
        for p in &mut packets {
            p.header.total_packets = total;
        }

        let batch = PacketBatch { total_packets: total, packets };
        println!("Git Repo Batch (with pip.rs copy): {} packets", batch.total_packets);

        // 5. Verify
        assert!(batch.total_packets > 0);
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_git_repo_with_file_copy\n");
    }

    #[test]
    fn test_packetize_and_broadcast_git_repo() {
        println!("\n>>> START: test_packetize_and_broadcast_git_repo");
        
        // 1. Clone repo
        let temp_dir = std::env::temp_dir().join("gnostr_git_test_clone");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        
        println!("Cloning repository...");
        let _ = Command::new("git")
            .args(["clone", "https://github.com/gnostr-org/git-test.git", &temp_dir.to_string_lossy()])
            .status()
            .unwrap();

        // 2. Packetize
        // Note: Packetization logic would be same as above.
        // For now, demonstrate preparation for broadcast.

        println!("Git Test Repo cloned and ready for packetization.");
        
        // This is where you would call the broadcast mechanism if approved.
        println!("Broadcasting not implemented: Awaiting explicit command.");
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_and_broadcast_git_repo\n");
    }
}

        let batch = PacketBatch { total_packets: total, packets };
        println!("Git Repo Batch: {} packets", batch.total_packets);

        // 5. Verify
        assert!(batch.total_packets > 0);
        assert!(batch.packets.iter().any(|p| p.is_parity));
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_git_repo\n");
    
    #[test]
    fn test_packetize_git_repo_with_file_copy() {
        println!("\n>>> START: test_packetize_git_repo_with_file_copy");
        
        // 1. Setup temporary git repository
        let temp_dir = std::env::temp_dir().join("gnostr_pip_test_repo_copy");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        fs::create_dir_all(&temp_dir).unwrap();

        let _ = Command::new("git")
            .arg("init")
            .current_dir(&temp_dir)
            .status()
            .unwrap();
        
        // 2. Copy pip.rs into the repo
        let pip_rs_path = PathBuf::from("types/src/nostr/pip.rs");
        let dest_path = temp_dir.join("pip.rs");
        fs::copy(&pip_rs_path, &dest_path).unwrap();

        // 3. Walk directory and collect bytes
        let mut all_data = Vec::new();
        let mut entries: Vec<PathBuf> = fs::read_dir(&temp_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .collect();
        entries.sort();

        for entry in entries {
            let data = fs::read(&entry).unwrap();
            println!("Collected file: {:?}, size: {} bytes", entry.file_name(), data.len());
            all_data.extend(data);
        }

        // 4. Recursive packetize
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

        let mut seq = 0;
        let mut packets = recursive_process("ROOT".to_string(), all_data, &mut seq);
        let total = packets.len() as u64;
        
        for p in &mut packets {
            p.header.total_packets = total;
        }

        let batch = PacketBatch { total_packets: total, packets };
        println!("Git Repo Batch (with pip.rs copy): {} packets", batch.total_packets);

        // 5. Verify
        assert!(batch.total_packets > 0);
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_git_repo_with_file_copy\n");
    }

    #[test]
    fn test_packetize_and_broadcast_git_repo() {
        println!("\n>>> START: test_packetize_and_broadcast_git_repo");
        
        // 1. Clone repo
        let temp_dir = std::env::temp_dir().join("gnostr_git_test_clone");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        
        println!("Cloning repository...");
        let _ = Command::new("git")
            .args(["clone", "https://github.com/gnostr-org/git-test.git", &temp_dir.to_string_lossy()])
            .status()
            .unwrap();

        // 2. Packetize
        // Note: Packetization logic would be same as above.
        // For now, demonstrate preparation for broadcast.

        println!("Git Test Repo cloned and ready for packetization.");
        
        // This is where you would call the broadcast mechanism if approved.
        println!("Broadcasting not implemented: Awaiting explicit command.");
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_and_broadcast_git_repo\n");
    }
}

    #[test]
    fn test_packetize_git_repo_with_file_copy() {
        println!("\n>>> START: test_packetize_git_repo_with_file_copy");
        
        // 1. Setup temporary git repository
        let temp_dir = std::env::temp_dir().join("gnostr_pip_test_repo_copy");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        fs::create_dir_all(&temp_dir).unwrap();

        let _ = Command::new("git")
            .arg("init")
            .current_dir(&temp_dir)
            .status()
            .unwrap();
        
        // 2. Copy pip.rs into the repo
        let pip_rs_path = PathBuf::from("types/src/nostr/pip.rs");
        let dest_path = temp_dir.join("pip.rs");
        fs::copy(&pip_rs_path, &dest_path).unwrap();

        // 3. Walk directory and collect bytes
        let mut all_data = Vec::new();
        let mut entries: Vec<PathBuf> = fs::read_dir(&temp_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .collect();
        entries.sort();

        for entry in entries {
            let data = fs::read(&entry).unwrap();
            println!("Collected file: {:?}, size: {} bytes", entry.file_name(), data.len());
            all_data.extend(data);
        }

        // 4. Recursive packetize
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

        let mut seq = 0;
        let mut packets = recursive_process("ROOT".to_string(), all_data, &mut seq);
        let total = packets.len() as u64;
        
        for p in &mut packets {
            p.header.total_packets = total;
        }

        let batch = PacketBatch { total_packets: total, packets };
        println!("Git Repo Batch (with pip.rs copy): {} packets", batch.total_packets);

        // 5. Verify
        assert!(batch.total_packets > 0);
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_git_repo_with_file_copy\n");
    }

    #[test]
    fn test_packetize_and_broadcast_git_repo() {
        println!("\n>>> START: test_packetize_and_broadcast_git_repo");
        
        // 1. Clone repo
        let temp_dir = std::env::temp_dir().join("gnostr_git_test_clone");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        
        println!("Cloning repository...");
        let _ = Command::new("git")
            .args(["clone", "https://github.com/gnostr-org/git-test.git", &temp_dir.to_string_lossy()])
            .status()
            .unwrap();

        // 2. Packetize
        // Note: Packetization logic would be same as above.
        // For now, demonstrate preparation for broadcast.

        println!("Git Test Repo cloned and ready for packetization.");
        
        // This is where you would call the broadcast mechanism if approved.
        println!("Broadcasting not implemented: Awaiting explicit command.");
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
        println!("<<< END: test_packetize_and_broadcast_git_repo\n");
    }
}
