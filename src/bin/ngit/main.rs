#![cfg_attr(not(test), warn(clippy::pedantic))]
#![allow(clippy::large_futures)]
#![cfg_attr(not(test), warn(clippy::expect_used))]

use anyhow::Result;
use clap::Parser;
use cli::{AccountCommands, NgitCli, NgitCommands};

mod cli;
use ngit::{cli_interactor, client, git, git_events, login, repo_ref};

mod sub_commands;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = NgitCli::parse();
    match &cli.command {
        NgitCommands::Account(args) => match &args.account_command {
            AccountCommands::Login(sub_args) => sub_commands::login::launch(&cli, sub_args).await,
            AccountCommands::Logout => sub_commands::logout::launch().await,
            AccountCommands::ExportKeys => sub_commands::export_keys::launch().await,
        },
        NgitCommands::Init(args) => sub_commands::init::launch(&cli, args).await,
        NgitCommands::List => sub_commands::list::launch().await,
        NgitCommands::Send(args) => sub_commands::send::launch(&cli, args, false).await,
    }
}
