#![cfg_attr(not(test), warn(clippy::pedantic))]
#![cfg_attr(not(test), warn(clippy::expect_used))]
#![cfg_attr(not(test), warn(clippy::pedantic))]
#![cfg_attr(not(test), warn(clippy::expect_used))]
use crate::cli::LegitCommands;
#[cfg(not(test))]
use crate::sub_commands::fetch;
#[cfg(not(test))]
use crate::sub_commands::init;
#[cfg(not(test))]
use crate::sub_commands::list;
#[cfg(not(test))]
use crate::sub_commands::login;
#[cfg(not(test))]
use crate::sub_commands::pull;
#[cfg(not(test))]
use crate::sub_commands::push;
#[cfg(not(test))]
use crate::sub_commands::send;
use clap::Args;
use nostr_sdk_0_34_0::prelude::*;
use std::time::SystemTime;

#[cfg(test)]
use crate::sub_commands::legit_mocks::fetch;
#[cfg(test)]
use crate::sub_commands::legit_mocks::init;
#[cfg(test)]
use crate::sub_commands::legit_mocks::list;
#[cfg(test)]
use crate::sub_commands::legit_mocks::login;
#[cfg(test)]
use crate::sub_commands::legit_mocks::pull;
#[cfg(test)]
use crate::sub_commands::legit_mocks::push;
#[cfg(test)]
use crate::sub_commands::legit_mocks::send;


use serde::ser::StdError;
#[cfg(not(test))]
use gnostr_legit::command;
#[cfg(test)]
use crate::sub_commands::legit_mocks::gnostr_legit;


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
    #[arg(long, default_value = "000")]
    prefix: Option<String>,
    /// Number of worker threads to use
    #[arg(short, long, default_value_t = 1)]
    threads: usize,
    /// Commit message to use
    #[arg(short, long, default_value = "gnostr-legit commit")]
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
            let opts = gnostr_legit::gitminer::Options {
                threads: sub_command_args.threads as u32,
                target: sub_command_args.prefix.clone().unwrap_or_default(),
                message: if sub_command_args.message.is_empty() {
                    "gnostr-legit-test".to_string()
                } else {
                    sub_command_args.message.clone()
                },
                repo: sub_command_args.repository_path.clone().unwrap_or(".".to_string()),
                timestamp: SystemTime::now(),
            };
            gnostr_legit::command::run_legit_command(opts).map_err(|e| Box::new(e) as Box<dyn StdError>)?;
        }
    }
    Ok(())
}
