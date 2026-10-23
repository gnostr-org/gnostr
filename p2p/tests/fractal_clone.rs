//! Integration test mirroring `scripts/gnostr-p2p-fractal-clone.sh`.
//!
//! Packetizes a bare git repository fixture, reconstructs the directory tree,
//! and verifies the result can be cloned with `git clone`. Also checks every
//! reconstructed file byte-for-byte against the original fixture so payload
//! corruption is caught immediately.

use std::fs;
use std::path::Path;
use std::process::Command;

use gnostr_p2p::{reconstruct_directory, run_directory_walk};
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
fn fractal_swarm_perfect_ip_reconstructs_git_repo() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../web/crlf.git")
        .canonicalize()
        .expect("locate web/crlf.git fixture");
    println!("fixture: {}", fixture.display());

    let temp = TempDir::new().expect("temp dir");
    let reconstructed = temp.path().join("gnostr_web_crlf");
    let clone = temp.path().join("gnostr_web_crlf_repo");

    let walk = run_directory_walk(&fixture, 3, false).expect("packetize fixture");
    let file_count = walk.files.len();
    let total_packets: usize = walk.files.values().map(|v| v.len()).sum();
    let empty_dir_count = walk.empty_dirs.len();
    println!(
        "packetized {file_count} files into {total_packets} slices ({empty_dir_count} empty dirs)"
    );

    reconstruct_directory(walk, &reconstructed).expect("reconstruct directory");
    println!("reconstructed: {}", reconstructed.display());

    assert_dirs_equal(&fixture, &reconstructed);
    println!("payload verification passed");

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
