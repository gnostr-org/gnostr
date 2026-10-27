#[cfg(feature = "ipfs")]
use std::sync::Arc;

#[cfg(feature = "ipfs")]
use axum::{extract::Extension, http::StatusCode, response::IntoResponse, Json};
#[cfg(feature = "ipfs")]
use futures_util::StreamExt;
#[cfg(feature = "ipfs")]
use serde::Serialize;
#[cfg(feature = "ipfs")]
use tokio::sync::Mutex;

#[cfg(feature = "ipfs")]
use kubo_ffi::{self, Node};

#[cfg(feature = "ipfs")]
pub struct IpfsState {
    pub repo_path: String,
    pub node: Mutex<Option<Node>>,
    pub gateway: Mutex<Option<String>>,
    pub api_url: Mutex<Option<String>>,
    pub enabled: bool,
    pub last_error: Mutex<Option<String>>,
}

#[cfg(feature = "ipfs")]
#[derive(Serialize)]
pub struct IpfsStatus {
    pub enabled: bool,
    pub gateway: Option<String>,
    pub api_url: Option<String>,
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
    /// Initialize IPFS support.
    ///
    /// First, probe for an already-running Kubo daemon (either via the repo's
    /// `api` file or the default local endpoint). If one is reachable we use it
    /// instead of starting a second node. Otherwise, initialize a fresh repo
    /// and start an embedded node.
    pub fn init(repo_path: &str, online: bool) -> Arc<Self> {
        let repo_path = repo_path.to_string();
        let _ = std::fs::create_dir_all(&repo_path);

        if let Some((api_url, gateway_url)) = detect_existing_daemon(&repo_path) {
            tracing::info!("Using existing IPFS daemon at {api_url}, gateway {gateway_url}");
            return Arc::new(Self {
                repo_path,
                node: Mutex::new(None),
                gateway: Mutex::new(Some(gateway_url)),
                api_url: Mutex::new(Some(api_url)),
                enabled: true,
                last_error: Mutex::new(None),
            });
        }

        // No running daemon found; initialize our own repo and start a node.
        let init_res = if online {
            kubo_ffi::init_repo_networked(&repo_path)
        } else {
            kubo_ffi::init_repo(&repo_path)
        };
        if let Err(e) = &init_res {
            tracing::debug!("IPFS repo init skipped: {e}");
        }

        let mut node_opt = None;
        let mut gw_opt = None;
        let mut err_opt = None;

        match Node::start(&repo_path, online) {
            Ok(node) => {
                // Full daemon API (CoreAPI + WebUI) for trusted/local use.
                let _ = node.start_api("/ip4/127.0.0.1/tcp/0");
                // Keep a public-safe gateway for /ipfs/:cid retrieval.
                match node.start_gateway("/ip4/127.0.0.1/tcp/0") {
                    Some(ma) => {
                        let port = parse_port(&ma).unwrap_or(0);
                        gw_opt = Some(format!("http://127.0.0.1:{port}"));
                        node_opt = Some(node);
                    }
                    None => {
                        err_opt = kubo_ffi::last_error();
                    }
                }
            }
            Err(e) => {
                err_opt = Some(e);
            }
        }

        let enabled = node_opt.is_some();
        Arc::new(Self {
            repo_path,
            node: Mutex::new(node_opt),
            gateway: Mutex::new(gw_opt),
            api_url: Mutex::new(None),
            enabled,
            last_error: Mutex::new(err_opt),
        })
    }

    pub async fn add_bytes(&self, data: &[u8]) -> Result<String, String> {
        {
            let guard = self.node.lock().await;
            if let Some(node) = guard.as_ref() {
                return node.unixfs_add_bytes(data);
            }
        }
        if let Some(api_url) = self.api_url.lock().await.clone() {
            return external_add(&api_url, data).await;
        }
        Err(self
            .last_error
            .lock()
            .await
            .clone()
            .unwrap_or_else(|| "IPFS not available".into()))
    }

