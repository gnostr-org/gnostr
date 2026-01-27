#![cfg_attr(not(test), warn(clippy::pedantic))]
#![cfg_attr(not(test), warn(clippy::expect_used))]
use clap::Args;
use serde::ser::StdError;

use crate::{
    cli::NgitCommands,
    sub_commands::{fetch, init, list, login, pull, push, query, send},
    types::{Event, EventKind, Keys, Tag},
};

#[derive(Args, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct NgitSubCommand {
    #[command(subcommand)]
    pub command: NgitCommands,
    ///// nsec or hex private key
    #[arg(short, long, global = true)]
    pub nsec: Option<String>,
    ///// password to decrypt nsec
    #[arg(short, long, global = true)]
    pub password: Option<String>,
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub disable_cli_spinners: bool,
}

/// ngit
///
/// # Errors
///
/// This function will return an error if the command fails.
pub async fn ngit(sub_command_args: &NgitSubCommand) -> Result<(), Box<dyn StdError>> {
    match &sub_command_args.command {
        NgitCommands::Login(args) => login::launch(args).await?,
        NgitCommands::Init(args) => init::launch(args).await?,
        NgitCommands::Send(args) => send::launch(args, true).await?,
        NgitCommands::List => list::launch().await?,
        NgitCommands::Pull => pull::launch().await?,
        NgitCommands::Push(args) => push::launch(args).await?,
        NgitCommands::Fetch(args) => fetch::launch(args).await?,
        NgitCommands::Query(args) => query::launch(args).await?,
    }
    Ok(())
}
