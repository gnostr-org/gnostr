#![cfg_attr(not(test), warn(clippy::pedantic))]
#![cfg_attr(not(test), warn(clippy::expect_used))]
use crate::cli::Commands as NgitCommands;
use crate::sub_commands::fetch;
use crate::sub_commands::init;
use crate::sub_commands::list;
use crate::sub_commands::login;
use crate::sub_commands::pull;
use crate::sub_commands::push;
use crate::sub_commands::send;
use clap::Args;
use nostr_sdk::prelude::*;

use serde::ser::StdError;

#[derive(Args)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct NgitSubCommand {
    #[command(subcommand)]
    command: NgitCommands,
    ///// nsec or hex private key
    #[arg(short, long, global = true)]
    nsec: Option<String>,
    ///// password to decrypt nsec
    #[arg(short, long, global = true)]
    password: Option<String>,
    ///// disable spinner animations
    #[arg(long, action)]
    disable_cli_spinners: bool,
}

pub async fn ngit(sub_command_args: &NgitSubCommand) -> Result<(), Box<dyn StdError>> {
	match &sub_command_args.command {
        NgitCommands::Login(args) => login::launch(&args).await?,
        NgitCommands::Init(args) => init::launch(&args).await?,
        NgitCommands::Send(args) => send::launch(&args, true).await?,
        NgitCommands::List => list::launch().await?,
        NgitCommands::Pull => pull::launch().await?,
        NgitCommands::Push(args) => push::launch(&args).await?,
        NgitCommands::Fetch(args) => fetch::launch(&args).await?,
    }
    Ok(())
}
