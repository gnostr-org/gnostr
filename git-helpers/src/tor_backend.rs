//! Tor SOCKS5 backend for `git-remote-tor`.
//!
//! Wraps [`BlossomRemote`] with a reqwest client that routes all HTTP through
//! a Tor SOCKS5 proxy.  Identical to `git-remote-blossom` in every way except
//! that the HTTP connection is anonymized via Tor.
//!
//! ## URL format
//! ```text
//! blossom+onion://<.onion-host>/<pubkey-hex>/<repo>
//! tor://<.onion-host>/<pubkey-hex>/<repo>
//! ```
//!
//! ## Environment variables
//! | Variable       | Default                        | Description                   |
//! |----------------|--------------------------------|-------------------------------|
//! | `SOCKS5_PROXY` | `socks5h://127.0.0.1:9050`     | Tor SOCKS5 proxy URL          |
//! | `NOSTR_NSEC`   | —                              | nsec1… or hex for upload auth |
//!
//! ## Notes
//! - Uses `socks5h://` (DNS via SOCKS) by default so `.onion` hostnames are
//!   resolved by the Tor daemon, not the local resolver.
//! - Requires a running Tor daemon (e.g. `brew services start tor` or
//!   `sudo systemctl start tor`).
//! - The Blossom server must be reachable as a Tor hidden service (`.onion`).

use anyhow::{Context, Result, bail};

use crate::blossom_backend::{BlossomRemote, parse_blossom_url};
use crate::protocol::{FetchCmd, GitRef, PushResult, PushSpec, RemoteHelper};

// ── Backend ────────────────────────────────────────────────────────────────

pub struct TorRemote {
    inner: BlossomRemote,
}

impl TorRemote {
    /// Build a Tor-proxied blossom remote from a `blossom+onion://` or `tor://` URL.
    pub fn new(url: &str) -> Result<Self> {
        let (server, pubkey, repo) = parse_tor_url(url)?;
        let proxy_url = std::env::var("SOCKS5_PROXY")
            .unwrap_or_else(|_| "socks5h://127.0.0.1:9050".into());

        eprintln!("[tor] routing via {proxy_url}");

        let nsec = std::env::var("NOSTR_NSEC").ok();
        if nsec.is_none() {
            eprintln!("[tor] NOSTR_NSEC not set — uploads will be unauthenticated");
        }

        let proxy = reqwest::Proxy::all(&proxy_url)
            .with_context(|| format!("invalid SOCKS5_PROXY URL: {proxy_url}"))?;

        let client = reqwest::blocking::Client::builder()
            .proxy(proxy)
            .user_agent("git-remote-tor/0.1")
            .build()
            .context("build Tor HTTP client")?;

        Ok(Self {
            inner: BlossomRemote::with_client(client, nsec, &server, &pubkey, &repo),
        })
    }
}

impl RemoteHelper for TorRemote {
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

// ── URL parser ─────────────────────────────────────────────────────────────

/// Parse a `blossom+onion://` or `tor://` URL into `(server_url, pubkey, repo)`.
///
/// Accepted formats:
/// - `blossom+onion://<host.onion>/<pubkey>/<repo>`
/// - `tor://<host.onion>/<pubkey>/<repo>`
///
/// The `.onion` host is kept as-is; the HTTP scheme is always `http://` since
/// Tor hidden services don't use TLS (the Tor circuit provides encryption).
pub fn parse_tor_url(url: &str) -> Result<(String, String, String)> {
    // Normalise to blossom+http:// so parse_blossom_url can handle it
    if let Some(rest) = url.strip_prefix("blossom+onion://") {
        return parse_blossom_url(&format!("blossom+http://{rest}"));
    }
    if let Some(rest) = url.strip_prefix("tor://") {
        return parse_blossom_url(&format!("blossom+http://{rest}"));
    }
    bail!("not a blossom+onion:// or tor:// URL: {url}");
}
