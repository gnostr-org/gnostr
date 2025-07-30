#![cfg_attr(not(test), warn(clippy::pedantic))]
#![cfg_attr(not(test), warn(clippy::expect_used))]
use crate::cli::LegitCommands;
use crate::sub_commands::fetch;
use crate::sub_commands::init;
use crate::sub_commands::list;
use crate::sub_commands::login;
use crate::sub_commands::pull;
use crate::sub_commands::push;
use crate::sub_commands::send;
use clap::Args;
use nostr_sdk_0_34_0::prelude::*;

use serde::ser::StdError;

#[derive(Args, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct LegitSubCommand {
    #[command(subcommand)]
    command: LegitCommands,
    ///// nsec or hex private key
    #[arg(short, long, global = true)]
    nsec: Option<String>,
    ///// password to decrypt nsec
    #[arg(short, long, global = true)]
    password: Option<String>,
    ///// nsec or hex private key
    #[arg(short, long, global = true)]
    repo: Option<String>,
    ///// password to decrypt nsec
    #[arg(long, global = true)]
    pow: Option<String>,
    ///// disable spinner animations
    #[arg(long, action)]
    disable_cli_spinners: bool,
}

pub async fn legit(sub_command_args: &LegitSubCommand) -> Result<(), Box<dyn StdError>> {
    match &sub_command_args.command {
        LegitCommands::Login(args) => login::launch(&args).await?,
        LegitCommands::Init(args) => init::launch(&args).await?,
        LegitCommands::Send(args) => send::launch(&args, true).await?,
        LegitCommands::List => list::launch().await?,
        LegitCommands::Pull => pull::launch().await?,
        LegitCommands::Push(args) => push::launch(&args).await?,
        LegitCommands::Fetch(args) => fetch::launch(&args).await?,
    }
    Ok(())
}
