//! Build and sign arbitrary Nostr events (NIP-01).
//!
//! Uses `secp256k1` (from the blossom-rs workspace dep) and the
//! `compute_event_id` helper already in `blossom_rs::protocol`.
//!
//! The signed event is returned as a `serde_json::Value` ready to be
//! published via [`crate::nostr_relay::RelayCmd::Publish`].

use serde_json::{Value, json};

// ── Public API ────────────────────────────────────────────────────────────────

/// Build and sign a Nostr event.
///
/// * `hex_seckey` — 64-char hex private key
/// * `kind`       — event kind (0, 1063, 10002, …)
/// * `tags`       — list of tag arrays, e.g. `vec![vec!["r".into(), url]]`
/// * `content`    — UTF-8 content string
///
/// Returns the completed event as a JSON object with all required fields.
pub fn sign_event(
    hex_seckey: &str,
    kind: u32,
    tags: Vec<Vec<String>>,
    content: &str,
) -> Result<Value, SignError> {
    use secp256k1::{Keypair, Message, Secp256k1, SecretKey};

    let sk_bytes = hex::decode(hex_seckey).map_err(|_| SignError::InvalidKey)?;
    let sk = SecretKey::from_slice(&sk_bytes).map_err(|_| SignError::InvalidKey)?;
    let secp = Secp256k1::new();
    let kp   = Keypair::from_secret_key(&secp, &sk);
    let (xonly, _) = kp.x_only_public_key();
    let pubkey_hex = hex::encode(xonly.serialize());

    let created_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Serialise tags for the JSON representation.
    let tags_value: Vec<Vec<Value>> = tags
        .iter()
        .map(|row| row.iter().map(|s| Value::String(s.clone())).collect())
        .collect();

    let event_id_bytes = blossom_rs::protocol::compute_event_id(
        &pubkey_hex,
        created_at,
        kind,
        &tags,
        content,
    );
    let event_id = hex::encode(event_id_bytes);

    // Sign the id.
    let id_bytes = event_id_bytes;
    let msg = Message::from_digest_slice(&id_bytes).map_err(|_| SignError::Internal)?;
    let sig = secp.sign_schnorr_no_aux_rand(&msg, &kp);
    let sig_hex = hex::encode(sig.serialize());

    Ok(json!({
        "id":         event_id,
        "pubkey":     pubkey_hex,
        "created_at": created_at,
        "kind":       kind,
        "tags":       tags_value,
        "content":    content,
        "sig":        sig_hex,
    }))
}

// ── Kind-specific event builders ──────────────────────────────────────────────

/// Build and sign a kind:0 (User Metadata) event.
/// `meta` keys: name, about, picture, nip05, website, lud16 (all optional).
pub fn kind0_metadata(
    hex_seckey: &str,
    meta: &serde_json::Map<String, Value>,
) -> Result<Value, SignError> {
    let content = serde_json::to_string(meta).map_err(|_| SignError::Internal)?;
    sign_event(hex_seckey, 0, vec![], &content)
}

/// Build and sign a kind:1063 (File Metadata / NIP-94) event.
pub fn kind1063_file_metadata(
    hex_seckey: &str,
    url: &str,
    sha256: &str,
    mime: &str,
    size: Option<u64>,
    caption: &str,
) -> Result<Value, SignError> {
    let mut tags = vec![
        vec!["url".into(),  url.into()],
        vec!["x".into(),    sha256.into()],
        vec!["ox".into(),   sha256.into()],
        vec!["m".into(),    mime.into()],
    ];
    if let Some(sz) = size {
        tags.push(vec!["size".into(), sz.to_string()]);
    }
    sign_event(hex_seckey, 1063, tags, caption)
}

/// Build and sign a kind:10002 (Relay List Metadata / NIP-65) event.
/// `relays`: list of `(url, marker)` where marker is `"read"`, `"write"`,
/// or empty string for both.
pub fn kind10002_relay_list(
    hex_seckey: &str,
    relays: &[(String, String)],
) -> Result<Value, SignError> {
    let tags = relays
        .iter()
        .map(|(url, marker)| {
            if marker.is_empty() {
                vec!["r".into(), url.clone()]
            } else {
                vec!["r".into(), url.clone(), marker.clone()]
            }
        })
        .collect();
    sign_event(hex_seckey, 10002, tags, "")
}

/// Build and sign a kind:10063 (Blossom Server List / NIP-B7) event.
pub fn kind10063_server_list(
    hex_seckey: &str,
    servers: &[String],
) -> Result<Value, SignError> {
    let tags = servers
        .iter()
        .map(|url| vec!["server".into(), url.clone()])
        .collect();
    sign_event(hex_seckey, 10063, tags, "")
}