    pub async fn cat_bytes(&self, cid: &str) -> Result<Vec<u8>, String> {
        {
            let guard = self.node.lock().await;
            if let Some(node) = guard.as_ref() {
                return node.unixfs_cat(cid);
            }
        }
        if let Some(api_url) = self.api_url.lock().await.clone() {
            return external_cat(&api_url, cid).await;
        }
        Err(self
            .last_error
            .lock()
            .await
            .clone()
            .unwrap_or_else(|| "IPFS not available".into()))
    }

    pub async fn run_cmd(&self, args: &[&str]) -> Result<String, String> {
        {
            let guard = self.node.lock().await;
            if let Some(node) = guard.as_ref() {
                return node.run_on_node(&self.repo_path, args);
            }
        }
        if let Some(api_url) = self.api_url.lock().await.clone() {
            return match args {
                ["pin", "add", cid] => external_pin_add(&api_url, cid).await,
                ["pin", "ls"] => external_pin_ls(&api_url).await,
                _ => Err("unsupported external IPFS command".into()),
            };
        }
        Err(self
            .last_error
            .lock()
            .await
            .clone()
            .unwrap_or_else(|| "IPFS not available".into()))
    }

    pub async fn peers(&self) -> Vec<String> {
        {
            let guard = self.node.lock().await;
            if let Some(node) = guard.as_ref() {
                return node.swarm_peers();
            }
        }
        if let Some(api_url) = self.api_url.lock().await.clone() {
            return external_swarm_peers(&api_url).await;
        }
        vec![]
    }
}

#[cfg(feature = "ipfs")]
fn ipfs_probe_agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(3))
        .build()
}

#[cfg(feature = "ipfs")]
fn detect_existing_daemon(repo_path: &str) -> Option<(String, String)> {
    let api_path = std::path::Path::new(repo_path).join("api");
    let mut candidates: Vec<String> = Vec::new();

    if let Ok(content) = std::fs::read_to_string(&api_path) {
        let ma = content.trim();
        if !ma.is_empty() {
            if let Some(url) = multiaddr_to_http_url(ma) {
                candidates.push(url);
            }
        }
    }

    // Also probe the default local API endpoint.
    candidates.push("http://127.0.0.1:5001".to_string());

    let agent = ipfs_probe_agent();
    for url in candidates {
        let version_url = format!("{url}/api/v0/version");
        match agent.post(&version_url).call() {
            Ok(resp) if resp.status() == 200 => {
                let gateway = fetch_gateway_url(&agent, &url)
                    .unwrap_or_else(|| "http://127.0.0.1:8080".to_string());
                return Some((url, gateway));
            }
            _ => continue,
        }
    }
    None
}

#[cfg(feature = "ipfs")]
fn fetch_gateway_url(agent: &ureq::Agent, api_url: &str) -> Option<String> {
    let url = format!("{api_url}/api/v0/config?arg=Addresses.Gateway");
    let resp = agent.post(&url).call().ok()?;
    if resp.status() != 200 {
        return None;
    }
    let json: serde_json::Value = resp.into_json().ok()?;
    let value = json.get("Value")?.as_str()?;
    multiaddr_to_http_url(value)
}

#[cfg(feature = "ipfs")]
fn multiaddr_to_http_url(ma: &str) -> Option<String> {
    let parts: Vec<&str> = ma.split('/').collect();
    let mut host = None;
    let mut port = None;
    let mut i = 0;
    while i < parts.len() {
        match parts[i] {
            "ip4" | "ip6" => {
                if i + 1 < parts.len() {
                    host = Some(parts[i + 1]);
                }
            }
            "tcp" => {
                if i + 1 < parts.len() {
                    port = parts[i + 1].parse().ok();
                }
            }
            _ => {}
        }
        i += 1;
    }
    let host = host?;
    let port: u16 = port?;
    Some(format!("http://{host}:{port}"))
}

