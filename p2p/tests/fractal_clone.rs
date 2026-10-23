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

    let temp = TempDir::new().expect("temp dir");
    let reconstructed = temp.path().join("gnostr_web_crlf");
    let clone = temp.path().join("gnostr_web_crlf_repo");

    let packets = run_directory_walk(&fixture, 3, false).expect("packetize fixture");
    reconstruct_directory(packets, &reconstructed).expect("reconstruct directory");

    let status = Command::new("git")
        .arg("clone")
        .arg(&reconstructed)
        .arg(&clone)
        .status()
        .expect("spawn git clone");

    assert!(
        status.success(),
        "git clone from reconstructed directory failed"
    );
    assert!(clone.join(".git").exists(), "cloned repo missing .git");
}
