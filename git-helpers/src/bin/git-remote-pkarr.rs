//! `git-remote-pkarr` — discover Blossom servers via PKARR DHT.
//!
//! ## Setup
//! ```bash
//! # The server owner must have published a PKARR record with _blossom TXT.
//! # Get their z-base-32 PKARR public key.
//!
//! export NOSTR_NSEC=nsec1...         # your key (for upload auth + pubkey derivation)
//! git clone pkarr://<zbase32key>/my-repo
//! ```
//!
//! ## Publishing your PKARR record
//! Use `blossom-server` with the `pkarr-discovery` feature, or the `pkarr` CLI:
//! ```bash
//! pkarr publish --key <your-ed25519-key> '_blossom=https://blossom.example.com'
//! ```

use anyhow::{Context, Result};

use gnostr_git_helpers::pkarr_backend::{PkarrRemote, parse_pkarr_url};
use gnostr_git_helpers::protocol::run_helper;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_env("GIT_REMOTE_PKARR_LOG")
                .add_directive(tracing::Level::WARN.into()),
        )
        .init();

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        anyhow::bail!("usage: git-remote-pkarr <remote> <url>");
    }
    let url = &args[2];

    let (zbase32, repo) = parse_pkarr_url(url)
        .with_context(|| format!("invalid pkarr:// URL: {url}"))?;

    let helper = PkarrRemote::resolve(&zbase32, &repo)
        .with_context(|| format!("failed to resolve pkarr key {zbase32}"))?;

    run_helper(helper)
}
