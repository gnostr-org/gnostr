//! BUD-01 upload authorization — Nostr kind:24242 signed event.
//!
//! Builds the `Authorization: Nostr <base64>` header value required by
//! BUD-01 for authenticated uploads.

use anyhow::{bail, Context, Result};
use secp256k1::{Secp256k1, SecretKey};

/// Build a base64-encoded Nostr kind:24242 event for BUD-01 upload auth.
///
/// `nsec` may be a `nsec1…` bech32 string or a 64-char hex string.
/// `sha256` is the lowercase hex SHA-256 of the file being uploaded.
pub fn build_upload_auth(nsec: &str, sha256: &str, content_type: &str) -> Result<String> {
    let secret_bytes = decode_nsec(nsec).context("decode NOSTR_NSEC")?;
    let secp = Secp256k1::signing_only();
    let sk = SecretKey::from_slice(&secret_bytes).context("invalid secret key")?;
    let keypair = secp256k1::Keypair::from_secret_key(&secp, &sk);
    let pubkey = keypair.public_key().serialize_uncompressed();
    // x-only pubkey hex (bytes 1..33)
    let pubkey_hex = hex::encode(&pubkey[1..33]);

    let created_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // kind:24242 event for BUD-01
    let tags = serde_json::json!([
        ["t", "upload"],
        ["x", sha256],
        ["expiration", (created_at + 300).to_string()],
    ]);

    let content = format!("Upload {content_type}");

    // Build serialized event for ID computation
    let serialized = serde_json::json!([
        0,
        pubkey_hex,
        created_at,
        24242,
        tags,
        content
    ])
    .to_string();

    let id = {
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(serialized.as_bytes());
        hex::encode(h.finalize())
    };

    // Sign with BIP-340 Schnorr
    let msg = secp256k1::Message::from_digest_slice(&hex::decode(&id).context("decode event id")?)
        .context("build signing message")?;

    let sig = secp.sign_schnorr_no_aux_rand(&msg, &keypair);
    let sig_hex = hex::encode(sig.serialize());

    let event = serde_json::json!({
        "id": id,
        "pubkey": pubkey_hex,
        "created_at": created_at,
        "kind": 24242,
        "tags": tags,
        "content": content,
        "sig": sig_hex,
    });

    let json = serde_json::to_string(&event)?;
    Ok(base64_encode(json.as_bytes()))
}

/// Build a base64-encoded Nostr kind:27235 event for NIP-98 HTTP auth (push).
///
/// Used for GRASP server push authentication.
pub fn build_push_auth(nsec: &str, url: &str) -> Result<String> {
    let secret_bytes = decode_nsec(nsec).context("decode NOSTR_NSEC")?;
    let secp = Secp256k1::signing_only();
    let sk = SecretKey::from_slice(&secret_bytes).context("invalid secret key")?;
    let keypair = secp256k1::Keypair::from_secret_key(&secp, &sk);
    let pubkey = keypair.public_key().serialize_uncompressed();
    let pubkey_hex = hex::encode(&pubkey[1..33]);

    let created_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let tags = serde_json::json!([
        ["u", url],
        ["method", "POST"],
    ]);

    let serialized = serde_json::json!([0, pubkey_hex, created_at, 27235, tags, ""])
        .to_string();

    let id = {
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(serialized.as_bytes());
        hex::encode(h.finalize())
    };

    let msg =
        secp256k1::Message::from_digest_slice(&hex::decode(&id).context("decode event id")?)
            .context("build signing message")?;
    let sig = secp.sign_schnorr_no_aux_rand(&msg, &keypair);

    let event = serde_json::json!({
        "id": id,
        "pubkey": pubkey_hex,
        "created_at": created_at,
        "kind": 27235,
        "tags": tags,
        "content": "",
        "sig": hex::encode(sig.serialize()),
    });

    Ok(base64_encode(serde_json::to_string(&event)?.as_bytes()))
}


/// Decode a Nostr secret key from either `nsec1…` bech32 or 64-char hex.
pub fn decode_nsec(nsec: &str) -> Result<[u8; 32]> {
    if nsec.starts_with("nsec1") {
        let (hrp, data) = bech32::decode(nsec).context("bech32 decode")?;
        if hrp.as_str() != "nsec" {
            bail!("expected hrp 'nsec', got '{}'", hrp.as_str());
        }
        let bytes: Vec<u8> = data;
        let arr: [u8; 32] = bytes.try_into().map_err(|_| anyhow::anyhow!("nsec must be 32 bytes"))?;
        Ok(arr)
    } else {
        let bytes = hex::decode(nsec).context("hex decode nsec")?;
        let arr: [u8; 32] = bytes
            .try_into()
            .map_err(|_| anyhow::anyhow!("hex nsec must be 32 bytes (64 chars)"))?;
        Ok(arr)
    }
}

// ── base64 (no external dep) ──────────────────────────────────────────────

fn base64_encode(input: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((input.len() + 2) / 3 * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = if chunk.len() > 1 { chunk[1] as usize } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as usize } else { 0 };
        out.push(ALPHABET[b0 >> 2] as char);
        out.push(ALPHABET[((b0 & 3) << 4) | (b1 >> 4)] as char);
        out.push(if chunk.len() > 1 { ALPHABET[((b1 & 0xf) << 2) | (b2 >> 6)] as char } else { '=' });
        out.push(if chunk.len() > 2 { ALPHABET[b2 & 0x3f] as char } else { '=' });
    }
    out
}