#[cfg(feature = "ipfs")]
async fn external_add(api_url: &str, data: &[u8]) -> Result<String, String> {
    let client = reqwest::Client::new();
    let url = format!("{api_url}/api/v0/add");
    let part = reqwest::multipart::Part::bytes(data.to_vec()).file_name("file");
    let form = reqwest::multipart::Form::new().part("file", part);
    let resp = client
        .post(&url)
        .multipart(form)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("add failed: {}", resp.status()));
    }
    let text = resp.text().await.map_err(|e| e.to_string())?;
    let last = text.lines().last().unwrap_or(&text);
    let json: serde_json::Value = serde_json::from_str(last).map_err(|e| e.to_string())?;
    json.get("Hash")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "add response missing Hash".to_string())
}

#[cfg(feature = "ipfs")]
async fn external_cat(api_url: &str, cid: &str) -> Result<Vec<u8>, String> {
    let client = reqwest::Client::new();
    let url = format!("{api_url}/api/v0/cat?arg={cid}");
    let resp = client.post(&url).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("cat failed: {}", resp.status()));
    }
    resp.bytes()
        .await
        .map(|b| b.to_vec())
        .map_err(|e| e.to_string())
}

#[cfg(feature = "ipfs")]
async fn external_pin_add(api_url: &str, cid: &str) -> Result<String, String> {
    let client = reqwest::Client::new();
    let url = format!("{api_url}/api/v0/pin/add?arg={cid}");
    let resp = client.post(&url).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("pin add failed: {}", resp.status()));
    }
    resp.text().await.map_err(|e| e.to_string())
}

#[cfg(feature = "ipfs")]
async fn external_pin_ls(api_url: &str) -> Result<String, String> {
    let client = reqwest::Client::new();
    let url = format!("{api_url}/api/v0/pin/ls?stream=true");
    let resp = client.post(&url).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("pin ls failed: {}", resp.status()));
    }
    resp.text().await.map_err(|e| e.to_string())
}

#[cfg(feature = "ipfs")]
async fn external_swarm_peers(api_url: &str) -> Vec<String> {
    let client = reqwest::Client::new();
    let url = format!("{api_url}/api/v0/swarm/peers");
    match client.post(&url).send().await {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                if let Some(peers) = json.get("Peers").and_then(|p| p.as_array()) {
                    return peers
                        .iter()
                        .filter_map(|p| {
                            p.get("Peer")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string())
                        })
                        .collect();
                }
            }
        }
        _ => {}
    }
    vec![]
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
    match state.add_bytes(payload.data.as_bytes()).await {
        Ok(cid) => Json(AddResponse { cid }).into_response(),
        Err(e) => (StatusCode::SERVICE_UNAVAILABLE, e).into_response(),
    }
}

#[cfg(feature = "ipfs")]
pub async fn gateway(Extension(state): Extension<Arc<IpfsState>>) -> impl IntoResponse {
    let gw = state.gateway.lock().await.clone();
    match gw {
        Some(url) => Json(GatewayResponse { url }).into_response(),
        None => (StatusCode::NOT_FOUND, "no gateway").into_response(),
    }
}

