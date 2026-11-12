//! v2 integration test for `fractal_swarm_perfect_ip_v2`.
//!
//! Packetizes a bare git repository fixture with the v2 data plane,
//! reconstructs the directory tree, verifies every file byte-for-byte, runs
//! `git fsck --strict` (TODO.md plan 6), and clones the result. Also pins the
//! v2 fail-closed behavior: a packet whose bytes do not match the manifest
//! sha256 is rejected and reconstruction aborts instead of writing garbage.

use std::fs;
use std::path::Path;
use std::process::Command;

use gnostr_p2p::{
    git_fsck_strict, reconstruct_directory_v2, run_directory_walk_v2, DirectoryWalkV2, FileEntryV2,
    IntegrityManagerV2,
};
use tempfile::TempDir;

fn assert_dirs_equal(left: &Path, right: &Path) {
    let mut left_entries: Vec<_> = fs::read_dir(left)
        .expect("read left dir")
        .map(|e| e.expect("left entry"))
        .collect();
    left_entries.sort_by_key(|e| e.file_name());

    let mut right_entries: Vec<_> = fs::read_dir(right)
        .expect("read right dir")
        .map(|e| e.expect("right entry"))
        .collect();
    right_entries.sort_by_key(|e| e.file_name());

    assert_eq!(
        left_entries.len(),
        right_entries.len(),
        "directory entry count mismatch between {} and {}",
        left.display(),
        right.display()
    );

    for (left_entry, right_entry) in left_entries.iter().zip(right_entries.iter()) {
        assert_eq!(
            left_entry.file_name(),
            right_entry.file_name(),
            "entry name mismatch"
        );

        let left_path = left_entry.path();
        let right_path = right_entry.path();
        let left_meta = left_entry.metadata().expect("left metadata");
        let right_meta = right_entry.metadata().expect("right metadata");

        assert_eq!(
            left_meta.file_type(),
            right_meta.file_type(),
            "entry type mismatch for {}",
            left_path.display()
        );

        if left_meta.is_dir() {
            assert_dirs_equal(&left_path, &right_path);
        } else if left_meta.is_file() {
            let left_bytes = fs::read(&left_path).expect("read left file");
            let right_bytes = fs::read(&right_path).expect("read right file");
            assert_eq!(
                left_bytes,
                right_bytes,
                "payload mismatch for {} ({} bytes vs {} bytes)",
                left_path.display(),
                left_bytes.len(),
                right_bytes.len()
            );
        }
    }
}

#[test]
fn fractal_swarm_perfect_ip_v2_reconstructs_git_repo() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../web/crlf.git")
        .canonicalize()
        .expect("locate web/crlf.git fixture");
    println!("fixture: {}", fixture.display());

    let temp = TempDir::new().expect("temp dir");
    let reconstructed = temp.path().join("gnostr_web_crlf_v2");
    let clone = temp.path().join("gnostr_web_crlf_v2_repo");

    let walk = run_directory_walk_v2(&fixture, 3, false).expect("packetize fixture");
    println!(
        "packetized {} files into {} slices ({} empty dirs)",
        walk.files.len(),
        walk.total_packets(),
        walk.empty_dirs.len()
    );

    // Every file's slices must verify against the manifest before writing.
    for (rel_path, entry) in &walk.files {
        let mut manager = IntegrityManagerV2::new(entry.manifest.clone());
        for packet in &entry.packets {
            manager.record_slice(packet.clone());
        }
        assert!(
            manager.rejected.is_empty(),
            "rejected slices for {rel_path}: {:?}",
            manager.rejected
        );
        assert!(
            manager.verify_integrity(),
            "integrity failed for {rel_path}"
        );
    }

    reconstruct_directory_v2(&walk, &reconstructed).expect("reconstruct directory");
    println!("reconstructed: {}", reconstructed.display());

    assert_dirs_equal(&fixture, &reconstructed);
    println!("payload verification passed");

    // TODO.md plan 6: object-store corruption the clone might tolerate must
    // still fail the fsck step.
    git_fsck_strict(&reconstructed).expect("git fsck --strict");
    println!("git fsck --strict passed");

    let output = Command::new("git")
        .arg("clone")
        .arg(&reconstructed)
        .arg(&clone)
        .output()
        .expect("spawn git clone");

    if !output.stdout.is_empty() {
        println!(
            "git clone stdout:\n{}",
            String::from_utf8_lossy(&output.stdout)
        );
    }
    if !output.stderr.is_empty() {
        println!(
            "git clone stderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    assert!(
        output.status.success(),
        "git clone from reconstructed directory failed"
    );
    assert!(clone.join(".git").exists(), "cloned repo missing .git");
    println!("clone ok: {}", clone.display());

    let reconstructed_head = git_output(&reconstructed, &["rev-parse", "HEAD"]);
    let clone_head = git_output(&clone, &["rev-parse", "HEAD"]);
    assert_eq!(
        reconstructed_head, clone_head,
        "clone HEAD does not match reconstructed repo HEAD"
    );
    println!("HEAD matches: {clone_head}");

    let status = git_output(&clone, &["status", "--porcelain"]);
    assert!(
        status.is_empty(),
        "cloned working tree is not clean:\n{status}"
    );
    println!("cloned working tree is clean");
}

#[test]
fn fractal_v2_reconstruct_rejects_tampered_packets() {
    // TODO.md plan 3 verification: a slice whose bytes do not match the
    // manifest sha256 is rejected, and reconstruction fails closed.
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../web/crlf.git")
        .canonicalize()
        .expect("locate web/crlf.git fixture");

    let walk = run_directory_walk_v2(&fixture, 3, false).expect("packetize fixture");
    let (rel_path, entry) = walk
        .files
        .iter()
        .find(|(_, entry)| entry.packets.iter().any(|packet| !packet.data.is_empty()))
        .expect("a file with a non-empty packet");

    let mut packets = entry.packets.clone();
    let packet = packets
        .iter_mut()
        .find(|packet| !packet.data.is_empty())
        .expect("a non-empty packet to tamper with");
    packet.data[0] ^= 0xFF;
    let tampered = DirectoryWalkV2 {
        files: [(
            rel_path.clone(),
            FileEntryV2 {
                packets,
                manifest: entry.manifest.clone(),
            },
        )]
        .into_iter()
        .collect(),
        empty_dirs: walk.empty_dirs.clone(),
    };

    let temp = TempDir::new().expect("temp dir");
    let result = reconstruct_directory_v2(&tampered, temp.path());
    let error = result.expect_err("tampered reconstruction must fail");
    assert!(
        error.to_string().contains("rejected slices"),
        "unexpected error: {error}"
    );
}

fn git_output(repo: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .expect("spawn git");

    if !output.stdout.is_empty() {
        println!(
            "git -C {} {} stdout:\n{}",
            repo.display(),
            args.join(" "),
            String::from_utf8_lossy(&output.stdout)
        );
    }
    if !output.stderr.is_empty() {
        println!(
            "git -C {} {} stderr:\n{}",
            repo.display(),
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    assert!(
        output.status.success(),
        "git -C {} {} failed",
        repo.display(),
        args.join(" ")
    );
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}
