//! `git-remote-tor` — Blossom git remote over Tor hidden services.
//!
//! ## Setup
//! ```bash
//! # Ensure Tor daemon is running
//! brew services start tor   # macOS
//! sudo systemctl start tor  # Linux
//!
//! # The Blossom server must be running as a hidden service (.onion address)
//! export NOSTR_NSEC=nsec1...
//! git clone blossom+onion://<your-pubkey-hex.onion>/<pubkey-hex>/my-repo
//!
//! # Or using the tor:// scheme alias
//! git clone tor://<host.onion>/<pubkey-hex>/my-repo
//! ```
//!
//! ## Proxy
//! Set `SOCKS5_PROXY=socks5h://127.0.0.1:9050` (default) to point at your Tor daemon.
//! Use `socks5h://` (not `socks5://`) so `.onion` hostnames resolve via Tor.

use anyhow::{Context, Result};

use gnostr_git_helpers::protocol::run_helper;
use gnostr_git_helpers::tor_backend::TorRemote;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_env("GIT_REMOTE_TOR_LOG")
                .add_directive(tracing::Level::WARN.into()),
        )
        .init();

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        anyhow::bail!("usage: git-remote-tor <remote> <url>");
    }
    let url = &args[2];

    let helper =
        TorRemote::new(url).with_context(|| format!("failed to create Tor remote for {url}"))?;

    run_helper(helper)
}
