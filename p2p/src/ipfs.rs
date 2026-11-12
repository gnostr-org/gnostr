//! Optional embedded Kubo (IPFS) node sharing for the p2p plane.
//!
//! Only compiled with the `ipfs` feature.

#[cfg(feature = "ipfs")]
use std::sync::{Arc, Mutex, OnceLock};

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
impl IpfsState {
    pub fn init(repo_path: &str, online: bool) -> Arc<Self> {
        let _ = std::fs::create_dir_all(repo_path);
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
            repo_path: repo_path.to_string(),
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
static GLOBAL: OnceLock<Arc<IpfsState>> = OnceLock::new();

#[cfg(feature = "ipfs")]
pub fn init_global(repo_path: &str, online: bool) -> Arc<IpfsState> {
    let state = IpfsState::init(repo_path, online);
    let _ = GLOBAL.set(state.clone());
    state
}

#[cfg(feature = "ipfs")]
pub fn global() -> Option<Arc<IpfsState>> {
    GLOBAL.get().cloned()
}
