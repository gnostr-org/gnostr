//! Bridge helpers between the embedded Kubo (IPFS) node and the libp2p swarm.
//!
//! Only compiled with the `ipfs` feature.

#[cfg(feature = "ipfs")]
use crate::ipfs::IpfsState;

#[cfg(feature = "ipfs")]
pub fn kubo_peer_id(state: &IpfsState) -> Option<String> {
    let guard = state.node.lock().ok()?;
    guard.as_ref().and_then(|n| n.peer_id())
}

#[cfg(feature = "ipfs")]
pub fn kubo_listening_addrs(state: &IpfsState) -> Vec<String> {
    let guard = match state.node.lock() {
        Ok(g) => g,
        Err(_) => return vec![],
    };
    guard
        .as_ref()
        .and_then(|n| n.listening_addrs())
        .map(|s| s.lines().map(|l| l.to_string()).filter(|l| !l.is_empty()).collect())
        .unwrap_or_default()
}

#[cfg(feature = "ipfs")]
pub fn kubo_swarm_peers(state: &IpfsState) -> Vec<String> {
    let guard = match state.node.lock() {
        Ok(g) => g,
        Err(_) => return vec![],
    };
    guard.as_ref().map(|n| n.swarm_peers()).unwrap_or_default()
}

#[cfg(feature = "ipfs")]
pub fn connect_kubo(state: &IpfsState, addr: &str) -> bool {
    let guard = match state.node.lock() {
        Ok(g) => g,
        Err(_) => return false,
    };
    guard.as_ref().map(|n| n.connect(addr)).unwrap_or(false)
}

#[cfg(feature = "ipfs")]
/// Bootstrap the p2p swarm's view of Kubo by returning Kubo's peer id and
/// listening addresses. Callers should dial these addresses from their swarm.
pub fn kubo_bootstrap_info(state: &IpfsState) -> (Option<String>, Vec<String>) {
    (kubo_peer_id(state), kubo_listening_addrs(state))
}
