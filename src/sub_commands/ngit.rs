#![cfg_attr(not(test), warn(clippy::pedantic))]
#![cfg_attr(not(test), warn(clippy::expect_used))]
use clap::Args;
use gnostr_ngit::sub_commands::init;
use gnostr_ngit::sub_commands::list;
use gnostr_ngit::sub_commands::login;
use gnostr_ngit::sub_commands::pull;
use gnostr_ngit::sub_commands::push;
use gnostr_ngit::sub_commands::send;
use gnostr_ngit::Commands as NgitCommands;
use nostr_sdk::prelude::*;

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

pub async fn ngit(sub_command_args: &NgitSubCommand) -> Result<()> {
    use gnostr_ngit::Commands;
    match &sub_command_args.command {
        Commands::Login(args) => login::launch(&args).await?,
        Commands::Init(args) => init::launch(&args).await?,
        Commands::Send(args) => send::launch(&args).await?,
        Commands::List => list::launch().await?,
        Commands::Pull => pull::launch().await?,
        Commands::Push(args) => push::launch(&args).await?,
    }
    Ok(())
}
