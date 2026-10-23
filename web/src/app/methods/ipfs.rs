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
    pub repo_path: String,
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
#[derive(serde::Deserialize)]
pub struct BatchAddRequest {
    pub items: Vec<String>,
}

#[cfg(feature = "ipfs")]
#[derive(Serialize)]
pub struct BatchAddResponse {
    pub cids: Vec<String>,
}

#[cfg(feature = "ipfs")]
#[derive(Serialize)]
pub struct GatewayResponse {
    pub url: String,
}

#[cfg(feature = "ipfs")]
impl IpfsState {
    pub fn init(repo_path: &str, online: bool) -> Arc<Self> {
        let repo_path = repo_path.to_string();
        let _ = std::fs::create_dir_all(&repo_path);
        // Best-effort init; ignore "already initialized" errors.
        let _ = kubo_ffi::init_repo(&repo_path);

        let mut node_opt = None;
        let mut gw_opt = None;
        let mut err_opt = None;

        match Node::start(&repo_path, online) {
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
            repo_path: repo_path.clone(),
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

#[cfg(feature = "ipfs")]
use axum::extract::Path as AxumPath;

#[cfg(feature = "ipfs")]
pub async fn cat(
    Extension(state): Extension<Arc<IpfsState>>,
    AxumPath(cid): AxumPath<String>,
) -> impl IntoResponse {
    let guard = state.node.lock().unwrap();
    let Some(node) = guard.as_ref() else {
        let err = state.last_error.lock().unwrap().clone();
        return (StatusCode::SERVICE_UNAVAILABLE, err.unwrap_or_default()).into_response();
    };
    match node.unixfs_cat(&cid) {
        Ok(bytes) => {
            let mut resp = axum::response::Response::new(axum::body::Body::from(bytes));
            resp.headers_mut().insert(
                axum::http::header::CONTENT_TYPE,
                axum::http::HeaderValue::from_static("application/octet-stream"),
            );
            resp
        }
        Err(e) => (StatusCode::NOT_FOUND, e).into_response(),
    }
}

#[cfg(feature = "ipfs")]
pub async fn pin(
    Extension(state): Extension<Arc<IpfsState>>,
    AxumPath(cid): AxumPath<String>,
) -> impl IntoResponse {
    let guard = state.node.lock().unwrap();
    let Some(node) = guard.as_ref() else {
        let err = state.last_error.lock().unwrap().clone();
        return (StatusCode::SERVICE_UNAVAILABLE, err.unwrap_or_default()).into_response();
    };
    let repo = state.repo_path.clone();
    match node.run_on_node(&repo, &["pin", "add", &cid]) {
        Ok(out) => Json(serde_json::json!({"cid": cid, "output": out})).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

#[cfg(feature = "ipfs")]
pub async fn ls(Extension(state): Extension<Arc<IpfsState>>) -> impl IntoResponse {
    let guard = state.node.lock().unwrap();
    let Some(node) = guard.as_ref() else {
        let err = state.last_error.lock().unwrap().clone();
        return (StatusCode::SERVICE_UNAVAILABLE, err.unwrap_or_default()).into_response();
    };
    let repo = state.repo_path.clone();
    match node.run_on_node(&repo, &["pin", "ls"]) {
        Ok(out) => Json(serde_json::json!({"output": out})).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

#[cfg(feature = "ipfs")]
pub async fn ipfs_proxy(
    Extension(state): Extension<Arc<IpfsState>>,
    AxumPath(cid): AxumPath<String>,
) -> impl IntoResponse {
    let gw = state.gateway.lock().unwrap().clone();
    let Some(base) = gw else {
        return (StatusCode::SERVICE_UNAVAILABLE, "no gateway").into_response();
    };
    // Redirect to the Kubo gateway (avoids streaming through axum).
    axum::response::Redirect::temporary(&format!("{base}/ipfs/{cid}")).into_response()
}

#[cfg(feature = "ipfs")]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct ManifestPublishRequest {
    pub repos: Vec<serde_json::Value>,
}

#[cfg(feature = "ipfs")]
pub async fn publish_manifest(
    Extension(state): Extension<Arc<IpfsState>>,
    Json(payload): Json<ManifestPublishRequest>,
) -> impl IntoResponse {
    let guard = state.node.lock().unwrap();
    let Some(node) = guard.as_ref() else {
        let err = state.last_error.lock().unwrap().clone();
        return (StatusCode::SERVICE_UNAVAILABLE, err.unwrap_or_default()).into_response();
    };
    let data = serde_json::to_vec(&payload).unwrap_or_default();
    match node.unixfs_add_bytes(&data) {
        Ok(cid) => Json(AddResponse { cid }).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}


#[cfg(feature = "ipfs")]
pub async fn add_batch(
    Extension(state): Extension<Arc<IpfsState>>,
    Json(payload): Json<BatchAddRequest>,
) -> impl IntoResponse {
    let guard = state.node.lock().unwrap();
    let Some(node) = guard.as_ref() else {
        let err = state.last_error.lock().unwrap().clone();
        return (StatusCode::SERVICE_UNAVAILABLE, err.unwrap_or_default()).into_response();
    };
    let mut cids = Vec::with_capacity(payload.items.len());
    for item in &payload.items {
        match node.unixfs_add_bytes(item.as_bytes()) {
            Ok(cid) => cids.push(cid),
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
        }
    }
    Json(BatchAddResponse { cids }).into_response()
}

#[cfg(feature = "ipfs")]
pub async fn peers(Extension(state): Extension<Arc<IpfsState>>) -> impl IntoResponse {
    let guard = state.node.lock().unwrap();
    let Some(node) = guard.as_ref() else {
        let err = state.last_error.lock().unwrap().clone();
        return (StatusCode::SERVICE_UNAVAILABLE, err.unwrap_or_default()).into_response();
    };
    Json(serde_json::json!({"peers": node.swarm_peers()})).into_response()
}

#[cfg(all(test, feature = "ipfs"))]
mod tests {
    use super::*;

    #[test]
    fn parse_port_extracts_tcp_port() {
        assert_eq!(parse_port("/ip4/127.0.0.1/tcp/5001"), Some(5001));
        assert_eq!(parse_port("/ip4/0.0.0.0/tcp/0"), Some(0));
        assert_eq!(parse_port("nonsense"), None);
    }

    #[test]
    fn status_serializes() {
        let s = IpfsStatus {
            enabled: true,
            gateway: Some("http://127.0.0.1:1234".into()),
            error: None,
        };
        let v = serde_json::to_value(s).unwrap();
        assert_eq!(v["enabled"], true);
        assert_eq!(v["gateway"], "http://127.0.0.1:1234");
    }

    #[test]
    #[ignore = "requires Go toolchain to build kubo-ffi"]
    fn node_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let state = IpfsState::init(dir.path().to_str().unwrap(), false);
        assert!(state.enabled, "kubo node should start: {:?}", state.last_error.lock().unwrap());
        let guard = state.node.lock().unwrap();
        let node = guard.as_ref().unwrap();
        let cid = node.unixfs_add_bytes(b"hello phase5").unwrap();
        let bytes = node.unixfs_cat(&cid).unwrap();
        assert_eq!(bytes, b"hello phase5");
    }
}
