use anyhow::{Context, Result};
use ngit::{
    client::{fetching_with_report, Client, Connect, Params},
    git::{Repo, RepoActions, nostr_url::NostrUrlDecoded},
    repo_ref::get_repo_coordinates_when_remote_unknown,
};
use nostr::nips::nip19::Nip19Coordinate;

#[derive(Clone, Debug, clap::Args)]
pub struct SubCommandArgs {
    /// address pointer to repo announcement
    #[arg(long)]
    pub repo: Vec<String>,
}

pub async fn launch(args: &SubCommandArgs) -> Result<()> {
    let git_repo = Repo::discover().context("cannot find a git repository")?;
    let client = Client::new(Params::with_git_config_relay_defaults(&Some(&git_repo)));

    let repo_coordinates: Vec<Nip19Coordinate> = if args.repo.is_empty() {
        vec![get_repo_coordinates_when_remote_unknown(&git_repo, &client).await?]
    } else {
        let mut out = Vec::with_capacity(args.repo.len());
        for repo in &args.repo {
            out.push(
                NostrUrlDecoded::parse_and_resolve(repo, &Some(&git_repo))
                    .await?
                    .coordinate,
            );
        }
        out
    };

    for repo_coordinate in repo_coordinates {
        fetching_with_report(git_repo.get_path()?, &client, &repo_coordinate).await?;
    }
    client.disconnect().await?;
    Ok(())
}
