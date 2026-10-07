use anyhow::{Context, Result};
use gnostr_git_helpers::protocol::run_helper;
use gnostr_git_helpers::tor_backend::TorRemote;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let url = args.get(2).context("usage: git-remote-tor <remote> <url>")?;

    let helper = TorRemote::new(url)?;
    run_helper(helper)
}
