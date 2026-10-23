//! Shared peer registry file for cross-process discovery (crawler <-> p2p).

use std::path::PathBuf;

pub fn registry_path() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("gnostr").join("p2p-peers.json"))
}

pub fn read_peers() -> Vec<String> {
    let Some(path) = registry_path() else { return vec![]; };
    let Ok(content) = std::fs::read_to_string(path) else { return vec![]; };
    serde_json::from_str(&content).unwrap_or_default()
}

pub fn write_peers(peers: &[String]) -> std::io::Result<()> {
    let path = registry_path().ok_or_else(|| std::io::Error::other("no data dir"))?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string(peers).map_err(std::io::Error::other)?;
    std::fs::write(path, json)
}
