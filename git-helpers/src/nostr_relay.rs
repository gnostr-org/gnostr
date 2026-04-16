//! Minimal Nostr relay WebSocket client for NIP-34 repo resolution.
//!
//! Queries a relay for kind:30617 (RepoAnnounce) events to find the
//! GRASP HTTP server URL for a given npub + repo.

use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::collections::HashMap;

// ── Well-known public relays (fallback when none specified) ────────────────

/// Ordered list of well-known public relays tried as fallbacks when the
/// primary relay fails or is not specified.
pub const DEFAULT_RELAY_FALLBACKS: &[&str] = &[
    "wss://relay.damus.io",
    "wss://relay.nostr.band",
    "wss://nos.lol",
    "wss://relay.primal.net",
    "wss://nostr.wine",
    "wss://relay.snort.social",
];

// ── Relay URL normalisation ────────────────────────────────────────────────

/// Normalise a relay host/URL string to a full `wss://` (or `ws://`) URL.
///
/// Handles all common variations:
/// - `relay.damus.io`         → `wss://relay.damus.io`
/// - `wss://relay.damus.io`   → unchanged
/// - `ws://localhost:7777`    → unchanged
/// - `https://relay.example`  → `wss://relay.example`  (scheme swap)
/// - `http://localhost:7777`  → `ws://localhost:7777`   (scheme swap)
pub fn normalize_relay_url(raw: &str) -> String {
    if raw.starts_with("wss://") || raw.starts_with("ws://") {
        raw.to_string()
    } else if let Some(host) = raw.strip_prefix("https://") {
        format!("wss://{host}")
    } else if let Some(host) = raw.strip_prefix("http://") {
        format!("ws://{host}")
    } else {
        // bare hostname / host:port
        format!("wss://{raw}")
    }
}

// ── Relay query ────────────────────────────────────────────────────────────

/// Query a single Nostr relay for the NIP-34 kind:30617 event.
/// Returns the GRASP clone/web URL if found.
pub fn resolve_grasp_url(relay_url: &str, pubkey_hex: &str, repo: &str) -> Result<String> {
    let rt = tokio::runtime::Runtime::new().context("tokio runtime")?;
    rt.block_on(async move { ws_query(relay_url, pubkey_hex, repo).await })
}

/// Try each relay in `relays` in order, returning the first successful
/// GRASP URL resolution.  Logs a warning for each failing relay.
pub fn resolve_grasp_url_with_fallbacks(
    relays: &[String],
    pubkey_hex: &str,
    repo: &str,
) -> Result<String> {
    let mut last_err = anyhow::anyhow!("no relays to try");

    for relay in relays {
        eprintln!("[nostr] trying relay {}…", relay);
        match resolve_grasp_url(relay, pubkey_hex, repo) {
            Ok(url) => {
                eprintln!("[nostr] resolved via {relay} → {url}");
                return Ok(url);
            }
            Err(e) => {
                eprintln!("[nostr] relay {relay} failed: {e:#}");
                last_err = e;
            }
        }
    }

    Err(last_err).context(format!(
        "no NIP-34 kind:30617 event found for repo '{}' on any relay ({} tried)",
        repo,
        relays.len()
    ))
}

/// Build the ordered relay list for a query: primary relay first, then
/// fallbacks (deduped).
pub fn build_relay_list(primary: Option<&str>) -> Vec<String> {
    let mut relays: Vec<String> = Vec::new();

    if let Some(p) = primary {
        let norm = normalize_relay_url(p);
        if !norm.is_empty() {
            relays.push(norm);
        }
    }

    for &r in DEFAULT_RELAY_FALLBACKS {
        let norm = normalize_relay_url(r);
        if !relays.contains(&norm) {
            relays.push(norm);
        }
    }

    relays
}

async fn ws_query(relay_url: &str, pubkey_hex: &str, repo: &str) -> Result<String> {
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::{connect_async, tungstenite::Message};

    let (mut ws, _) = connect_async(relay_url)
        .await
        .with_context(|| format!("connect to relay {relay_url}"))?;

    // REQ filter: kind:30617, authored by pubkey, d-tag = repo
    let sub_id = "blossom-git-1";
    let filter = serde_json::json!({
        "kinds": [30617],
        "authors": [pubkey_hex],
        "#d": [repo]
    });
    let req_msg = serde_json::json!(["REQ", sub_id, filter]).to_string();
    ws.send(Message::Text(req_msg.into())).await.context("send REQ")?;

    // Read until we get an EVENT or EOSE
    let timeout = tokio::time::Duration::from_secs(10);
    let mut result: Option<String> = None;

    let _ = tokio::time::timeout(timeout, async {
        while let Some(msg) = ws.next().await {
            let Ok(Message::Text(text)) = msg else { continue };
            let Ok(Value::Array(arr)) = serde_json::from_str::<Value>(&text) else { continue };

            match arr.first().and_then(|v| v.as_str()) {
                Some("EVENT") => {
                    if let Some(event) = arr.get(2) {
                        if let Some(url) = extract_web_url(event) {
                            result = Some(url);
                            break;
                        }
                    }
                }
                Some("EOSE") => break,
                Some("NOTICE") => {
                    if let Some(msg) = arr.get(1).and_then(|v| v.as_str()) {
                        eprintln!("[nostr] relay notice: {msg}");
                    }
                }
                _ => {}
            }
        }
    })
    .await;

    let _ = ws.close(None).await;

    result.with_context(|| {
        format!(
            "no NIP-34 kind:30617 event found for {pubkey_hex:.8}…/{repo} on {relay_url}"
        )
    })
}

/// Extract a GRASP clone/web URL from a NIP-34 kind:30617 event's tags.
///
/// Tag priority:
/// 1. `["clone", "<url>", ...]` — first HTTPS URL in a clone tag
/// 2. `["web",   "<url>"]`     — fallback if no clone tag present
///
/// The `clone` tag may list multiple URLs (all in the same tag array).  We
/// return the first one that starts with `https://` or `http://`.
fn extract_web_url(event: &Value) -> Option<String> {
    let tags = event["tags"].as_array()?;

    // Try all clone tags first — pick first HTTP(S) value in each
    for tag in tags {
        let arr = tag.as_array()?;
        if arr.first().and_then(|v| v.as_str()) != Some("clone") {
            continue;
        }
        for val in arr.iter().skip(1) {
            if let Some(url) = val.as_str() {
                if url.starts_with("https://") || url.starts_with("http://") {
                    return Some(url.to_string());
                }
            }
        }
    }

    // Fall back to first web tag
    for tag in tags {
        let arr = tag.as_array()?;
        if arr.first().and_then(|v| v.as_str()) == Some("web") {
            if let Some(url) = arr.get(1).and_then(|v| v.as_str()) {
                if url.starts_with("https://") || url.starts_with("http://") {
                    return Some(url.to_string());
                }
            }
        }
    }

    None
}

