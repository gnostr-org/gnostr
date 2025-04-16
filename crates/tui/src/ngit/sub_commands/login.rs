use anyhow::{Context, Result};
use clap;

use crate::{
	cli::Cli,
	client::{Client, Connect},
	git::Repo,
	login,
};

#[derive(clap::Args, Debug)]
pub struct SubCommandArgs {
	/// don't fetch user metadata and relay list from relays
	#[arg(long, action)]
	offline: bool,
	/// remote signer address
	#[arg(long, global = true)]
	pub bunker_uri: Option<String>,
	/// remote signer app secret key
	#[arg(long, global = true)]
	pub bunker_app_key: Option<String>,
	/// nsec or hex private key
	#[arg(short, long, global = true)]
	pub nsec: Option<String>,
	/// password to decrypt nsec
	#[arg(short, long, global = true)]
	pub password: Option<String>,
	/// disable spinner animations
	#[arg(long, action)]
	pub disable_cli_spinners: bool,
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
