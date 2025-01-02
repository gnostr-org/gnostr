#[cfg_attr(not(test), warn(clippy::pedantic))]
#[cfg_attr(not(test), warn(clippy::expect_used))]
use nostr_sdk::prelude::*;
//use anyhow::Result;
use clap::{Args, Parser, Subcommand};

//use ngit::*;
//
//use ngit::cli_interactor;
//use ngit::client;
//use ngit::config;
//use ngit::git;
//use ngit::key_handling;
//use ngit::login;
//use ngit::repo_ref;
use ngit::sub_commands;

#[derive(Args, Debug)]
pub struct NgitSubCommand {
    /// ngit --init
    #[arg(long, short, default_value_t = false)]
    init: bool,
    /// ngit --send
    #[arg(long, short, default_value_t = false)]
    send: bool,
    /// ngit --list
    #[arg(long, default_value_t = false)]
    list: bool,
    /// ngit --push
    #[arg(long, default_value_t = false)]
    push: bool,
    /// ngit --pull
    #[arg(long, default_value_t = false)]
    pull: bool,
    /// ngit --login
    #[arg(long, short, default_value_t = false)]
    login: bool,
    /// ngit --help
    #[arg(long, short, default_value_t = false)]
    ngit_help: bool,
}

pub async fn ngit(sub_command_args: &NgitSubCommand) -> Result<()> {
    if sub_command_args.init {
        println!("sub_command_args.init={}", sub_command_args.init);
    } else if sub_command_args.send {
        println!("sub_command_args.send={}", sub_command_args.send);
    } else if sub_command_args.list {
        println!("sub_command_args.list={}", sub_command_args.list);
    } else if sub_command_args.push {
        println!("sub_command_args.push={}", sub_command_args.push);
    } else if sub_command_args.pull {
        println!("sub_command_args.pull={}", sub_command_args.pull);
    } else if sub_command_args.login {
        println!("sub_command_args.login={}", sub_command_args.login);
    } else if sub_command_args.ngit_help {
        println!("sub_command_args.ngit_help={}", sub_command_args.ngit_help);
    } else {
        println!("sub_command_args={:?}", sub_command_args);
    }

    Ok(())
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
    /// nsec or hex private key
    #[arg(short, long, global = true)]
    nsec: Option<String>,
    /// password to decrypt nsec
    #[arg(short, long, global = true)]
    password: Option<String>,
    /// disable spinner animations
    #[arg(long, action)]
    disable_cli_spinners: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// signal you are this repo's maintainer accepting proposals via nostr
    Init(sub_commands::init::SubCommandArgs),
    /// issue commits as a proposal
    Send(sub_commands::send::SubCommandArgs),
    /// list proposals; checkout, apply or download selected
    List,
    /// send proposal revision
    Push(sub_commands::push::SubCommandArgs),
    /// fetch and apply new proposal commits / revisions linked to branch
    Pull,
    /// run with --nsec flag to change npub
    Login(sub_commands::login::SubCommandArgs),
}

//#[tokio::main]
//async fn main() -> Result<()> {
//    let cli = Cli::parse();
//    match &cli.command {
//        Commands::Login(args) => sub_commands::login::launch(&cli, args).await,
//        Commands::Init(args) => sub_commands::init::launch(&cli, args).await,
//        Commands::Send(args) => sub_commands::send::launch(&cli, args).await,
//        Commands::List => sub_commands::list::launch().await,
//        Commands::Pull => sub_commands::pull::launch().await,
//        Commands::Push(args) => sub_commands::push::launch(&cli, args).await,
//    }
//}
