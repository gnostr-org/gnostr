use std::collections::HashSet;

use anyhow::{Context, Result};
use clap;
use nostr_0_34_1::nips::nip01::Coordinate;

use crate::{
    //cli::Cli,
    client::{fetching_with_report, Client, Connect},
    git::{Repo, RepoActions},
    repo_ref::get_repo_coordinates,
};

#[derive(clap::Args, Debug)]
pub struct SubCommandArgs {
    /// address pointer to repo announcement
    #[arg(long, action)]
    repo: Vec<String>,
}

pub async fn launch(
    //args: &Cli,
    args: &SubCommandArgs,
) -> Result<()> {
    let _ = args;
    let git_repo = Repo::discover().context("cannot find a git repository")?;
    let client = Client::default();

    #[cfg(test)]
    let mut client: &crate::client::MockConnect = &mut Default::default();
    #[cfg(not(test))]
    let mut client = Client::default();

    let repo_coordinates = if args.repo.is_empty() {
        get_repo_coordinates(&git_repo, &client).await?
    } else {
        let mut repo_coordinates = HashSet::new();
        for repo in &args.repo {
            repo_coordinates.insert(Coordinate::parse(repo.clone())?);
        }
        repo_coordinates
    };
    fetching_with_report(git_repo.get_path()?, &client, &repo_coordinates).await?;
    client.disconnect().await?;
    Ok(())
}
