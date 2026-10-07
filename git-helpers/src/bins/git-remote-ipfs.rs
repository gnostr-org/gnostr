use anyhow::{Context, Result};
use gnostr_git_helpers::ipfs_backend::{parse_ipfs_url, IpfsRemote};
use gnostr_git_helpers::protocol::run_helper;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let url = args.get(2).context("usage: git-remote-ipfs <remote> <url>")?;

    let (api, repo) = parse_ipfs_url(url)?;
    let helper = IpfsRemote::new(&api, &repo);
    run_helper(helper)
}