/// Build and sign a kind:30617 (Repository Announcement / NIP-34) event.
#[allow(clippy::too_many_arguments)]
pub fn kind30617_repo_announcement(
    hex_seckey: &str,
    repo_id: &str,
    name: &str,
    description: &str,
    clone_urls: &[String],
    web_urls: &[String],
    relays: &[String],
    earliest_commit: Option<&str>,
) -> Result<Value, SignError> {
    let mut tags = vec![
        vec!["d".into(), repo_id.into()],
        vec!["name".into(), name.into()],
        vec!["description".into(), description.into()],
    ];
    for url in clone_urls {
        tags.push(vec!["clone".into(), url.clone()]);
    }
    for url in web_urls {
        tags.push(vec!["web".into(), url.clone()]);
    }
    if !relays.is_empty() {
        let mut relay_tag = vec!["relays".into()];
        relay_tag.extend(relays.iter().cloned());
        tags.push(relay_tag);
    }
    if let Some(euc) = earliest_commit {
        tags.push(vec!["r".into(), euc.into(), "euc".into()]);
    }
    sign_event(hex_seckey, 30617, tags, "")
}

/// Build and sign a kind:1621 (Issue / NIP-34) event.
pub fn kind1621_issue(
    hex_seckey: &str,
    repo_addr: &str,   // "30617:<owner-pubkey>:<repo-id>"
    title: &str,
    body: &str,
    labels: &[String],
) -> Result<Value, SignError> {
    let mut tags = vec![
        vec!["a".into(), repo_addr.into()],
        vec!["subject".into(), title.into()],
    ];
    for label in labels {
        tags.push(vec!["t".into(), label.clone()]);
    }
    sign_event(hex_seckey, 1621, tags, body)
}

// ── Error ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignError {
    InvalidKey,
    Internal,
}

impl std::fmt::Display for SignError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidKey => write!(f, "invalid private key"),
            Self::Internal   => write!(f, "internal signing error"),
        }
    }
}

impl std::error::Error for SignError {}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use secp256k1::{Secp256k1, SecretKey};

    fn test_key() -> String {
        // deterministic test key
        let sk = SecretKey::from_slice(&[0x42u8; 32]).unwrap();
        hex::encode(sk.secret_bytes())
    }

    #[test]
    fn sign_event_fields_present() {
        let ev = sign_event(&test_key(), 1, vec![], "hello").unwrap();
        assert!(ev["id"].is_string());
        assert!(ev["pubkey"].is_string());
        assert!(ev["sig"].is_string());
        assert_eq!(ev["kind"], 1);
        assert_eq!(ev["content"], "hello");
    }

    #[test]
    fn kind0_produces_valid_json_content() {
        let mut meta = serde_json::Map::new();
        meta.insert("name".into(), serde_json::Value::String("Alice".into()));
        let ev = kind0_metadata(&test_key(), &meta).unwrap();
        assert_eq!(ev["kind"], 0);
        let content: serde_json::Value =
            serde_json::from_str(ev["content"].as_str().unwrap()).unwrap();
        assert_eq!(content["name"], "Alice");
    }

    #[test]
    fn kind1063_tags_present() {
        let ev = kind1063_file_metadata(
            &test_key(),
            "https://example.com/blob",
            "aabbcc",
            "image/png",
            Some(1024),
            "my photo",
        )
        .unwrap();
        assert_eq!(ev["kind"], 1063);
        let tags = ev["tags"].as_array().unwrap();
        let has_url = tags.iter().any(|t| t[0] == "url");
        let has_size = tags.iter().any(|t| t[0] == "size");
        assert!(has_url);
        assert!(has_size);
    }

    #[test]
    fn invalid_key_rejected() {
        assert_eq!(
            sign_event("not_hex", 0, vec![], "").unwrap_err(),
            SignError::InvalidKey
        );
    }

    #[test]
    fn pubkey_derivation_consistent() {
        let secp = Secp256k1::new();
        let sk = SecretKey::from_slice(&[0x42u8; 32]).unwrap();
        let kp = secp256k1::Keypair::from_secret_key(&secp, &sk);
        let (xonly, _) = kp.x_only_public_key();
        let expected = hex::encode(xonly.serialize());

        let ev = sign_event(&test_key(), 0, vec![], "").unwrap();
        assert_eq!(ev["pubkey"].as_str().unwrap(), expected);
    }
}
