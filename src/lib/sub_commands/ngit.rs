#![cfg_attr(not(test), warn(clippy::pedantic))]
#![cfg_attr(not(test), warn(clippy::expect_used))]
use crate::cli::{NgitCli, NgitCommands};
use crate::sub_commands::init;
use crate::sub_commands::list;
use crate::sub_commands::login;
use crate::sub_commands::pull;
use crate::sub_commands::send;

use crate::sub_commands::init::SubCommandArgs;
//use crate::sub_commands::login::SubCommandArgs;
//use crate::sub_commands::send::SubCommandArgs;

use clap::Args;
use nostr_sdk_0_37_0::prelude::*;

use serde::ser::StdError;

#[derive(Args, Debug)]
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

pub async fn ngit(ngit_cli: &NgitCli, sub_command_args: &SubCommandArgs) -> Result<(), Box<dyn StdError>> {
    match &sub_command_args.command {
        NgitCommands::Login(args) => login::launch(ngit_cli, sub_command_args).await?,
        NgitCommands::Init(args) => init::launch(ngit_cli, sub_command_args).await?,
        NgitCommands::Send(args) => send::launch(ngit_cli, sub_command_args, true).await?,
        NgitCommands::List => list::launch().await?,
        NgitCommands::Pull => pull::launch().await?,
    }
    Ok(())
}
