use anyhow::{Context, Result};
use gnostr_git_helpers::nostr_backend::{parse_nostr_url, NostrRemote};
use gnostr_git_helpers::protocol::run_helper;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let url = args.get(2).context("usage: git-remote-nostr <remote> <url>")?;

    let (relay, pubkey, repo) = parse_nostr_url(url)?;
    let helper = NostrRemote::new(&relay, &pubkey, &repo);
    run_helper(helper)
}
