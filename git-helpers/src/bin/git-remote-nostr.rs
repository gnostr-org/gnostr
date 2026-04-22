//! git-remote-nostr — access NIP-34 git repositories via Nostr relays.
//!
//! Resolves a `nostr://` URL to a GRASP HTTP smart-protocol URL by querying
//! a Nostr relay for the kind:30617 (RepoAnnounce) event, then delegates
//! all git transport to that HTTP URL.
//!
//! # Usage
//! ```bash
//! # With relay in URL
//! git clone nostr+wss://relay.example.com/<npub>/<repo>
//!
//! # With relay from environment
//! export NOSTR_RELAY=wss://relay.example.com
//! git clone nostr://<npub>/<repo>
//!
//! # Push (requires NOSTR_NSEC for auth)
//! export NOSTR_NSEC=nsec1...
//! git push origin main
//!
//! # Skip relay lookup — use a known GRASP server directly
//! export GRASP_SERVER=https://grasp.example.com
//! git clone nostr://<npub>/<repo>
//! ```
//!
//! # Environment
//! - `NOSTR_RELAY`  — WebSocket relay URL (wss://…)
//! - `NOSTR_NSEC`   — signing key for push auth (nsec1… or hex)
//! - `GRASP_SERVER` — skip relay lookup, use this HTTP server base URL

use gnostr_git_helpers::nostr_backend::{parse_nostr_url, NostrRemote};
use gnostr_git_helpers::protocol::run_helper;

fn main() {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_env("GIT_REMOTE_NOSTR_LOG"),
        )
        .init();

    let args: Vec<String> = std::env::args().collect();
    // git calls: git-remote-nostr <remote-name> <url>
    let url = args.get(2).map(|s| s.as_str()).unwrap_or_else(|| {
        eprintln!("usage: git-remote-nostr <remote> <url>");
        eprintln!("  url: nostr://<npub>/<repo>");
        eprintln!("       nostr+wss://<relay-host>/<npub>/<repo>");
        std::process::exit(1);
    });

    let (relay, pubkey_hex, repo) = parse_nostr_url(url).unwrap_or_else(|e| {
        eprintln!("git-remote-nostr: invalid URL '{url}': {e}");
        std::process::exit(1);
    });

    eprintln!("[nostr] relay={relay}  pubkey={}…  repo={repo}", &pubkey_hex[..8]);

    let helper = NostrRemote::new(&relay, &pubkey_hex, &repo);
    if let Err(e) = run_helper(helper) {
        eprintln!("git-remote-nostr: fatal: {e:#}");
        std::process::exit(1);
    }
}
