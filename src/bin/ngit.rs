#![cfg_attr(not(test), warn(clippy::pedantic))]
#![allow(clippy::large_futures)]
#![cfg_attr(not(test), warn(clippy::expect_used))]
#![allow(clippy::single_match_else)]
use anyhow::Result;
use clap::Parser;
use ngit::cli::{Cli, Commands};

use ngit::{
	cli, cli_interactor, client, git, git_events, login, repo_ref,
};

use ngit::sub_commands;

#[tokio::main]
async fn main() -> Result<()> {
	let cli = Cli::parse();
	match &cli.command {
		Commands::Fetch(args) => {
			sub_commands::fetch::launch(&cli, args).await
		}
		Commands::Login(args) => {
			sub_commands::login::launch(&cli, args).await
		}
		Commands::Init(args) => {
			sub_commands::init::launch(&cli, args).await
		}
		Commands::Send(args) => {
			sub_commands::send::launch(&cli, args, false).await
		}
		Commands::List => sub_commands::list::launch().await,
		Commands::Pull => sub_commands::pull::launch().await,
		Commands::Push(args) => {
			sub_commands::push::launch(&cli, args).await
		}
	}
}