#[cfg(feature = "ipfs")]
pub async fn status(Extension(state): Extension<Arc<IpfsState>>) -> impl IntoResponse {
    let gw = state.gateway.lock().await.clone();
    let api = state.api_url.lock().await.clone();
    let err = state.last_error.lock().await.clone();
    Json(IpfsStatus {
        enabled: state.enabled,
        gateway: gw,
        api_url: api,
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
    match state.cat_bytes(&cid).await {
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
    match state.run_cmd(&["pin", "add", &cid]).await {
        Ok(out) => Json(serde_json::json!({"cid": cid, "output": out})).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

#[cfg(feature = "ipfs")]
pub async fn ls(Extension(state): Extension<Arc<IpfsState>>) -> impl IntoResponse {
    match state.run_cmd(&["pin", "ls"]).await {
        Ok(out) => Json(serde_json::json!({"output": out})).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

#[cfg(feature = "ipfs")]
pub async fn ipfs_proxy(
    Extension(state): Extension<Arc<IpfsState>>,
    AxumPath(ipfs_path): AxumPath<String>,
) -> impl IntoResponse {
    let gw = state.gateway.lock().await.clone();
    let Some(base) = gw else {
        return (StatusCode::SERVICE_UNAVAILABLE, "no gateway").into_response();
    };

    let url = format!("{base}/ipfs/{ipfs_path}");
    let resp = match reqwest::get(&url).await {
        Ok(resp) => resp,
        Err(e) => {
            return (StatusCode::BAD_GATEWAY, e.to_string()).into_response();
        }
    };

    let status = axum::http::StatusCode::from_u16(resp.status().as_u16()).unwrap_or(StatusCode::OK);
    let mut builder = axum::response::Response::builder().status(status);

    for header in [
        axum::http::header::CONTENT_TYPE,
        axum::http::header::CONTENT_LENGTH,
        axum::http::header::ACCEPT_RANGES,
        axum::http::header::CACHE_CONTROL,
    ] {
        if let Some(value) = resp.headers().get(&header) {
            builder = builder.header(header, value);
        }
    }

    let stream = resp
        .bytes_stream()
        .map(|result| result.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)));
    let body = axum::body::Body::from_stream(stream);

    builder
        .body(body)
        .unwrap_or_else(|_| (StatusCode::INTERNAL_SERVER_ERROR, "proxy error").into_response())
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
    let data = serde_json::to_vec(&payload).unwrap_or_default();
    match state.add_bytes(&data).await {
        Ok(cid) => Json(AddResponse { cid }).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

#[cfg(feature = "ipfs")]
pub async fn add_batch(
    Extension(state): Extension<Arc<IpfsState>>,
    Json(payload): Json<BatchAddRequest>,
) -> impl IntoResponse {
    let mut cids = Vec::with_capacity(payload.items.len());
    for item in &payload.items {
        match state.add_bytes(item.as_bytes()).await {
            Ok(cid) => cids.push(cid),
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
        }
    }
    Json(BatchAddResponse { cids }).into_response()
}

#[cfg(feature = "ipfs")]
pub async fn peers(Extension(state): Extension<Arc<IpfsState>>) -> impl IntoResponse {
    Json(serde_json::json!({"peers": state.peers().await })).into_response()
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
    fn multiaddr_to_http_url_works() {
        assert_eq!(
            multiaddr_to_http_url("/ip4/127.0.0.1/tcp/5001"),
            Some("http://127.0.0.1:5001".into())
        );
        assert_eq!(multiaddr_to_http_url("nonsense"), None);
    }

    #[test]
    fn status_serializes() {
        let s = IpfsStatus {
            enabled: true,
            gateway: Some("http://127.0.0.1:1234".into()),
            api_url: Some("http://127.0.0.1:5001".into()),
            error: None,
        };
        let v = serde_json::to_value(s).unwrap();
        assert_eq!(v["enabled"], true);
        assert_eq!(v["gateway"], "http://127.0.0.1:1234");
        assert_eq!(v["api_url"], "http://127.0.0.1:5001");
    }

    #[test]
    #[ignore = "requires Go toolchain to build kubo-ffi"]
    fn node_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let state = IpfsState::init(dir.path().to_str().unwrap(), false);
        assert!(
            state.enabled,
            "kubo node should start: {:?}",
            state.last_error.blocking_lock()
        );
        let guard = state.node.blocking_lock();
        let node = guard.as_ref().unwrap();
        let cid = node.unixfs_add_bytes(b"hello phase5").unwrap();
        let bytes = node.unixfs_cat(&cid).unwrap();
        assert_eq!(bytes, b"hello phase5");
    }
}
