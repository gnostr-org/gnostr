#![cfg_attr(not(test), warn(clippy::pedantic))]
#![cfg_attr(not(test), warn(clippy::expect_used))]
use crate::cli::AccountCommands;
use crate::cli::NgitCli;
use crate::cli::NgitCommands;

use crate::sub_commands::export_keys;
use crate::sub_commands::init;
use crate::sub_commands::list;
use crate::sub_commands::login;
use crate::sub_commands::logout;
use crate::sub_commands::send;

use clap::Args;
use nostr_sdk_0_34_0::prelude::*;

use serde::ser::StdError;

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
    ///// disable spinner animations
    #[arg(long, action)]
    pub disable_cli_spinners: bool,
}

pub async fn ngit(
    ngit_cli: &NgitCli,
    sub_command_args: &NgitSubCommand,
) -> Result<(), Box<dyn StdError>> {
    match &sub_command_args.command {
        NgitCommands::Account(args) => match &args.account_command {
            AccountCommands::Login(sub_args) => login::launch(&ngit_cli, sub_args).await?,
            AccountCommands::Logout => logout::launch().await?,
            AccountCommands::ExportKeys => export_keys::launch().await?,
        },

        //NgitCommands::Login(args) => login::launch(ngit_cli, &args).await?,
        NgitCommands::Init(args) => init::launch(ngit_cli, &args).await?,
        NgitCommands::Send(args) => send::launch(ngit_cli, &args, true).await?,
        NgitCommands::List => list::launch().await?,
    }
    Ok(())
}
