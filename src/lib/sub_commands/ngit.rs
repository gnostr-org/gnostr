#![cfg_attr(not(test), warn(clippy::pedantic))]
#![cfg_attr(not(test), warn(clippy::expect_used))]
use clap::Args;
use ngit::sub_commands::init;
use ngit::sub_commands::list;
use ngit::sub_commands::login;
use ngit::sub_commands::pull;
use ngit::sub_commands::push;
use ngit::sub_commands::send;
use ngit::sub_commands::fetch;
use ngit::cli::Commands as NgitCommands;
use nostr_sdk::prelude::*;

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

pub async fn ngit(sub_command_args: &NgitSubCommand) -> Result<()> {
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
