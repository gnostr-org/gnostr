use anyhow::{Context, Result};
use clap;
use nostr::nips::{nip01::Coordinate, nip19::Nip19Coordinate};

use ngit::{
    client::{fetching_with_report, Client, Connect},
    git::{Repo, RepoActions},
    repo_ref::get_repo_coordinates_when_remote_unknown,
};

#[derive(clap::Args, Debug, Clone)]
pub struct FetchArgs {
    /// address pointer to repo announcement
    #[arg(long)]
    pub repo: Vec<String>,
}

pub async fn launch(
    //args: &Cli,
    args: &FetchArgs,
) -> Result<()> {
    let git_repo = Repo::discover().context("cannot find a git repository")?;

    #[cfg(test)]
    let client: &crate::client::MockConnect = &mut Default::default();
    #[cfg(not(test))]
    let client = Client::default();

    let repo_coordinates: Nip19Coordinate = if args.repo.is_empty() {
        get_repo_coordinates_when_remote_unknown(&git_repo, &client).await?
    } else {
        Nip19Coordinate {
            coordinate: Coordinate::parse(&args.repo[0])?,
            relays: vec![],
        }
    };
    fetching_with_report(git_repo.get_path()?, &client, &repo_coordinates).await?;
    client.disconnect().await?;
    Ok(())
}
