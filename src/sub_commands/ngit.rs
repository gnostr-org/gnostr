#![cfg_attr(not(test), warn(clippy::pedantic))]
#![cfg_attr(not(test), warn(clippy::expect_used))]

use nostr_sdk::prelude::*;
//use ngit::*;
//use anyhow::Result;
use clap::{Args, Parser, Subcommand};

use crate::sub_commands as gnostr_subcommands;
use crate::sub_commands::init;
use crate::sub_commands::list;
use crate::sub_commands::login;
use crate::sub_commands::pull;
use crate::sub_commands::push;
use crate::sub_commands::send;
//
use crate::Cli as GnostrCli;
use crate::Commands as GnostrCommands;

use ngit::Commands as NgitCommands;
//use ngit::Subcommand as NgitSubcommand;
use ngit::Cli as NgitCli;

//use ngit::cli_interactor;
//use ngit::client;
//use ngit::config;
//use ngit::git;
//use ngit::key_handling;
//use ngit::login;
//use ngit::repo_ref;
//use ngit::sub_commands as Ngit_Sub_Commands;
//use ngit::sub_commands::*;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct LocalCli {
    #[command(subcommand)]
    command: LocalCommands,
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
enum LocalCommands {
    /// signal you are this repo's maintainer accepting proposals via nostr
    Init(init::SubCommandArgs),
    /// issue commits as a proposal
    Send(send::SubCommandArgs),
    /// list proposals; checkout, apply or download selected
    List,
    /// send proposal revision
    Push(push::SubCommandArgs),
    /// fetch and apply new proposal commits / revisions linked to branch
    Pull,
    /// run with --nsec flag to change npub
    Login(login::SubCommandArgs),
}

#[derive(Args, Debug)]
pub struct NgitSubCommand {
    /// ngit --init
    #[arg(long, default_value_t = false)]
    init: bool,
    /// ngit --send
    #[arg(long, default_value_t = false)]
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
    #[arg(long, default_value_t = false)]
    login: bool,
    /// ngit --help
    #[arg(long, default_value_t = false)]
    ngit_help: bool,
    /// Prefixes
    #[arg(short, long, required = false, action = clap::ArgAction::Append)]
    prefixes: Vec<String>,
    /// Vanity pubkey in hex format
    #[arg(long, default_value_t = false)]
    hex: bool,
}

pub async fn ngit(sub_command_args: &NgitSubCommand) -> Result<()> {
    //let cli = NgitCli::parse();
    if sub_command_args.init {
        println!("ngit:sub_command_args.init={}", sub_command_args.init);
    } else if sub_command_args.send {
        println!("ngit:sub_command_args.send={}", sub_command_args.send);
    } else if sub_command_args.list {
        println!("ngit:sub_command_args.list={}", sub_command_args.list);
    } else if sub_command_args.push {
        println!("ngit:sub_command_args.push={}", sub_command_args.push);
    } else if sub_command_args.pull {
        println!("ngit:sub_command_args.pull={}", sub_command_args.pull);
    } else if sub_command_args.login {
        println!("ngit:sub_command_args.login={}", sub_command_args.login);
    } else if sub_command_args.ngit_help {
        println!(
            "ngit:sub_command_args.ngit_help={}",
            sub_command_args.ngit_help
        );
    } else if sub_command_args.prefixes.len() > 0 {
        let num_cores = num_cpus::get();
        let keys = Keys::vanity(
            sub_command_args.prefixes.clone(),
            !sub_command_args.hex,
            num_cores,
        )?;

        if sub_command_args.hex {
            println!("Public key (hex): {}", keys.public_key());
        } else {
            println!("Public key: {}", keys.public_key().to_bech32()?);
        }

        println!("Private key: {}", keys.secret_key()?.to_bech32()?);
    } else {
        println!("ngit:else:sub_command_args={:?}", sub_command_args);
		//for arg in sub_command_args {
		//	print!("{}", arg);
		//}
        //let cli = NgitCli::parse();
		use ngit::Cli;
        //let cli = ngit::Cli;
		let cli = ngit::Cli { command: val, nsec: val, password: val, disable_cli_spinners: val };
        let _ = match &cli.command {
            NgitCommands::Login(args) => ngit::sub_commands::login::launch(&cli, &args).await,
            NgitCommands::Init(args) => ngit::sub_commands::init::launch(&cli, &args).await,
            NgitCommands::Send(args) => ngit::sub_commands::send::launch(&cli, &args).await,
            NgitCommands::List => ngit::sub_commands::list::launch().await,
            NgitCommands::Pull => ngit::sub_commands::pull::launch().await,
            NgitCommands::Push(args) => ngit::sub_commands::push::launch(&cli, &args).await,
        };
    };

    Ok(())
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
