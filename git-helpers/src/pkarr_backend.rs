//! PKARR DHT discovery backend for `git-remote-pkarr`.
//!
//! Resolves a PKARR Ed25519 public key from the Mainline DHT (or relay), reads
//! the `_blossom` TXT record to find the Blossom server URL, then delegates
//! all git operations to [`BlossomRemote`].
//!
//! ## URL format
//! ```text
//! pkarr://<zbase32-pubkey>/<repo>
//! ```
//!
//! ## Environment variables
//! | Variable        | Description                                              |
//! |-----------------|----------------------------------------------------------|
//! | `NOSTR_NSEC`    | nsec1… or hex secret key — used to sign uploads and to  |
//! |                 | derive the Nostr pubkey for BUD-02 blob listing          |
//! | `NOSTR_PUBKEY`  | Hex Nostr pubkey override (skip derivation from nsec)   |
//!
//! ## PKARR TXT records (published by blossom-rs `pkarr-discovery` feature)
//! | Name              | Value                          |
//! |-------------------|--------------------------------|
//! | `_blossom`        | Blossom HTTP server URL        |
//! | `_nostr`          | Nostr relay WebSocket URL      |

use anyhow::{Context, Result, bail};
use pkarr::dns::rdata::RData;
use pkarr::Client;

use crate::auth::decode_nsec;
use crate::blossom_backend::BlossomRemote;
use crate::protocol::{FetchCmd, GitRef, PushResult, PushSpec, RemoteHelper};

// ── PKARR resolver ────────────────────────────────────────────────────────

/// Resolve `_blossom` TXT record for a PKARR public key via DHT/relay.
///
/// Returns the blossom server HTTP URL if found.
pub async fn resolve_blossom_url(public_key: &pkarr::PublicKey) -> Result<String> {
    let client = Client::builder()
        .build()
        .map_err(|e| anyhow::anyhow!("pkarr client: {e}"))?;

    let packet = client
        .resolve(public_key)
        .await
        .with_context(|| format!("resolve pkarr key {}", public_key))?;

    for record in packet.resource_records("_blossom") {
        if let RData::TXT(txt) = &record.rdata {
            if let Ok(s) = String::try_from(txt.clone()) {
                return Ok(s);
            }
        }
    }

    bail!(
        "no _blossom TXT record found for pkarr key {}",
        public_key
    );
}

// ── Backend ────────────────────────────────────────────────────────────────

pub struct PkarrRemote {
    inner: BlossomRemote,
}

impl PkarrRemote {
    /// Create by resolving the PKARR key to a blossom endpoint.
    ///
    /// `zbase32` is the z-base-32 representation of the Ed25519 public key
    /// (the string you get from `pkarr publish --list` or from blossom-rs
    /// `pkarr_discovery`).
    pub fn resolve(zbase32: &str, repo: &str) -> Result<Self> {
        eprintln!("[pkarr] resolving key {}…", &zbase32[..zbase32.len().min(12)]);

        let public_key: pkarr::PublicKey = zbase32
            .parse()
            .map_err(|_| anyhow::anyhow!("invalid PKARR z-base-32 pubkey: {zbase32}"))?;

        let blossom_url = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .context("build tokio runtime")?
            .block_on(resolve_blossom_url(&public_key))?;

        eprintln!("[pkarr] resolved → {blossom_url}");

        // Derive the Nostr hex pubkey from environment
        let nostr_pubkey_hex = derive_nostr_pubkey()?;

        Ok(Self {
            inner: BlossomRemote::new(&blossom_url, &nostr_pubkey_hex, repo),
        })
    }
}

impl RemoteHelper for PkarrRemote {
    fn capabilities(&self) -> &[&'static str] {
        self.inner.capabilities()
    }

    fn list(&mut self, for_push: bool) -> Result<Vec<GitRef>> {
        self.inner.list(for_push)
    }

    fn fetch(&mut self, cmds: Vec<FetchCmd>) -> Result<()> {
        self.inner.fetch(cmds)
    }

    fn push(&mut self, specs: Vec<PushSpec>) -> Result<Vec<PushResult>> {
        self.inner.push(specs)
    }
}

// ── Nostr pubkey derivation ────────────────────────────────────────────────

/// Derive a Nostr hex pubkey from NOSTR_PUBKEY env (direct) or NOSTR_NSEC (derive).
fn derive_nostr_pubkey() -> Result<String> {
    // Explicit pubkey override
    if let Ok(pk) = std::env::var("NOSTR_PUBKEY") {
        if pk.len() == 64 && pk.chars().all(|c| c.is_ascii_hexdigit()) {
            return Ok(pk);
        }
        bail!("NOSTR_PUBKEY must be a 64-char hex string, got: {pk}");
    }

    // Derive from nsec
    let nsec = std::env::var("NOSTR_NSEC").context(
        "set NOSTR_NSEC (or NOSTR_PUBKEY) to your Nostr secret key for pkarr:// remotes",
    )?;

    let secret_key = decode_nsec(&nsec)?;
    let sk = secp256k1::SecretKey::from_slice(&secret_key).context("invalid secret key bytes")?;
    let secp = secp256k1::Secp256k1::new();
    let keypair = secp256k1::Keypair::from_secret_key(&secp, &sk);
    let (xonly, _) = keypair.x_only_public_key();
    Ok(hex::encode(xonly.serialize()))
}

// ── URL parser ─────────────────────────────────────────────────────────────

/// Parse a `pkarr://` URL into `(zbase32_pubkey, repo_name)`.
///
/// Format: `pkarr://<zbase32-pubkey>/<repo>`
pub fn parse_pkarr_url(url: &str) -> Result<(String, String)> {
    let rest = url
        .strip_prefix("pkarr://")
        .with_context(|| format!("not a pkarr:// URL: {url}"))?;

    let (key, repo) = rest
        .split_once('/')
        .with_context(|| format!("pkarr URL must be pkarr://<key>/<repo>, got: {url}"))?;

    if key.is_empty() {
        bail!("pkarr URL missing pubkey: {url}");
    }
    if repo.is_empty() {
        bail!("pkarr URL missing repo name: {url}");
    }

    Ok((key.to_string(), repo.trim_end_matches(".git").to_string()))
}
