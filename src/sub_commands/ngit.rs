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
        //println!("ngit:else:command={:?}", command);
        let cli = ngit::Cli {
            command: command.command,
            nsec: Some(String::from("")),
            password: Some(String::from("")),
            disable_cli_spinners: true,
        };
        //let _ = match &cli.command {
		if sub_command_args.login {
            login::launch(&cli, &args).await;
		} else
		if sub_command_args.init {
            //let args = Commands::Init(args);
            init::launch(&cli, &args).await;
		} else
		if sub_command_args.send {
            //let args = Commands::Send(args);
            send::launch(&cli, &args).await;
		} else
		if sub_command_args.list {
            //let args = Commands::List(args);
            list::launch().await;
		} else
		if sub_command_args.pull {
            //let args = Commands::Pull(args);
            //sub_commands::pull::launch().await;
            pull::launch().await;
		} else
		if sub_command_args.push {
            //let args = Commands::Push(args);
            push::launch(&cli, &args).await;
            //Commands::Push(args) => sub_commands::push::launch(&cli, args).await,
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
