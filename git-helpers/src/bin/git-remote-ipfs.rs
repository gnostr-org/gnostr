//! `git-remote-ipfs` — store git repositories in IPFS via Kubo MFS.
//!
//! ## Setup
//! ```bash
//! # Ensure ipfs daemon is running
//! ipfs daemon &
//!
//! # Add a remote
//! git remote add origin ipfs://my-repo
//! git push origin main
//!
//! # Clone
//! git clone ipfs://my-repo
//! ```
//!
//! Set `IPFS_API=http://127.0.0.1:5001` (default) to point at your Kubo node.

use anyhow::{Context, Result};

use gnostr_git_helpers::ipfs_backend::{parse_ipfs_url, IpfsRemote};
use gnostr_git_helpers::protocol::run_helper;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_env("GIT_REMOTE_IPFS_LOG")
                .add_directive(tracing::Level::WARN.into()),
        )
        .init();

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        anyhow::bail!("usage: git-remote-ipfs <remote> <url>");
    }
    let url = &args[2];

    let (api, repo) = parse_ipfs_url(url).with_context(|| format!("invalid ipfs:// URL: {url}"))?;

    eprintln!("[ipfs] api={api} repo={repo}");
    let helper = IpfsRemote::new(&api, &repo);
    run_helper(helper)
}
