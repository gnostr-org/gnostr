#![cfg_attr(not(test), warn(clippy::pedantic))]
#![cfg_attr(not(test), warn(clippy::expect_used))]
use clap::Args;
use serde::ser::StdError;

use crate::cli::NgitCommands;

#[derive(Args)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct NgitSubCommand {
    #[command(subcommand)]
    pub command: Option<NgitCommands>,
    /// remote signer address
    #[arg(long, global = true, hide = true)]
    pub bunker_uri: Option<String>,
    /// remote signer app secret key
    #[arg(long, global = true, hide = true)]
    pub bunker_app_key: Option<String>,
    /// nsec or hex private key
    #[arg(short, long, global = true)]
    pub nsec: Option<String>,
    /// password to decrypt nsec
    #[arg(short, long, global = true, hide = true)]
    pub password: Option<String>,
    /// disable spinner animations
    #[arg(long, action = clap::ArgAction::SetTrue, hide = true)]
    pub disable_cli_spinners: bool,
    /// show customization options via git config
    #[arg(short, long, global = true)]
    pub customize: bool,
    /// Use default values without prompting
    #[arg(short = 'd', long, global = true, conflicts_with = "interactive")]
    pub defaults: bool,
    /// Enable interactive prompts
    #[arg(short = 'i', long, global = true)]
    pub interactive: bool,
    /// Force operations, bypass safety guards
    #[arg(short = 'f', long, global = true)]
    pub force: bool,
    /// Enable verbose output
    #[arg(short = 'v', long, global = true)]
    pub verbose: bool,
}

/// ngit
///
/// # Errors
///
/// This function will return an error if the command fails.
pub async fn ngit(sub_command_args: NgitSubCommand) -> Result<(), Box<dyn StdError>> {
    let cli = ::ngit::cli::Cli {
        command: sub_command_args.command,
        bunker_uri: sub_command_args.bunker_uri,
        bunker_app_key: sub_command_args.bunker_app_key,
        nsec: sub_command_args.nsec,
        password: sub_command_args.password,
        disable_cli_spinners: sub_command_args.disable_cli_spinners,
        customize: sub_command_args.customize,
        defaults: sub_command_args.defaults,
        interactive: sub_command_args.interactive,
        force: sub_command_args.force,
        verbose: sub_command_args.verbose,
    };
    ::ngit::run_cli(&cli).await?;
    Ok(())
}
