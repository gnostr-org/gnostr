#![cfg_attr(not(test), warn(clippy::pedantic))]
#![cfg_attr(not(test), warn(clippy::expect_used))]
use crate::cli::ChatCommands;
use crate::sub_commands::fetch;
use crate::sub_commands::init;
use crate::sub_commands::list;
use crate::sub_commands::login;
use crate::sub_commands::pull;
use crate::sub_commands::push;
use crate::sub_commands::send;
use clap::Args;
use nostr_sdk::prelude::*;

#[derive(Args)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct ChatSubCommand {
    #[command(subcommand)]
    command: ChatCommands,
    ///// nsec or hex private key
    #[arg(short, long, global = true)]
    nsec: Option<String>,
    ///// password to decrypt nsec
    #[arg(short, long, global = true)]
    password: Option<String>,
    ///// chat topic
    #[arg(short, long, global = true)]
    topic: Option<String>,
    ///// disable spinner animations
    #[arg(long, action)]
    disable_cli_spinners: bool,
}

pub async fn chat(sub_command_args: &ChatSubCommand) -> Result<()> {
    match &sub_command_args.command {
        ChatCommands::Login(args) => login::launch(&args).await?,
        ChatCommands::Init(args) => init::launch(&args).await?,
        ChatCommands::Send(args) => send::launch(&args, true).await?,
        ChatCommands::List => list::launch().await?,
        ChatCommands::Pull => pull::launch().await?,
        ChatCommands::Push(args) => push::launch(&args).await?,
        ChatCommands::Fetch(args) => fetch::launch(&args).await?,
    }
    Ok(())
}
