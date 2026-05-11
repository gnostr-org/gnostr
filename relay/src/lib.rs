use std::{
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
};

use actix_web::{web, HttpServer};

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

pub async fn run_app_with_endpoint(app_data: nostr_relay::App) -> std::io::Result<()> {
    let data_path = app_data.setting.read().data.path.clone();
    let settings = app_data.setting.read();
    let host = settings.network.host.clone();
    let port = settings.network.port;
    let num = if settings.thread.http == 0 {
        num_cpus::get()
    } else {
        settings.thread.http
    };
    drop(settings);

    let listener = std::net::TcpListener::bind((host.as_str(), port))?;
    let addr = listener.local_addr()?;
    let data = web::Data::new(app_data);

    write_listen_endpoint(&data_path, &[addr])?;

    HttpServer::new(move || create_web_app(data.clone()))
        .workers(num)
        .listen(listener)?
        .run()
        .await
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
