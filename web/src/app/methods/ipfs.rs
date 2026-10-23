use std::sync::{Mutex, OnceLock};

use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use kubo_ffi::{self, Node};

#[derive(Deserialize)]
pub struct AddRequest {
    pub data: String,
}

#[derive(Serialize)]
pub struct AddResponse {
    pub cid: String,
}

#[derive(Serialize)]
pub struct GatewayResponse {
    pub url: String,
}

static NODE: OnceLock<Mutex<Option<Node>>> = OnceLock::new();
static GATEWAY: OnceLock<String> = OnceLock::new();

fn node_store() -> &'static Mutex<Option<Node>> {
    NODE.get_or_init(|| Mutex::new(None))
}

pub fn init(repo_path: &str) -> Result<String, String> {
    // Best-effort init; ignore "already initialized" errors.
    let _ = kubo_ffi::init_repo(repo_path);

    let node = Node::start(repo_path, false)
        .map_err(|e| format!("kubo node start failed: {e}"))?;

    let multiaddr = node
        .start_gateway("/ip4/127.0.0.1/tcp/0")
        .ok_or_else(|| kubo_ffi::last_error().unwrap_or_else(|| "gateway start failed".into()))?;

    let port = parse_port(&multiaddr)
        .ok_or_else(|| format!("could not parse gateway port from {multiaddr}"))?;
    let url = format!("http://127.0.0.1:{port}");

    *node_store().lock().unwrap() = Some(node);
    let _ = GATEWAY.set(url.clone());
    Ok(url)
}

fn parse_port(multiaddr: &str) -> Option<u16> {
    let parts: Vec<&str> = multiaddr.split('/').collect();
    let mut port = None;
    let mut i = 0;
    while i < parts.len() {
        if parts[i] == "tcp" && i + 1 < parts.len() {
            port = parts[i + 1].parse().ok();
        }
        i += 1;
    }
    port
}

pub fn gateway_url() -> Option<String> {
    GATEWAY.get().cloned()
}

pub async fn add(Json(payload): Json<AddRequest>) -> impl IntoResponse {
    let guard = node_store().lock().unwrap();
    let Some(node) = guard.as_ref() else {
        return (StatusCode::SERVICE_UNAVAILABLE, "kubo node not running").into_response();
    };
    match node.unixfs_add_bytes(payload.data.as_bytes()) {
        Ok(cid) => Json(AddResponse { cid }).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

pub async fn gateway() -> impl IntoResponse {
    match gateway_url() {
        Some(url) => Json(GatewayResponse { url }).into_response(),
        None => (StatusCode::NOT_FOUND, "no gateway").into_response(),
    }
}
