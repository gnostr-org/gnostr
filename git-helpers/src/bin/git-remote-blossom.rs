//! git-remote-blossom — store git repositories on a Blossom blob server.
//!
//! # Usage
//! ```bash
//! export NOSTR_NSEC=nsec1...
//! git clone blossom://<host>/<pubkey-hex>/<repo>
//! git remote add blossom blossom://<host>/<pubkey-hex>/<repo>
//! git push blossom main
//! ```
//!
//! # URL format
//! ```
//! blossom://<server-host>/<pubkey-hex>/<repo-name>
//! blossom+https://<server-host>/<pubkey-hex>/<repo-name>
//! blossom+http://<server-host>/<pubkey-hex>/<repo-name>   (dev only)
//! ```
//!
//! # Environment
//! - `NOSTR_NSEC` — signing key for BUD-01 upload auth (nsec1… or hex)

use gnostr_git_helpers::blossom_backend::{parse_blossom_url, BlossomRemote};
use gnostr_git_helpers::protocol::run_helper;

fn main() {
    // Initialise stderr logging (git ignores our stderr)
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_env("GIT_REMOTE_BLOSSOM_LOG"),
        )
        .init();

    let args: Vec<String> = std::env::args().collect();
    // git calls: git-remote-blossom <remote-name> <url>
    let url = args.get(2).map(|s| s.as_str()).unwrap_or_else(|| {
        eprintln!("usage: git-remote-blossom <remote> <url>");
        eprintln!("  url: blossom://<host>/<pubkey-hex>/<repo>");
        std::process::exit(1);
    });

    let (server, pubkey, repo) = parse_blossom_url(url).unwrap_or_else(|e| {
        eprintln!("git-remote-blossom: invalid URL '{url}': {e}");
        std::process::exit(1);
    });

    eprintln!("[blossom] server={server}  pubkey={}…  repo={repo}", &pubkey[..8]);

    let helper = BlossomRemote::new(&server, &pubkey, &repo);
    if let Err(e) = run_helper(helper) {
        eprintln!("git-remote-blossom: fatal: {e:#}");
        std::process::exit(1);
    }
}
