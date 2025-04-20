use anyhow::{Context, Result};
use clap;

use crate::{
	cli::Cli,
	client::{Client, Connect},
	git::Repo,
	login,
};

#[derive(clap::Args)]
pub struct SubCommandArgs {
	/// don't fetch user metadata and relay list from relays
	#[arg(long, action)]
	offline: bool,
    disable_cli_spinners: Option<bool>,
    password: Option<String>,
    nsec: Option<String>,
    bunker_app_key: Option<String>,
    bunker_uri: Option<String>,
}

pub async fn launch(
	//args: &Cli,
	args: &SubCommandArgs,
) -> Result<()> {
	let git_repo =
		Repo::discover().context("cannot find a git repository")?;
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
