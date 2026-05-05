use std::{
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
};

pub mod cli;
pub mod launcher;

pub use nostr_relay::*;

pub fn write_listen_endpoint(data_path: &Path, addrs: &[SocketAddr]) -> std::io::Result<PathBuf> {
    let Some(addr) = addrs.first().copied() else {
        return Err(std::io::Error::other("relay did not bind any addresses"));
    };

    let host = if addr.ip().is_unspecified() {
        "127.0.0.1".to_string()
    } else {
        addr.ip().to_string()
    };
    let endpoint = format!("ws://{host}:{}", addr.port());
    fs::create_dir_all(data_path)?;
    let endpoint_path = data_path.join("relay-endpoint");
    fs::write(&endpoint_path, format!("{endpoint}\n"))?;
    if let Some(shared_path) = shared_endpoint_path() {
        if let Some(parent) = shared_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(shared_path, format!("{endpoint}\n"))?;
    }

    Ok(endpoint_path)
}

fn shared_endpoint_path() -> Option<PathBuf> {
    let dirs = directories::ProjectDirs::from("org", "gnostr", "gnostr")?;
    let path = dirs.data_local_dir().join("relay-endpoint");
    if path.to_string_lossy().is_empty() {
        None
    } else {
        Some(path)
    }
}
