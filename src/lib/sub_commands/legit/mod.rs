#![cfg_attr(not(test), warn(clippy::pedantic))]
#![cfg_attr(not(test), warn(clippy::expect_used))]
use ::time::OffsetDateTime;
use clap::Args;
use gnostr_legit::gitminer;
use serde::ser::StdError;

use crate::{
    cli::LegitCommands,
    legit::command,
    sub_commands::{fetch, init, list, login, pull, push, send},
    types::{Event, EventKind, Keys, Tag},
};

#[derive(Args, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct LegitSubCommand {
    #[command(subcommand)]
    command: Option<LegitCommands>,
    /// Path to your git repository
    repository_path: Option<String>,
    ///// nsec or hex private key
    #[arg(long, global = true)]
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
    #[arg(short, long, action = clap::ArgAction::Append)]
    message: Option<Vec<String>>,
    /// Nostr event kind to use for the git event
    #[arg(long, default_value = "1617")]
    kind: Option<u16>,
}

/// legit
///
/// # Panics
///
/// Panics if the local time offset cannot be determined.
///
/// # Errors
///
/// This function will return an error if the command fails.
pub async fn legit(sub_command_args: &LegitSubCommand) -> Result<(), Box<dyn StdError>> {
    match &sub_command_args.command {
        Some(LegitCommands::Login(args)) => login::launch(args).await?,
        Some(LegitCommands::Init(args)) => init::launch(args).await?,
        Some(LegitCommands::Send(args)) => send::launch(args, true).await?,
        Some(LegitCommands::List) => list::launch().await?,
        Some(LegitCommands::Pull) => pull::launch().await?,
        Some(LegitCommands::Push(args)) => push::launch(args).await?,
        Some(LegitCommands::Fetch(args)) => fetch::launch(args).await?,
        Some(LegitCommands::Mine) | None => {
            #[allow(clippy::cast_possible_truncation)]
            let opts = gitminer::Options {
                threads: sub_command_args.threads as u32,
                target: sub_command_args
                    .pow
                    .clone()
                    .unwrap_or(sub_command_args.prefix.clone().unwrap_or_default()),
                message: sub_command_args.message.clone().unwrap_or_default(),
                repo: sub_command_args.repo.clone().unwrap_or(
                    sub_command_args
                        .repository_path
                        .clone()
                        .unwrap_or(".".to_string()),
                ),

                timestamp: OffsetDateTime::now_local().unwrap(),
                kind: sub_command_args.kind,
            };
            command::run_legit_command(opts)
                .await
                .map_err(|e| Box::new(e) as Box<dyn StdError>)?;
        }
    }
    Ok(())
}
