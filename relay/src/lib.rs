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
    let endpoint_path = data_path.join("relay-endpoint");

    fs::create_dir_all(data_path)?;
    fs::write(&endpoint_path, format!("{endpoint}\n"))?;

    Ok(endpoint_path)
}
