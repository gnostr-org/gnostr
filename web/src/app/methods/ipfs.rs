#[cfg(feature = "ipfs")]
use std::sync::{Arc, Mutex};

#[cfg(feature = "ipfs")]
use axum::{extract::Extension, http::StatusCode, response::IntoResponse, Json};
#[cfg(feature = "ipfs")]
use serde::Serialize;

#[cfg(feature = "ipfs")]
use kubo_ffi::{self, Node};

#[cfg(feature = "ipfs")]
pub struct IpfsState {
    pub node: Mutex<Option<Node>>,
    pub gateway: Mutex<Option<String>>,
    pub enabled: bool,
    pub last_error: Mutex<Option<String>>,
}

#[cfg(feature = "ipfs")]
#[derive(Serialize)]
pub struct IpfsStatus {
    pub enabled: bool,
    pub gateway: Option<String>,
    pub error: Option<String>,
}

#[cfg(feature = "ipfs")]
#[derive(serde::Deserialize)]
pub struct AddRequest {
    pub data: String,
}

#[cfg(feature = "ipfs")]
#[derive(Serialize)]
pub struct AddResponse {
    pub cid: String,
}

#[cfg(feature = "ipfs")]
#[derive(Serialize)]
pub struct GatewayResponse {
    pub url: String,
}

#[cfg(feature = "ipfs")]
impl IpfsState {
    pub fn init(repo_path: &str, online: bool) -> Arc<Self> {
        let _ = std::fs::create_dir_all(repo_path);
        // Best-effort init; ignore "already initialized" errors.
        let _ = kubo_ffi::init_repo(repo_path);

        let mut node_opt = None;
        let mut gw_opt = None;
        let mut err_opt = None;

        match Node::start(repo_path, online) {
            Ok(node) => match node.start_gateway("/ip4/127.0.0.1/tcp/0") {
                Some(ma) => {
                    let port = parse_port(&ma).unwrap_or(0);
                    gw_opt = Some(format!("http://127.0.0.1:{port}"));
                    node_opt = Some(node);
                }
                None => {
                    err_opt = kubo_ffi::last_error();
                }
            },
            Err(e) => {
                err_opt = Some(e);
            }
        }

        let enabled = node_opt.is_some();
        Arc::new(Self {
            node: Mutex::new(node_opt),
            gateway: Mutex::new(gw_opt),
            enabled,
            last_error: Mutex::new(err_opt),
        })
    }
}

#[cfg(feature = "ipfs")]
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

#[cfg(feature = "ipfs")]
pub async fn add(
    Extension(state): Extension<Arc<IpfsState>>,
    Json(payload): Json<AddRequest>,
) -> impl IntoResponse {
    let guard = state.node.lock().unwrap();
    let Some(node) = guard.as_ref() else {
        let err = state.last_error.lock().unwrap().clone();
        return (StatusCode::SERVICE_UNAVAILABLE, err.unwrap_or_default()).into_response();
    };
    match node.unixfs_add_bytes(payload.data.as_bytes()) {
        Ok(cid) => Json(AddResponse { cid }).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

#[cfg(feature = "ipfs")]
pub async fn gateway(Extension(state): Extension<Arc<IpfsState>>) -> impl IntoResponse {
    let gw = state.gateway.lock().unwrap().clone();
    match gw {
        Some(url) => Json(GatewayResponse { url }).into_response(),
        None => (StatusCode::NOT_FOUND, "no gateway").into_response(),
    }
}

#[cfg(feature = "ipfs")]
pub async fn status(Extension(state): Extension<Arc<IpfsState>>) -> impl IntoResponse {
    let gw = state.gateway.lock().unwrap().clone();
    let err = state.last_error.lock().unwrap().clone();
    Json(IpfsStatus {
        enabled: state.enabled,
        gateway: gw,
        error: err,
    })
}
