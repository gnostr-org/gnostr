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
        .map(|s| {
            s.lines()
                .map(|l| l.to_string())
                .filter(|l| !l.is_empty())
                .collect()
        })
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

#[cfg(feature = "ipfs")]
pub const CID_ANNOUNCE_TOPIC: &str = "gnostr/ipfs/cid-announce";

#[cfg(feature = "ipfs")]
pub fn announce_message(cid: &str, peer_id: &str) -> String {
    format!(r#"{{"cid":"{cid}","peer":"{peer_id}"}}"#)
}

#[cfg(feature = "ipfs")]
pub fn fetch_block(state: &IpfsState, cid: &str) -> Option<Vec<u8>> {
    let guard = state.node.lock().ok()?;
    guard.as_ref()?.unixfs_cat(cid).ok()
}

#[cfg(feature = "ipfs")]
pub fn provide_block(state: &IpfsState, data: &[u8]) -> Option<String> {
    let guard = state.node.lock().ok()?;
    guard.as_ref()?.unixfs_add_bytes(data).ok()
}

#[cfg(feature = "ipfs")]
pub async fn fetch_via_gateway(gateway_url: &str, cid: &str) -> Option<Vec<u8>> {
    let url = format!("{}/ipfs/{}", gateway_url.trim_end_matches('/'), cid);
    let resp = reqwest::get(&url).await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    resp.bytes().await.ok().map(|b| b.to_vec())
}

#[cfg(all(test, feature = "ipfs"))]
mod tests {
    use super::*;
    use crate::ipfs::IpfsState;
    use tempfile::tempdir;

    #[test]
    #[ignore = "requires Go toolchain to build kubo-ffi"]
    fn block_roundtrip_interop() {
        let dir = tempdir().unwrap();
        let state = IpfsState::init(dir.path().to_str().unwrap(), false);
        assert!(state.enabled);
        let cid = provide_block(&state, b"interop payload").unwrap();
        let bytes = fetch_block(&state, &cid).unwrap();
        assert_eq!(bytes, b"interop payload");
    }

    use super::*;

    #[test]
    fn announce_message_contains_cid_and_peer() {
        let msg = announce_message("QmTest", "12D3KooW");
        assert!(msg.contains("QmTest"));
        assert!(msg.contains("12D3KooW"));
    }

    #[test]
    fn cid_announce_topic_nonempty() {
        assert!(!CID_ANNOUNCE_TOPIC.is_empty());
    }
}
