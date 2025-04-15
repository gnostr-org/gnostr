#![cfg_attr(not(test), warn(clippy::pedantic))]
#![cfg_attr(not(test), warn(clippy::expect_used))]
use clap::Args;
use gnostr_ngit::sub_commands::*;
use gnostr_ngit::sub_commands::init;
//use gnostr_ngit::sub_commands::list;
//use gnostr_ngit::sub_commands::login;
//use gnostr_ngit::sub_commands::pull;
//use gnostr_ngit::sub_commands::push;
//use gnostr_ngit::sub_commands::send;
use gnostr_ngit::cli::Commands as NgitCommands;
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
    use gnostr_ngit::cli::Commands;
    match &sub_command_args.command {
        //NgitCommands::Login(args) => gnostr_ngit::sub_commands::login::launch(&args).await?,
        NgitCommands::Init(args) => gnostr_ngit::sub_commands::init::launch(&args).await?,
        NgitCommands::Send(args) => gnostr_ngit::sub_commands::send::launch(&args, true).await?,
        NgitCommands::List => gnostr_ngit::sub_commands::list::launch().await?,
        //NgitCommands::Pull => gnostr_ngit::sub_commands::pull::launch().await?,
        //NgitCommands::Push(args) => gnostr_ngit::sub_commands::push::launch(&args).await?,
		&gnostr_ngit::cli::Commands::Account(_) => todo!(),
    }
    Ok(())
}
