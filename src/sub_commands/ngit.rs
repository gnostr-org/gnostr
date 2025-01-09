#![cfg_attr(not(test), warn(clippy::pedantic))]
#![cfg_attr(not(test), warn(clippy::expect_used))]

//use ngit::*;
//use anyhow::Result;
//use ngit::sub_commands;
//use ngit::sub_commands::*;
use ngit::sub_commands::init;
use ngit::sub_commands::list;
use ngit::sub_commands::login;
use ngit::sub_commands::pull;
use ngit::sub_commands::push;
use ngit::sub_commands::send;

use ngit::sub_commands::init::InitSubCommandArgs;
use ngit::sub_commands::login::LoginSubCommandArgs;
use ngit::sub_commands::push::PushSubCommandArgs;
use ngit::sub_commands::send::SendSubCommandArgs;

use ngit::Cli as NgitCli;
use nostr_sdk::prelude::*;

use clap::{Args, Parser, Subcommand};

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
		use std::env;
		let args: Vec<String> = env::args().collect();
        println!("ngit:else:args={:?}", args);
        println!("ngit:else:sub_command_args={:?}", sub_command_args);
        let command = NgitCli::parse();
        let cli = ngit::Cli {
            command: command.command,
            nsec: Some(String::from("")),
            password: Some(String::from("")),
            disable_cli_spinners: true,
        };
		if sub_command_args.login {
/// run with --nsec flag to change npub
/// 
/// Usage: ngit login [OPTIONS]
/// 
/// Options:
///       --offline              don't fetch user metadata and relay list from relays
///   -n, --nsec <NSEC>          nsec or hex private key
///   -p, --password <PASSWORD>  password to decrypt nsec
///   -h, --help                 Print help
///   -V, --version              Print version
/// 
			let args = ngit::sub_commands::login::LoginSubCommandArgs { offline: false };
            login::launch(&cli, &args).await;
		} else
		if sub_command_args.init {


/// Usage: ngit init [OPTIONS]
/// 
/// Options:
///   -t, --title <TITLE>
///           name of repository
///   -d, --description <DESCRIPTION>
///           optional description
///       --clone-url <CLONE_URL>
///           git server url users can clone from
///   -w, --web <WEB>...
///           homepage
///   -r, --relays <RELAYS>...
///           relays contributors push patches and comments to
///   -o, --other-maintainers <OTHER_MAINTAINERS>...
///           npubs of other maintainers
///       --earliest-unique-commit <EARLIEST_UNIQUE_COMMIT>
///           usually root commit but will be more recent commit for forks
///   -n, --nsec <NSEC>
///           nsec or hex private key
///   -i, --identifier <IDENTIFIER>
///           shortname with no spaces or special characters
///   -p, --password <PASSWORD>
///           password to decrypt nsec
///   -h, --help
///           Print help
///   -V, --version
///           Print version


			let args = ngit::sub_commands::init::InitSubCommandArgs { title: Some(String::from("")), description:Some(String::from("")), clone_url: vec![String::from("")], earliest_unique_commit: Some(String::from("")), identifier: Some(String::from("")), other_maintainers: vec![String::from("")], relays: vec![String::from("")], web: vec![String::from("")] };
            init::launch(&cli, &args).await;
		} else
		if sub_command_args.send {
			let args = ngit::sub_commands::send::SendSubCommandArgs { in_reply_to: vec![String::from("")], no_cover_letter: false, description: Some(String::from("")), since_or_range: String::from(""), title: Some(String::from("")) };
            send::launch(&cli, &args).await;
		} else
		if sub_command_args.list {
            list::launch().await;
		} else
		if sub_command_args.pull {
            pull::launch().await;
		} else
		if sub_command_args.push {
			let args = ngit::sub_commands::push::PushSubCommandArgs { force: true, no_cover_letter: true };
            push::launch(&cli, &args).await;
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