/// Extract relay URLs from the `relays` tag of a kind:30617 event.
///
/// These can supplement the query relay list for subsequent fallback attempts.
/// Trailing slashes are stripped and URLs are normalised.
pub fn extract_event_relays(event: &Value) -> Vec<String> {
    let Some(tags) = event["tags"].as_array() else {
        return vec![];
    };

    for tag in tags {
        let Some(arr) = tag.as_array() else { continue };
        if arr.first().and_then(|v| v.as_str()) != Some("relays") {
            continue;
        }
        return arr
            .iter()
            .skip(1)
            .filter_map(|v| v.as_str())
            .map(|r| normalize_relay_url(r.trim_end_matches('/')))
            .collect();
    }
    vec![]
}

// ── NIP-19 npub decoder ────────────────────────────────────────────────────

/// Decode an `npub1…` bech32 string to a 32-byte public key and return
/// its lowercase hex representation.  Also accepts a raw 64-char hex string.
pub fn npub_to_hex(npub: &str) -> Result<String> {
    if npub.starts_with("npub1") {
        let (_hrp, data) = bech32::decode(npub).context("bech32 decode npub")?;
        let bytes: Vec<u8> = data;
        if bytes.len() != 32 {
            bail!("npub decoded to {} bytes, expected 32", bytes.len());
        }
        Ok(hex::encode(&bytes))
    } else if npub.len() == 64 && npub.chars().all(|c| c.is_ascii_hexdigit()) {
        Ok(npub.to_lowercase())
    } else {
        bail!("expected npub1… bech32 or 64-char hex pubkey, got: {npub}");
    }
}

// ── NIP-34 multi-kind helpers ─────────────────────────────────────────────

/// Parse the `a` tag from any NIP-34 event that references a repository.
///
/// The `a` tag has one of two formats:
/// - `"30617:<pubkey_hex>:<repo-id>"` — standard address
/// - `"<npub>/<repo-id>"` — seen in some 1631 status events
///
/// Returns `(pubkey_hex, repo_id)` on success.
pub fn extract_repo_addr(event: &Value) -> Option<(String, String)> {
    let tags = event["tags"].as_array()?;
    for tag in tags {
        let arr = tag.as_array()?;
        if arr.first().and_then(|v| v.as_str()) != Some("a") {
            continue;
        }
        let val = arr.get(1).and_then(|v| v.as_str())?;
        // Standard: "30617:<pubkey>:<repo-id>"
        let parts: Vec<&str> = val.splitn(3, ':').collect();
        if parts.len() == 3 {
            return Some((parts[1].to_string(), parts[2].to_string()));
        }
        // Fallback: "npub1.../<repo-id>" (as seen in some status events)
        if let Some((npub, repo)) = val.split_once('/') {
            if let Ok(hex) = npub_to_hex(npub) {
                return Some((hex, repo.to_string()));
            }
        }
    }
    None
}

/// Extract the current HEAD branch name from a kind:30618 (RepoState) event.
///
/// Reads the `["HEAD", "ref: refs/heads/<branch>"]` tag.
/// Returns the bare branch name, e.g. `"main"`.
pub fn extract_head_branch(event: &Value) -> Option<String> {
    let tags = event["tags"].as_array()?;
    for tag in tags {
        let arr = tag.as_array()?;
        if arr.first().and_then(|v| v.as_str()) != Some("HEAD") {
            continue;
        }
        let val = arr.get(1).and_then(|v| v.as_str())?;
        // "ref: refs/heads/main"
        let branch = val
            .strip_prefix("ref: refs/heads/")
            .unwrap_or(val);
        return Some(branch.to_string());
    }
    None
}

/// Extract all git refs from a kind:30618 (RepoState) event.
///
/// Returns a map of `"refs/heads/<branch>"` / `"refs/tags/<tag>"` → commit SHA.
pub fn extract_refs(event: &Value) -> HashMap<String, String> {
    let mut out = HashMap::new();
    let Some(tags) = event["tags"].as_array() else {
        return out;
    };
    for tag in tags {
        let Some(arr) = tag.as_array() else { continue };
        let Some(key) = arr.first().and_then(|v| v.as_str()) else { continue };
        if !key.starts_with("refs/") {
            continue;
        }
        if let Some(sha) = arr.get(1).and_then(|v| v.as_str()) {
            out.insert(key.to_string(), sha.to_string());
        }
    }
    out
}

/// Map a NIP-34 status kind number to a human-readable label.
///
/// - `1630` → `"open"`
/// - `1631` → `"applied"`
/// - `1632` → `"closed"`
/// - `1633` → `"draft"`
/// - anything else → `"unknown"`
pub fn status_kind_str(kind: u64) -> &'static str {
    match kind {
        1630 => "open",
        1631 => "applied",
        1632 => "closed",
        1633 => "draft",
        _ => "unknown",
    }
}

/// Extract the subject/title from a kind:1618 (PR) or kind:1621 (Issue) event.
///
/// Reads the first `["subject", "<text>"]` tag.
pub fn extract_subject(event: &Value) -> Option<String> {
    let tags = event["tags"].as_array()?;
    for tag in tags {
        let arr = tag.as_array()?;
        if arr.first().and_then(|v| v.as_str()) == Some("subject") {
            return arr.get(1).and_then(|v| v.as_str()).map(|s| s.to_string());
        }
    }
    None
}

/// Extract the commit SHA tip from a NIP-34 event.
///
/// - kind:1617 (Patch): reads the `r` tag (earliest unique commit)
/// - kind:1618 (PR): reads the `c` tag (commit tip)
/// - kind:1631 (Applied): reads the `merge-commit` tag
pub fn extract_tip_commit(event: &Value) -> Option<String> {
    let tags = event["tags"].as_array()?;
    // Try c tag (PR tip), then merge-commit, then r tag (patch commit ref)
    for needle in &["c", "merge-commit", "r"] {
        for tag in tags {
            let arr = tag.as_array()?;
            if arr.first().and_then(|v| v.as_str()) == Some(needle) {
                if let Some(sha) = arr.get(1).and_then(|v| v.as_str()) {
                    if sha.len() == 40 && sha.chars().all(|c| c.is_ascii_hexdigit()) {
                        return Some(sha.to_string());
                    }
                }
            }
        }
    }
    None
}

