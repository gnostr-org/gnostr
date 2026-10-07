use anyhow::{Context, Result};
use gnostr_git_helpers::blossom_backend::{parse_blossom_url, BlossomRemote};
use gnostr_git_helpers::protocol::run_helper;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let url = args.get(2).context("usage: git-remote-blossom <remote> <url>")?;

    let (server, pubkey, repo) = parse_blossom_url(url)?;
    let helper = BlossomRemote::new(&server, &pubkey, &repo);
    run_helper(helper)
}
