//! Integration test mirroring `scripts/gnostr-p2p-fractal-clone.sh`.
//!
//! Packetizes a bare git repository fixture, reconstructs the directory tree,
//! and verifies the result can be cloned with `git clone`.

use std::path::Path;
use std::process::Command;

use gnostr_p2p::{reconstruct_directory, run_directory_walk};
use tempfile::TempDir;

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

    let packets = run_directory_walk(&fixture, 3, false).expect("packetize fixture");
    let file_count = packets.len();
    let total_packets: usize = packets.values().map(|v| v.len()).sum();
    println!("packetized {file_count} files into {total_packets} slices");

    reconstruct_directory(packets, &reconstructed).expect("reconstruct directory");
    println!("reconstructed: {}", reconstructed.display());

    let output = Command::new("git")
        .arg("clone")
        .arg(&reconstructed)
        .arg(&clone)
        .output()
        .expect("spawn git clone");

    if !output.stdout.is_empty() {
        println!("git clone stdout:\n{}", String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() {
        println!("git clone stderr:\n{}", String::from_utf8_lossy(&output.stderr));
    }

    assert!(
        output.status.success(),
        "git clone from reconstructed directory failed"
    );
    assert!(clone.join(".git").exists(), "cloned repo missing .git");
    println!("clone ok: {}", clone.display());
}