/// Extract labels from `t` tags on a kind:1621 (Issue) or kind:1618 (PR) event.
pub fn extract_labels(event: &Value) -> Vec<String> {
    let Some(tags) = event["tags"].as_array() else {
        return vec![];
    };
    tags.iter()
        .filter_map(|tag| {
            let arr = tag.as_array()?;
            if arr.first().and_then(|v| v.as_str()) != Some("t") {
                return None;
            }
            arr.get(1).and_then(|v| v.as_str()).map(|s| s.to_string())
        })
        .collect()
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Real event fixtures (from `gnostr query --kinds 30617 --limit 10`) ─

    /// YouBlossom — clone tag has TWO URLs; web tag is a subdomain.
    fn event_youblossom() -> Value {
        serde_json::json!({
            "kind": 30617,
            "pubkey": "5c1eeccff05aa3ff47bc56fa80bc5c254a8eb67c3a8be2d29bf9b142aa57a7da",
            "tags": [
                ["d", "YouBlossom"],
                ["name", "YouBlossom"],
                ["clone",
                    "https://git.shakespeare.diy/npub1ts0wenlst23l73au2magp0zuy49gadnu8297955mlxc592jh5ldq0xzwcx/YouBlossom.git",
                    "https://relay.ngit.dev/npub1ts0wenlst23l73au2magp0zuy49gadnu8297955mlxc592jh5ldq0xzwcx/YouBlossom.git"
                ],
                ["relays",
                    "wss://git.shakespeare.diy/",
                    "wss://relay.ngit.dev/",
                    "wss://nos.lol/",
                    "wss://relay.damus.io/",
                    "wss://relay.primal.net/"
                ],
                ["web", "https://YouBlossom.shakespeare.wtf"],
                ["web", "https://YouBlossom.shakespeare.wtf"],
                ["alt", "git repository: YouBlossom"]
            ]
        })
    }

    /// fresh-repo — single clone URL; relay list is a single relay.
    fn event_fresh_repo() -> Value {
        serde_json::json!({
            "kind": 30617,
            "pubkey": "b1576eb99a4774158a32fc5e190afa3ded4da19f51fbfa0b1a1bf6421ea5733a",
            "tags": [
                ["d", "fresh-repo"],
                ["name", "fresh-repo"],
                ["clone", "https://blossom.gnostr.cloud/npub1k9tkawv6ga6ptz3jl30pjzh68hk5mgvl28al5zc6r0myy849wvaq38a70g/fresh-repo.git"],
                ["web", "https://gitworkshop.dev/repo/fresh-repo"],
                ["relays", "wss://blossom.gnostr.cloud"],
                ["alt", "git repository: fresh-repo"]
            ]
        })
    }

    /// jmp — two clone URLs; relays with no trailing slashes.
    fn event_jmp() -> Value {
        serde_json::json!({
            "kind": 30617,
            "pubkey": "7459d333af66066f066cf87796e690db3a96ff4534f9edf4eab74df2f207289b",
            "tags": [
                ["d", "jmp"],
                ["name", "jmp"],
                ["description", "JoinMarket Protocol (JMP) Specifications"],
                ["clone",
                    "https://relay.ngit.dev/npub1w3vaxva0vcrx7pnvlpmede5smvafdl69xnu7ma82kaxl9us89zdsht4c5c/jmp.git",
                    "https://gitnostr.com/npub1w3vaxva0vcrx7pnvlpmede5smvafdl69xnu7ma82kaxl9us89zdsht4c5c/jmp.git"
                ],
                ["web", "https://gitworkshop.dev/npub1w3vaxva0vcrx7pnvlpmede5smvafdl69xnu7ma82kaxl9us89zdsht4c5c/relay.ngit.dev/jmp"],
                ["relays", "wss://relay.ngit.dev", "wss://gitnostr.com"]
            ]
        })
    }

    /// Beer — NO clone tag, only web tags; relays have trailing slashes.
    fn event_beer() -> Value {
        serde_json::json!({
            "kind": 30617,
            "pubkey": "c62ea154ea5352df528b9bb79fdfcd0432636371098d4336943ace394a70b555",
            "tags": [
                ["d", "Beer"],
                ["name", "iDrink"],
                ["relays",
                    "wss://git.shakespeare.diy/",
                    "wss://relay.ngit.dev/",
                    "wss://relay.primal.net/",
                    "wss://relay.damus.io/",
                    "wss://relay.westernbtc.com/"
                ],
                ["web", "https://iBeer.shakespeare.wtf"],
                ["alt", "git repository: Beer"]
            ]
        })
    }

    /// satshoot — two clone URLs from different GRASP servers.
    fn event_satshoot() -> Value {
        serde_json::json!({
            "kind": 30617,
            "pubkey": "d04ecf33a303a59852fdb681ed8b412201ba85d8d2199aec73cb62681d62aa90",
            "tags": [
                ["d", "satshoot"],
                ["name", "satshoot"],
                ["clone",
                    "https://grasp.budabit.club/npub16p8v7varqwjes5hak6q7mz6pygqm4pwc6gve4mrned3xs8tz42gq7kfhdw/satshoot.git",
                    "https://gitnostr.com/npub16p8v7varqwjes5hak6q7mz6pygqm4pwc6gve4mrned3xs8tz42gq7kfhdw/satshoot.git"
                ],
                ["web", "https://gitnostr.com/npub16p8v7varqwjes5hak6q7mz6pygqm4pwc6gve4mrned3xs8tz42gq7kfhdw/satshoot"],
                ["relays", "wss://gitnostr.com", "wss://relay.primal.net", "wss://nos.lol", "wss://relay.damus.io", "wss://grasp.budabit.club"]
            ]
        })
    }

    /// Minimal event — no clone, no web, no relays tags.
    fn event_empty() -> Value {
        serde_json::json!({
            "kind": 30617,
            "tags": [["d", "empty-repo"], ["name", "empty"]]
        })
    }

    // ── extract_web_url ───────────────────────────────────────────────────

    #[test]
    fn multi_clone_returns_first_url() {
        let url = extract_web_url(&event_youblossom()).unwrap();
        eprintln!("multi_clone_returns_first_url => {url}");
        assert_eq!(
            url,
            "https://git.shakespeare.diy/npub1ts0wenlst23l73au2magp0zuy49gadnu8297955mlxc592jh5ldq0xzwcx/YouBlossom.git"
        );
    }

    #[test]
    fn single_clone_returned() {
        let url = extract_web_url(&event_fresh_repo()).unwrap();
        eprintln!("single_clone_returned => {url}");
        assert_eq!(
            url,
            "https://blossom.gnostr.cloud/npub1k9tkawv6ga6ptz3jl30pjzh68hk5mgvl28al5zc6r0myy849wvaq38a70g/fresh-repo.git"
        );
    }

    #[test]
    fn clone_takes_priority_over_web() {
        let url = extract_web_url(&event_jmp()).unwrap();
        eprintln!("clone_takes_priority_over_web => {url}");
        assert!(url.starts_with("https://relay.ngit.dev/"), "expected ngit clone, got: {url}");
    }

    #[test]
    fn falls_back_to_web_when_no_clone() {
        let url = extract_web_url(&event_beer()).unwrap();
        eprintln!("falls_back_to_web_when_no_clone => {url}");
        assert_eq!(url, "https://iBeer.shakespeare.wtf");
    }

    #[test]
    fn satshoot_first_clone_url() {
        let url = extract_web_url(&event_satshoot()).unwrap();
        eprintln!("satshoot_first_clone_url => {url}");
        assert!(
            url.starts_with("https://grasp.budabit.club/"),
            "expected budabit.club as first clone, got: {url}"
        );
    }

    #[test]
    fn empty_event_returns_none() {
        let result = extract_web_url(&event_empty());
        eprintln!("empty_event_returns_none => {result:?}");
        assert!(result.is_none());
    }

    // ── extract_event_relays ──────────────────────────────────────────────

    #[test]
    fn extract_relays_from_youblossom() {
        let relays = extract_event_relays(&event_youblossom());
        eprintln!("extract_relays_from_youblossom => {relays:?}");
        assert!(relays.contains(&"wss://git.shakespeare.diy".to_string()));
        assert!(relays.contains(&"wss://relay.ngit.dev".to_string()));
        assert!(relays.contains(&"wss://nos.lol".to_string()));
        assert_eq!(relays.len(), 5);
    }

    #[test]
    fn extract_relays_trailing_slashes_stripped() {
        let relays = extract_event_relays(&event_youblossom());
        eprintln!("extract_relays_trailing_slashes_stripped => {relays:?}");
        for r in &relays {
            assert!(!r.ends_with('/'), "relay has trailing slash: {r}");
        }
    }

    #[test]
    fn extract_relays_single_entry() {
        let relays = extract_event_relays(&event_fresh_repo());
        eprintln!("extract_relays_single_entry => {relays:?}");
        assert_eq!(relays, vec!["wss://blossom.gnostr.cloud"]);
    }

    #[test]
    fn extract_relays_no_tag_returns_empty() {
        let relays = extract_event_relays(&event_empty());
        eprintln!("extract_relays_no_tag_returns_empty => {relays:?}");
        assert!(relays.is_empty());
    }

    #[test]
    fn extract_relays_normalises_bare_hosts() {
        let event = serde_json::json!({
            "tags": [["relays", "relay.damus.io", "nos.lol"]]
        });
        let relays = extract_event_relays(&event);
        eprintln!("extract_relays_normalises_bare_hosts => {relays:?}");
        assert_eq!(relays[0], "wss://relay.damus.io");
        assert_eq!(relays[1], "wss://nos.lol");
    }

    // ── normalize_relay_url ───────────────────────────────────────────────

    #[test]
    fn normalize_bare_hostname() {
        let out = normalize_relay_url("relay.damus.io");
        eprintln!("normalize_bare_hostname: relay.damus.io => {out}");
        assert_eq!(out, "wss://relay.damus.io");
    }

    #[test]
    fn normalize_bare_host_port() {
        let out = normalize_relay_url("localhost:7777");
        eprintln!("normalize_bare_host_port: localhost:7777 => {out}");
        assert_eq!(out, "wss://localhost:7777");
    }

    #[test]
    fn normalize_wss_passthrough() {
        let out = normalize_relay_url("wss://relay.nostr.band");
        eprintln!("normalize_wss_passthrough: wss://relay.nostr.band => {out}");
        assert_eq!(out, "wss://relay.nostr.band");
    }

    #[test]
    fn normalize_ws_passthrough() {
        let out = normalize_relay_url("ws://localhost:7777");
        eprintln!("normalize_ws_passthrough: ws://localhost:7777 => {out}");
        assert_eq!(out, "ws://localhost:7777");
    }

    #[test]
    fn normalize_https_to_wss() {
        let out = normalize_relay_url("https://relay.example.com");
        eprintln!("normalize_https_to_wss: https://relay.example.com => {out}");
        assert_eq!(out, "wss://relay.example.com");
    }

    #[test]
    fn normalize_http_to_ws() {
        let out = normalize_relay_url("http://localhost:7777");
        eprintln!("normalize_http_to_ws: http://localhost:7777 => {out}");
        assert_eq!(out, "ws://localhost:7777");
    }

    #[test]
    fn normalize_trailing_slash_handled_by_caller() {
        let r = normalize_relay_url("wss://relay.damus.io/");
        eprintln!("normalize_trailing_slash_handled_by_caller: wss://relay.damus.io/ => {r}");
        assert_eq!(r, "wss://relay.damus.io/");
    }

    // ── build_relay_list ──────────────────────────────────────────────────

    #[test]
    fn relay_list_primary_first() {
        let list = build_relay_list(Some("wss://my-relay.example.com"));
        eprintln!("relay_list_primary_first => {list:?}");
        assert_eq!(list[0], "wss://my-relay.example.com");
    }

    #[test]
    fn relay_list_no_duplicates() {
        let list = build_relay_list(Some("wss://relay.damus.io"));
        eprintln!("relay_list_no_duplicates => {list:?}");
        let count = list.iter().filter(|r| r.as_str() == "wss://relay.damus.io").count();
        assert_eq!(count, 1, "damus should appear once: {list:?}");
    }

    #[test]
    fn relay_list_without_primary_uses_defaults() {
        let list = build_relay_list(None);
        eprintln!("relay_list_without_primary_uses_defaults => {list:?}");
        assert!(!list.is_empty());
        assert_eq!(list[0], DEFAULT_RELAY_FALLBACKS[0]);
    }

    #[test]
    fn relay_list_normalises_primary() {
        let list = build_relay_list(Some("relay.damus.io"));
        eprintln!("relay_list_normalises_primary: relay.damus.io => {list:?}");
        assert_eq!(list[0], "wss://relay.damus.io");
    }

    // ── npub_to_hex ───────────────────────────────────────────────────────

    #[test]
    fn npub_bech32_roundtrip() {
        let npub = "npub1ahaz04ya9tehace3uy39hdhdryfvdkve9qdndkqp3tvehs6h8s5slq45hy";
        let hex = npub_to_hex(npub).expect("valid npub");
        eprintln!("npub_bech32_roundtrip: {npub} => {hex}");
        assert_eq!(hex.len(), 64);
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn npub_hex_passthrough() {
        let hex = "ee".repeat(32);
        let out = npub_to_hex(&hex).unwrap();
        eprintln!("npub_hex_passthrough: {hex} => {out}");
        assert_eq!(out, hex);
    }

    #[test]
    fn npub_hex_uppercased_normalised_to_lowercase() {
        let upper = "EE".repeat(32);
        let out = npub_to_hex(&upper).unwrap();
        eprintln!("npub_hex_uppercased_normalised_to_lowercase: {upper} => {out}");
        assert_eq!(out, "ee".repeat(32));
    }

    #[test]
    fn npub_real_pubkeys_from_events() {
        for hex in [
            "5c1eeccff05aa3ff47bc56fa80bc5c254a8eb67c3a8be2d29bf9b142aa57a7da",
            "b1576eb99a4774158a32fc5e190afa3ded4da19f51fbfa0b1a1bf6421ea5733a",
            "7459d333af66066f066cf87796e690db3a96ff4534f9edf4eab74df2f207289b",
            "34aff4e955e5d9a609434f5768ea8c089ee2afaa36d6e2be210c6813049a295f",
            "d04ecf33a303a59852fdb681ed8b412201ba85d8d2199aec73cb62681d62aa90",
        ] {
            let out = npub_to_hex(hex).unwrap_or_else(|_| panic!("failed for {hex}"));
            eprintln!("npub_real_pubkeys_from_events: {hex} => {out}");
            assert_eq!(out, hex, "hex pubkey should pass through unchanged");
        }
    }

    #[test]
    fn npub_invalid_rejected() {
        for bad in ["not-a-key", "npub1bad!", ""] {
            let result = npub_to_hex(bad);
            eprintln!("npub_invalid_rejected: {bad:?} => {result:?}");
            assert!(result.is_err());
        }
    }

    // ── New patterns from `gnostr query --kinds 30617 --limit 100` ────────

    // Pattern: SSH-only clone URL — must skip git@ URLs, fall back to web tag
    fn event_ssh_only_clone() -> Value {
        serde_json::json!({
            "kind": 30617,
            "tags": [
                ["d", "test-nip34-repo"],
                ["name", "test-nip34-repo"],
                ["clone", "git@example.com:test/test-nip34-repo.git"]
            ]
        })
    }

    // Pattern: SSH clone + web tag fallback
    fn event_ssh_clone_with_web() -> Value {
        serde_json::json!({
            "kind": 30617,
            "tags": [
                ["d", "gitmark"],
                ["name", "gitmark"],
                ["clone", "git@github.com:solidpayorg/gitmark.git"],
                ["web", "https://github.com/solidpayorg/gitmark"]
            ]
        })
    }

    // Pattern: github.com clone URL (not a GRASP server but valid HTTPS)
    fn event_github_clone() -> Value {
        serde_json::json!({
            "kind": 30617,
            "tags": [
                ["d", "get_file_hash"],
                ["name", "get_file_hash"],
                ["clone", "https://github.com/gnostr-org/get_file_hash"],
                ["web", "https://github.com/gnostr-org/get_file_hash"]
            ]
        })
    }

    // Pattern: GRASP first then GitHub second — GRASP wins
    fn event_grasp_then_github() -> Value {
        serde_json::json!({
            "kind": 30617,
            "tags": [
                ["d", "societybuilder"],
                ["name", "societybuilder"],
                ["clone",
                    "https://pyramid.fiatjaf.com/npub1elta7cneng3w8p9y4dw633qzdjr4kyvaparuyuttyrx6e8xp7xnq32cume/societybuilder.git",
                    "https://github.com/lez/societybuilder"
                ],
                ["web", "https://pyramid.fiatjaf.com/npub1elta7cneng3w8p9y4dw633qzdjr4kyvaparuyuttyrx6e8xp7xnq32cume/societybuilder"],
                ["relays", "wss://relay.hunos.hu", "wss://nostr.huszonegy.world", "wss://relay.primal.net",
                    "wss://relay.damus.io", "wss://nos.lol", "wss://pyramid.fiatjaf.com", "wss://relay.nostr.hu"]
            ]
        })
    }

    // Pattern: GitHub first then GRASP — GitHub wins (first HTTPS URL)
    fn event_github_then_grasp() -> Value {
        serde_json::json!({
            "kind": 30617,
            "tags": [
                ["d", "societybuilder2"],
                ["name", "societybuilder2"],
                ["clone",
                    "https://github.com/lez/societybuilder2.git",
                    "https://pyramid.fiatjaf.com/npub1elta7cneng3w8p9y4dw633qzdjr4kyvaparuyuttyrx6e8xp7xnq32cume/societybuilder2.git"
                ]
            ]
        })
    }

    // Pattern: THREE clone URLs (gitlab + github + grasp)
    fn event_three_clone_urls() -> Value {
        serde_json::json!({
            "kind": 30617,
            "tags": [
                ["d", "budabit-landing"],
                ["name", "budabit-landing"],
                ["clone",
                    "https://gitlab.com/Pleb5/budabit-landing.git",
                    "https://github.com/Pleb5/budabit-landing.git",
                    "https://grasp.budabit.club/npub16p8v7varqwjes5hak6q7mz6pygqm4pwc6gve4mrned3xs8tz42gq7kfhdw/budabit-landing.git"
                ],
                ["web", "https://gitlab.com/Pleb5/budabit-landing"],
                ["relays", "wss://relay.damus.io", "wss://grasp.budabit.club"]
            ]
        })
    }

    // Pattern: THREE clone URLs (codeberg + github + gitnostr)
    fn event_oba() -> Value {
        serde_json::json!({
            "kind": 30617,
            "tags": [
                ["d", "oba"],
                ["name", "Open Bitcoin Academy"],
                ["clone",
                    "https://codeberg.org/OpenBitcoinAcademy/oba.git",
                    "https://github.com/OpenBitcoinAcademy/oba.git",
                    "https://gitnostr.com/npub1qfcefq43er545mwvg69093almdkqs4wmcxaxjg9ad0rzq834qqfq9rhflh/oba.git"
                ],
                ["web", "https://codeberg.org/OpenBitcoinAcademy/oba"],
                ["relays", "wss://gitnostr.com", "wss://relay.damus.io", "wss://nos.lol", "wss://relay.nostr.band"]
            ]
        })
    }

    // Pattern: relay URL with path segment (wss://ditto.pub/relay)
    fn event_with_relay_paths() -> Value {
        serde_json::json!({
            "kind": 30617,
            "tags": [
                ["d", "nip89-app-directory"],
                ["name", "Nip89 App Directory"],
                ["clone",
                    "https://relay.ngit.dev/npub14rg4vrt2v374q95ezeeydu3hkdhmzglcj950mggacap4x0lv0gyq04wun7/nip89-app-directory.git",
                    "https://git.shakespeare.diy/npub14rg4vrt2v374q95ezeeydu3hkdhmzglcj950mggacap4x0lv0gyq04wun7/nip89-app-directory.git"
                ],
                ["relays",
                    "wss://relay.ngit.dev/", "wss://git.shakespeare.diy/",
                    "wss://relay.ditto.pub/", "wss://relay.damus.io/", "wss://nos.lol/",
                    "wss://nostr.wine/", "wss://relay.primal.net/", "wss://nostr.oxtr.dev/",
                    "wss://offchain.pub/", "wss://relay.mostr.pub/",
                    "wss://ditto.pub/relay", "wss://search.nos.today/",
                    "wss://relay.coinos.io/", "wss://drops.basspistol.org/",
                    "wss://relay.noswhere.com/", "wss://gleasonator.dev/relay"
                ]
            ]
        })
    }

    // Pattern: 9 relays all with trailing slashes (bchstr24)
    fn event_nine_relays() -> Value {
        serde_json::json!({
            "kind": 30617,
            "tags": [
                ["d", "bchstr24"],
                ["name", "Bchstr24"],
                ["clone", "https://relay.ngit.dev/npub1madu4w57wnxpwexfwuawzcpfnh094nmeg9hze9n43kazyhn8qlxq4lrgfg/bchstr24.git"],
                ["relays",
                    "wss://relay.ngit.dev/", "wss://relay.damus.io/", "wss://nostr.land/",
                    "wss://nostr.wine/", "wss://nos.lol/", "wss://nostr.mom/",
                    "wss://cache1.primal.net/", "wss://relay.snort.social/", "wss://relay.nostr.pub/"
                ],
                ["web", "https://bchstr24.shakespeare.wtf"]
            ]
        })
    }

    // Pattern: no clone/web/relays (deleted repo, just d/name)
    fn event_deleted_repo() -> Value {
        serde_json::json!({
            "kind": 30617,
            "tags": [
                ["d", "test-asdf"],
                ["name", "test-asdf"],
                ["deleted", "true"]
            ]
        })
    }

    // Pattern: bare refs tags only (no clone/web)
    fn event_refs_only() -> Value {
        serde_json::json!({
            "kind": 30617,
            "tags": [
                ["d", "7"],
                ["name", "7"],
                ["HEAD", "ref: refs/heads/gh-pages"],
                ["refs/heads/gh-pages", "74d90d2a0a86d1424cfdb44e97a25ad54e7c3040"]
            ]
        })
    }

    // ── extract_web_url: new pattern tests ────────────────────────────────

    #[test]
    fn ssh_only_clone_returns_none() {
        // git@ URLs are not HTTP(S) — should be skipped, no web fallback → None
        let result = extract_web_url(&event_ssh_only_clone());
        eprintln!("ssh_only_clone_returns_none => {result:?}");
        assert!(result.is_none(), "SSH-only clone should yield None, got: {result:?}");
    }

    #[test]
    fn ssh_clone_falls_back_to_web() {
        // git@ clone skipped, web tag present → web wins
        let url = extract_web_url(&event_ssh_clone_with_web()).unwrap();
        eprintln!("ssh_clone_falls_back_to_web => {url}");
        assert_eq!(url, "https://github.com/solidpayorg/gitmark");
    }

    #[test]
    fn github_clone_url_returned() {
        // github.com is a valid HTTPS clone URL — returned as-is
        let url = extract_web_url(&event_github_clone()).unwrap();
        eprintln!("github_clone_url_returned => {url}");
        assert_eq!(url, "https://github.com/gnostr-org/get_file_hash");
    }

    #[test]
    fn grasp_first_then_github_returns_grasp() {
        let url = extract_web_url(&event_grasp_then_github()).unwrap();
        eprintln!("grasp_first_then_github_returns_grasp => {url}");
        assert!(url.starts_with("https://pyramid.fiatjaf.com/"), "expected pyramid GRASP, got: {url}");
    }

    #[test]
    fn github_first_then_grasp_returns_github() {
        // First HTTPS wins regardless of server type
        let url = extract_web_url(&event_github_then_grasp()).unwrap();
        eprintln!("github_first_then_grasp_returns_github => {url}");
        assert!(url.starts_with("https://github.com/"), "expected github first, got: {url}");
    }

    #[test]
    fn three_clone_urls_first_wins() {
        // budabit-landing: gitlab / github / grasp — gitlab is first
        let url = extract_web_url(&event_three_clone_urls()).unwrap();
        eprintln!("three_clone_urls_first_wins => {url}");
        assert!(url.starts_with("https://gitlab.com/"), "expected gitlab first, got: {url}");
    }

    #[test]
    fn oba_three_clone_urls_first_wins() {
        // codeberg / github / gitnostr — codeberg is first
        let url = extract_web_url(&event_oba()).unwrap();
        eprintln!("oba_three_clone_urls_first_wins => {url}");
        assert!(url.starts_with("https://codeberg.org/"), "expected codeberg first, got: {url}");
    }

    #[test]
    fn deleted_repo_returns_none() {
        let result = extract_web_url(&event_deleted_repo());
        eprintln!("deleted_repo_returns_none => {result:?}");
        assert!(result.is_none());
    }

    #[test]
    fn refs_only_event_returns_none() {
        let result = extract_web_url(&event_refs_only());
        eprintln!("refs_only_event_returns_none => {result:?}");
        assert!(result.is_none());
    }

    // ── extract_event_relays: new pattern tests ───────────────────────────

    #[test]
    fn nine_relays_all_trailing_slashes_stripped() {
        let relays = extract_event_relays(&event_nine_relays());
        eprintln!("nine_relays_all_trailing_slashes_stripped => {relays:?}");
        assert_eq!(relays.len(), 9);
        for r in &relays {
            assert!(!r.ends_with('/'), "trailing slash not stripped: {r}");
        }
        assert!(relays.contains(&"wss://relay.ngit.dev".to_string()));
        assert!(relays.contains(&"wss://cache1.primal.net".to_string()));
    }

    #[test]
    fn relay_with_path_preserved() {
        // wss://ditto.pub/relay and wss://gleasonator.dev/relay have path segments
        // — the path must not be stripped (only trailing slash on host-only URLs)
        let relays = extract_event_relays(&event_with_relay_paths());
        eprintln!("relay_with_path_preserved => {relays:?}");
        assert_eq!(relays.len(), 16);
        // Path-carrying relays must survive intact
        assert!(
            relays.contains(&"wss://ditto.pub/relay".to_string()),
            "ditto.pub/relay missing: {relays:?}"
        );
        assert!(
            relays.contains(&"wss://gleasonator.dev/relay".to_string()),
            "gleasonator.dev/relay missing: {relays:?}"
        );
        // Host-only trailing slashes must be gone
        assert!(
            relays.contains(&"wss://relay.ngit.dev".to_string()),
            "ngit.dev missing: {relays:?}"
        );
    }

    #[test]
    fn sixteen_relays_count_correct() {
        let relays = extract_event_relays(&event_with_relay_paths());
        eprintln!("sixteen_relays_count_correct => {} relays", relays.len());
        assert_eq!(relays.len(), 16);
    }

    #[test]
    fn grasp_then_github_relay_list() {
        let relays = extract_event_relays(&event_grasp_then_github());
        eprintln!("grasp_then_github_relay_list => {relays:?}");
        assert_eq!(relays.len(), 7);
        assert!(relays.contains(&"wss://pyramid.fiatjaf.com".to_string()));
    }

    // ── normalize_relay_url: relay-with-path ──────────────────────────────

    #[test]
    fn normalize_relay_url_with_path_preserved() {
        // wss://ditto.pub/relay — path component must be kept
        let out = normalize_relay_url("wss://ditto.pub/relay");
        eprintln!("normalize_relay_url_with_path_preserved: wss://ditto.pub/relay => {out}");
        assert_eq!(out, "wss://ditto.pub/relay");
    }

    #[test]
    fn normalize_relay_gleasonator_path() {
        let out = normalize_relay_url("wss://gleasonator.dev/relay");
        eprintln!("normalize_relay_gleasonator_path: wss://gleasonator.dev/relay => {out}");
        assert_eq!(out, "wss://gleasonator.dev/relay");
    }

    // ── New fixtures from gnostr query --kinds 30617,30618,1617,1618,1619,1621,1630,1631 ──

    /// kind:30618 — vidstr repo: HEAD + single branch ref
    fn event_30618_vidstr() -> Value {
        serde_json::json!({
            "kind": 30618,
            "pubkey": "a8d1560d6a647d501699167246f237b36fb123f89168fda11dc743533fec7a08",
            "tags": [
                ["d", "vidstr"],
                ["HEAD", "ref: refs/heads/main"],
                ["refs/heads/main", "4f6724601d66ea5b5502bf9de8e3ee4dbce03b45"]
            ]
        })
    }

    /// kind:30618 — amethyst: many tags, a branch + HEAD (no clean HEAD tag in raw data, only refs/tags)
    fn event_30618_amethyst() -> Value {
        serde_json::json!({
            "kind": 30618,
            "pubkey": "460c25e682fda7832b52d1f22d3d22b3176d972f60dcdc3212ed8c92ef85065c",
            "tags": [
                ["d", "amethyst"],
                ["refs/tags/v0.11.0", "eff961699c544f6ee33ef66b27cda33cc5e5afa3"],
                ["refs/tags/v1.04.0", "503ebaeb725e705897da18b78b0f8494baec1e28"],
                ["refs/tags/v0.88.6", "86d948bd95acd275f69ac5ef43cb579192f9a1ca"],
                ["refs/heads/main", "4d8340a9d76a5b374617a2357ad5b89af6e7cd3d"],
                ["refs/heads/recommendation-engine", "5753472a44f3908fef724820fb47ab8408a4eaf1"],
                ["refs/heads/local-database", "7871236ce7c92f28d268010f7b0f920eb306c4ac"],
                ["refs/heads/full-outbox", "340760513f278bef03b1eeb8c1fb3487dfca11a4"]
            ]
        })
    }

    /// kind:30618 — empty refs (repo stopped tracking state)
    fn event_30618_empty_refs() -> Value {
        serde_json::json!({
            "kind": 30618,
            "pubkey": "deadbeef00000000000000000000000000000000000000000000000000000001",
            "tags": [
                ["d", "my-repo"]
            ]
        })
    }

    /// kind:1617 — real patch event (nprogram repo)
    fn event_1617_patch() -> Value {
        serde_json::json!({
            "kind": 1617,
            "pubkey": "00000001505e7e48927046e9bbaa728b1f3b511227e2200c578d6e6bb0c77eb9",
            "tags": [
                ["alt", "git patch: [PATCH] examples(resonate): Read the event ID as i32_le"],
                ["description", "[PATCH] examples(resonate): Read the event ID as i32_le"],
                ["a", "30617:3bf0c63fcb93463407af97a5e5ee64fa883d107ef9e558472c4eb9aaaefa459d:nprogram"],
                ["p", "3bf0c63fcb93463407af97a5e5ee64fa883d107ef9e558472c4eb9aaaefa459d"],
                ["r", "fb30ddf36678dd287b47fe979838672fa4a740f3"],
                ["t", "root"]
            ],
            "content": "From cfcc04e6 Mon Sep 17 00:00:00 2001\nSubject: [PATCH] examples(resonate): Read the event ID as i32_le\n"
        })
    }

    /// kind:1618 — real PR event (satshoot: Cashu payments)
    fn event_1618_pr() -> Value {
        serde_json::json!({
            "kind": 1618,
            "pubkey": "71df211931d26ee41121d295bd43cbc7e382505e333b5c13d4016ced9542d9d7",
            "tags": [
                ["a", "30617:d04ecf33a303a59852fdb681ed8b412201ba85d8d2199aec73cb62681d62aa90:satshoot"],
                ["subject", "Cashu Payments with Locktime and Refund Public Key"],
                ["t", "enhancement"],
                ["c", "c318b8577faef1bc57671b61524af476e86ac800"],
                ["clone", "https://github.com/rodant/satshoot.git"],
                ["branch-name", "testing"],
                ["merge-base", "78db010d6efcb936358461f0ea393eb63c5f9414"],
                ["p", "d04ecf33a303a59852fdb681ed8b412201ba85d8d2199aec73cb62681d62aa90"]
            ],
            "content": "Cashu payments in Nostr (Nutzaps) are perpetually locked..."
        })
    }

    /// kind:1621 — issue with subject + labels (oba repo)
    fn event_1621_issue() -> Value {
        serde_json::json!({
            "kind": 1621,
            "pubkey": "15a747c71d64ebe7842ce2d9b6d26b83d8c53a49164dc3dc7dbfafd4b17d2fb0",
            "tags": [
                ["subject", "Hi nsec1jwa3tt2jcqy9aqz9f8qqny9jjt0qs9r5v7lvnncqp265e8k5ahkq547yh8"],
                ["alt", "git repository issue: Hi ..."],
                ["a", "30617:02719482b1c8e95a6dcc468af2c7bfdb6c0855dbc1ba6920bd6bc6201e350012:oba"],
                ["p", "02719482b1c8e95a6dcc468af2c7bfdb6c0855dbc1ba6920bd6bc6201e350012"],
                ["t", "bug"],
                ["t", "security"]
            ],
            "content": "*NSEC_DELETED*"
        })
    }

    /// kind:1630 — Open status linked to a PR
    fn event_1630_open() -> Value {
        serde_json::json!({
            "kind": 1630,
            "pubkey": "71df211931d26ee41121d295bd43cbc7e382505e333b5c13d4016ced9542d9d7",
            "tags": [
                ["e", "323f1cf29cf7119357e63a4f352af197ac21acbce936526a55f4118cb607cfd6", "", "root"],
                ["p", "d04ecf33a303a59852fdb681ed8b412201ba85d8d2199aec73cb62681d62aa90"],
                ["p", "71df211931d26ee41121d295bd43cbc7e382505e333b5c13d4016ced9542d9d7"],
                ["a", "30617:d04ecf33a303a59852fdb681ed8b412201ba85d8d2199aec73cb62681d62aa90:satshoot"],
                ["r", "wss://relay.primal.net/"]
            ],
            "content": ""
        })
    }

    /// kind:1631 — Applied/Merged status with merge-commit and applied-as-commits
    fn event_1631_applied() -> Value {
        serde_json::json!({
            "kind": 1631,
            "pubkey": "d04ecf33a303a59852fdb681ed8b412201ba85d8d2199aec73cb62681d62aa90",
            "tags": [
                ["e", "323f1cf29cf7119357e63a4f352af197ac21acbce936526a55f4118cb607cfd6", "", "root"],
                ["p", "d04ecf33a303a59852fdb681ed8b412201ba85d8d2199aec73cb62681d62aa90"],
                ["p", "71df211931d26ee41121d295bd43cbc7e382505e333b5c13d4016ced9542d9d7"],
                ["a", "npub16p8v7varqwjes5hak6q7mz6pygqm4pwc6gve4mrned3xs8tz42gq7kfhdw/satshoot"],
                ["r", "wss://relay.primal.net"],
                ["merge-commit", "c318b8577faef1bc57671b61524af476e86ac800"],
                ["applied-as-commits",
                    "c318b8577faef1bc57671b61524af476e86ac800",
                    "e30a0c2fe8a7093f4513bf45e2e5e1f1ee521baa",
                    "25cfef28e773f9fd805bf97c33083fb76801fc40"]
            ],
            "content": "PR applied: Cashu Payments with Locktime and Refund Public Key"
        })
    }

    // ── extract_head_branch ────────────────────────────────────────────────

    #[test]
    fn head_branch_simple() {
        let branch = extract_head_branch(&event_30618_vidstr());
        eprintln!("head_branch_simple => {branch:?}");
        assert_eq!(branch, Some("main".to_string()));
    }

    #[test]
    fn head_branch_missing_returns_none() {
        // amethyst fixture has no HEAD tag
        let branch = extract_head_branch(&event_30618_amethyst());
        eprintln!("head_branch_missing_returns_none => {branch:?}");
        assert_eq!(branch, None);
    }

    #[test]
    fn head_branch_on_empty_refs_event() {
        let branch = extract_head_branch(&event_30618_empty_refs());
        eprintln!("head_branch_on_empty_refs_event => {branch:?}");
        assert_eq!(branch, None);
    }

    // ── extract_refs ──────────────────────────────────────────────────────

    #[test]
    fn refs_single_branch() {
        let refs = extract_refs(&event_30618_vidstr());
        eprintln!("refs_single_branch => {refs:?}");
        assert_eq!(refs.len(), 1);
        assert_eq!(
            refs.get("refs/heads/main").map(|s| s.as_str()),
            Some("4f6724601d66ea5b5502bf9de8e3ee4dbce03b45")
        );
    }

    #[test]
    fn refs_amethyst_multiple_branches_and_tags() {
        let refs = extract_refs(&event_30618_amethyst());
        eprintln!("refs_amethyst_multiple => {} refs", refs.len());
        // 3 tags + 4 branches = 7
        assert_eq!(refs.len(), 7);
        assert!(refs.contains_key("refs/heads/main"));
        assert!(refs.contains_key("refs/heads/recommendation-engine"));
        assert!(refs.contains_key("refs/heads/full-outbox"));
        assert!(refs.contains_key("refs/tags/v0.11.0"));
    }

    #[test]
    fn refs_empty_event_returns_empty_map() {
        let refs = extract_refs(&event_30618_empty_refs());
        eprintln!("refs_empty_event_returns_empty_map => {refs:?}");
        assert!(refs.is_empty());
    }

    // ── extract_repo_addr ─────────────────────────────────────────────────

    #[test]
    fn repo_addr_standard_format() {
        let addr = extract_repo_addr(&event_1617_patch());
        eprintln!("repo_addr_standard_format => {addr:?}");
        assert_eq!(
            addr,
            Some((
                "3bf0c63fcb93463407af97a5e5ee64fa883d107ef9e558472c4eb9aaaefa459d".to_string(),
                "nprogram".to_string()
            ))
        );
    }

    #[test]
    fn repo_addr_from_pr() {
        let addr = extract_repo_addr(&event_1618_pr());
        eprintln!("repo_addr_from_pr => {addr:?}");
        assert_eq!(
            addr,
            Some((
                "d04ecf33a303a59852fdb681ed8b412201ba85d8d2199aec73cb62681d62aa90".to_string(),
                "satshoot".to_string()
            ))
        );
    }

    #[test]
    fn repo_addr_from_issue() {
        let addr = extract_repo_addr(&event_1621_issue());
        eprintln!("repo_addr_from_issue => {addr:?}");
        assert_eq!(
            addr,
            Some((
                "02719482b1c8e95a6dcc468af2c7bfdb6c0855dbc1ba6920bd6bc6201e350012".to_string(),
                "oba".to_string()
            ))
        );
    }

    #[test]
    fn repo_addr_npub_slash_format_in_status() {
        // kind:1631 uses "npub.../satshoot" format in the a tag
        let addr = extract_repo_addr(&event_1631_applied());
        eprintln!("repo_addr_npub_slash_format => {addr:?}");
        let (pubkey, repo) = addr.expect("should parse npub/repo format");
        assert_eq!(repo, "satshoot");
        assert_eq!(pubkey.len(), 64);
        assert!(pubkey.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn repo_addr_missing_returns_none() {
        // 30618 events have no a tag
        let addr = extract_repo_addr(&event_30618_vidstr());
        eprintln!("repo_addr_missing_returns_none => {addr:?}");
        assert_eq!(addr, None);
    }

    // ── extract_subject ───────────────────────────────────────────────────

    #[test]
    fn subject_from_pr() {
        let subj = extract_subject(&event_1618_pr());
        eprintln!("subject_from_pr => {subj:?}");
        assert_eq!(
            subj,
            Some("Cashu Payments with Locktime and Refund Public Key".to_string())
        );
    }

    #[test]
    fn subject_from_issue() {
        let subj = extract_subject(&event_1621_issue());
        eprintln!("subject_from_issue => {subj:?}");
        assert!(subj.unwrap().starts_with("Hi nsec1"));
    }

    #[test]
    fn subject_missing_returns_none() {
        let subj = extract_subject(&event_1617_patch());
        eprintln!("subject_missing_returns_none => {subj:?}");
        assert_eq!(subj, None);
    }

    // ── extract_tip_commit ────────────────────────────────────────────────

    #[test]
    fn tip_commit_from_pr_c_tag() {
        let sha = extract_tip_commit(&event_1618_pr());
        eprintln!("tip_commit_from_pr_c_tag => {sha:?}");
        assert_eq!(sha, Some("c318b8577faef1bc57671b61524af476e86ac800".to_string()));
    }

    #[test]
    fn tip_commit_from_applied_merge_commit() {
        let sha = extract_tip_commit(&event_1631_applied());
        eprintln!("tip_commit_from_applied_merge_commit => {sha:?}");
        assert_eq!(sha, Some("c318b8577faef1bc57671b61524af476e86ac800".to_string()));
    }

    #[test]
    fn tip_commit_from_patch_r_tag() {
        let sha = extract_tip_commit(&event_1617_patch());
        eprintln!("tip_commit_from_patch_r_tag => {sha:?}");
        assert_eq!(sha, Some("fb30ddf36678dd287b47fe979838672fa4a740f3".to_string()));
    }

    #[test]
    fn tip_commit_missing_returns_none() {
        // Status open event has no commit tag
        let sha = extract_tip_commit(&event_1630_open());
        eprintln!("tip_commit_missing_returns_none => {sha:?}");
        assert_eq!(sha, None);
    }

    // ── status_kind_str ───────────────────────────────────────────────────

    #[test]
    fn status_kind_str_all_values() {
        for (k, expected) in [(1630u64, "open"), (1631, "applied"), (1632, "closed"), (1633, "draft"), (9999, "unknown")] {
            let s = status_kind_str(k);
            eprintln!("status_kind_str({k}) => {s}");
            assert_eq!(s, expected);
        }
    }

    // ── extract_labels ────────────────────────────────────────────────────

    #[test]
    fn labels_from_issue() {
        let labels = extract_labels(&event_1621_issue());
        eprintln!("labels_from_issue => {labels:?}");
        assert_eq!(labels, vec!["bug", "security"]);
    }

    #[test]
    fn labels_from_pr_single() {
        let labels = extract_labels(&event_1618_pr());
        eprintln!("labels_from_pr_single => {labels:?}");
        assert_eq!(labels, vec!["enhancement"]);
    }

    #[test]
    fn labels_missing_returns_empty() {
        let labels = extract_labels(&event_1617_patch());
        eprintln!("labels_missing_returns_empty (patch has t:root) => {labels:?}");
        // patch has t:root which is a valid label too
        assert!(labels.contains(&"root".to_string()));
    }
}
