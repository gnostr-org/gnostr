#![cfg_attr(not(test), warn(clippy::pedantic))]
#![allow(clippy::large_futures)]
#![cfg_attr(not(test), warn(clippy::expect_used))]

use anyhow::Result;

use ngit;
use ngit::cli::Cli;
use ngit::cli::{Commands, AccountCommands};
use ngit::sub_commands::*;
use ngit::sub_commands::send;
use ngit::sub_commands::list;
use ngit::sub_commands::init;
use ngit::sub_commands::login;
use ngit::sub_commands::export_keys;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Account(args) => match &args.account_command {
            AccountCommands::Login(sub_args) => login::login::launch(&cli, sub_args).await,
            AccountCommands::Logout => logout::launch().await,
            AccountCommands::ExportKeys => export_keys::launch().await,
        },
        Commands::Init(args) => init::launch(&cli, args).await,
        Commands::List => list::launch().await,
        Commands::Send(args) => send::launch(&cli, args, false).await,
    }
}
