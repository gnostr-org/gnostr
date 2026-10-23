//! Optional IPFS helpers for asyncgit.
#[cfg(feature = "ipfs")]
use kubo_ffi::{self, Node};

#[cfg(feature = "ipfs")]
pub fn add_bytes(data: &[u8]) -> Option<String> {
    let repo = std::env::temp_dir().join("gnostr-asyncgit-ipfs");
    let _ = std::fs::create_dir_all(&repo);
    let _ = kubo_ffi::init_repo(repo.to_str().unwrap_or("."));
    let node = Node::start(repo.to_str().unwrap_or("."), false).ok()?;
    let _ = node.start_gateway("/ip4/127.0.0.1/tcp/0");
    node.unixfs_add_bytes(data).ok()
}
