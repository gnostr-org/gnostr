use anyhow::{Context, Result};
use gnostr_git_helpers::pkarr_backend::{parse_pkarr_url, PkarrRemote};
use gnostr_git_helpers::protocol::run_helper;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let url = args.get(2).context("usage: git-remote-pkarr <remote> <url>")?;

    let (zbase32, repo) = parse_pkarr_url(url)?;
    let helper = PkarrRemote::resolve(&zbase32, &repo)?;
    run_helper(helper)
}
