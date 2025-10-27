#![cfg_attr(not(test), warn(clippy::pedantic))]
#![cfg_attr(not(test), warn(clippy::expect_used))]
use crate::cli::LegitCommands;
use crate::sub_commands::fetch;
use crate::sub_commands::init;
use crate::sub_commands::list;
use crate::sub_commands::login;
use crate::sub_commands::pull;
use crate::sub_commands::push;
use crate::sub_commands::send;
use clap::Args;
use nostr_sdk_0_34_0::prelude::*;

use serde::ser::StdError;
use gnostr_legit::command;
use gnostr_legit::gitminer;
use time::{get_time, now};

#[derive(Args, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct LegitSubCommand {
    #[command(subcommand)]
    command: Option<LegitCommands>,
    /// Path to your git repository
    repository_path: Option<String>,
    ///// nsec or hex private key
    #[arg(short, long, global = true)]
    nsec: Option<String>,
    ///// password to decrypt nsec
    #[arg(short, long, global = true)]
    password: Option<String>,
    ///// nsec or hex private key
    #[arg(short, long, global = true)]
    repo: Option<String>,
    ///// password to decrypt nsec
    #[arg(long, global = true)]
    pow: Option<String>,
    /// Desired commit prefix
    #[arg(short, long)]
    prefix: String,
    /// Number of worker threads to use
    #[arg(short, long)]
    threads: Option<usize>,
    /// Commit message to use
    #[arg(short, long)]
    message: String,
    ///// disable spinner animations
    #[arg(long, action)]
    disable_cli_spinners: bool,
}

pub async fn legit(sub_command_args: &LegitSubCommand) -> Result<(), Box<dyn StdError>> {
    match &sub_command_args.command {
        Some(LegitCommands::Login(args)) => login::launch(&args).await?,
        Some(LegitCommands::Init(args)) => init::launch(&args).await?,
        Some(LegitCommands::Send(args)) => send::launch(&args, true).await?,
        Some(LegitCommands::List) => list::launch().await?,
        Some(LegitCommands::Pull) => pull::launch().await?,
        Some(LegitCommands::Push(args)) => push::launch(&args).await?,
        Some(LegitCommands::Fetch(args)) => fetch::launch(&args).await?,
        Some(LegitCommands::Mine) | None => {
            let repo_path = sub_command_args.repository_path.clone().unwrap_or(".".to_string());
            let prefix = sub_command_args.prefix.clone();
            let threads = sub_command_args.threads.unwrap_or(8) as u32;
            let message = sub_command_args.message.clone();

            let opts = gitminer::Options {
                threads,
                target: prefix,
                message,
                repo: repo_path,
                timestamp: now(),
            };
            command::run_legit(opts).map_err(|e| Box::new(e) as Box<dyn StdError>)?;
        }
    }
    Ok(())
}
