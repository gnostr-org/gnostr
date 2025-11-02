use anyhow::{Context, Result};
use clap;

use crate::{
    //cli::Cli,
    client::Connect,
    git::Repo,
    login,
};

#[derive(clap::Args, Debug, Clone)]
pub struct LoginArgs {
    /// don't fetch user metadata and relay list from relays
    #[arg(long, action)]
    pub offline: bool,
    pub disable_cli_spinners: bool,
    pub password: Option<String>,
    pub nsec: Option<String>,
    pub bunker_app_key: Option<String>,
    pub bunker_uri: Option<String>,
}

pub async fn launch(
    //args: &Cli,
    args: &LoginArgs,
) -> Result<()> {
    let git_repo = Repo::discover().context("cannot find a git repository")?;
    if args.offline {
        login::launch(
            &git_repo,
            &args.bunker_uri,
            &args.bunker_app_key,
            &args.nsec,
            &args.password,
            None,
            true,
            false,
        )
        .await?;
        Ok(())
    } else {
        #[cfg(test)]
        let client: &crate::client::MockConnect = &mut Default::default();
        #[cfg(not(test))]
        let client = Client::default();

        login::launch(
            &git_repo,
            &args.bunker_uri,
            &args.bunker_app_key,
            &args.nsec,
            &args.password,
            Some(&client),
            true,
            false,
        )
        .await?;
        client.disconnect().await?;
        Ok(())
    }
}
