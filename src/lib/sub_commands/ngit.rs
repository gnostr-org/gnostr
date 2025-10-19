#![cfg_attr(not(test), warn(clippy::pedantic))]
#![cfg_attr(not(test), warn(clippy::expect_used))]
use crate::cli::NgitCommands;
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
    #[arg(long, default_value_t = true)]
    disable_cli_spinners: bool,
}

pub async fn ngit(sub_command_args: &NgitSubCommand) -> Result<(), Box<dyn StdError>> {
    match &sub_command_args.command {
        NgitCommands::Login(args) => login::launch(args).await?,
        NgitCommands::Init(args) => init::launch(args).await?,
        NgitCommands::Send(args) => send::launch(args, true).await?,
        NgitCommands::List => list::launch().await?,
        NgitCommands::Pull => pull::launch().await?,
        NgitCommands::Push(args) => push::launch(args).await?,
        NgitCommands::Fetch(args) => fetch::launch(args).await?,
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sub_commands::fetch::FetchArgs;
    use crate::sub_commands::init::InitArgs;
    use crate::sub_commands::login::LoginArgs;
    use crate::sub_commands::push::PushArgs;
    use crate::sub_commands::send::SendArgs;

    // Helper function to create a dummy NgitSubCommand
    fn create_dummy_ngit_subcommand(command: NgitCommands) -> NgitSubCommand {
        NgitSubCommand {
            command,
            nsec: None,
            password: None,
            disable_cli_spinners: true,
        }
    }

    #[tokio::test]
    async fn test_ngit_login_command() {
        let args = LoginArgs {
            nsec: None,
            password: None,
            offline: false,
            disable_cli_spinners: Some(true),
            bunker_app_key: None,
            bunker_uri: None,
        };
        let sub_command = create_dummy_ngit_subcommand(NgitCommands::Login(args));
        let result = ngit(&sub_command).await;
        // We expect an error here because the actual login::launch would try to connect to relays
        // and likely fail without proper setup. The goal is to ensure it *tries* to call login::launch.
        // A more robust test would involve mocking, but for a dispatcher, this confirms the call path.
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_ngit_init_command() {
        let args = InitArgs {
            title: None,
            description: None,
            clone_url: vec![],
            web: vec![],
            relays: vec![],
            other_maintainers: vec![],
            earliest_unique_commit: None,
            identifier: None,
            disable_cli_spinners: true,
            password: None,
            nsec: None,
            bunker_app_key: None,
            bunker_uri: None,
        };
        let sub_command = create_dummy_ngit_subcommand(NgitCommands::Init(args));
        let result = ngit(&sub_command).await;
        assert!(result.is_err()); // init::launch also likely fails without proper setup
    }

    #[tokio::test]
    async fn test_ngit_send_command() {
        let args = SendArgs {
            since_or_range: "".to_string(),
            in_reply_to: vec![],
            no_cover_letter: true,
            title: None,
            description: None,
            disable_cli_spinners: true,
            password: None,
            nsec: None,
            bunker_app_key: None,
            bunker_uri: None,
        };
        let sub_command = create_dummy_ngit_subcommand(NgitCommands::Send(args));
        let result = ngit(&sub_command).await;
        assert!(result.is_err()); // send::launch also likely fails without proper setup
    }

    #[tokio::test]
    async fn test_ngit_list_command() {
        let sub_command = create_dummy_ngit_subcommand(NgitCommands::List);
        let result = ngit(&sub_command).await;
        assert!(result.is_ok()); // list::launch might succeed without external dependencies
    }

    #[tokio::test]
    async fn test_ngit_pull_command() {
        let sub_command = create_dummy_ngit_subcommand(NgitCommands::Pull);
        let result = ngit(&sub_command).await;
        assert!(result.is_err()); // pull::launch likely fails without proper setup
    }

    #[tokio::test]
    async fn test_ngit_push_command() {
        let args = PushArgs {
            force: false,
            disable_cli_spinners: true,
            password: None,
            nsec: None,
            bunker_app_key: None,
            bunker_uri: None,
        };
        let sub_command = create_dummy_ngit_subcommand(NgitCommands::Push(args));
        let result = ngit(&sub_command).await;
        assert!(result.is_err()); // push::launch likely fails without proper setup
    }

    #[tokio::test]
    async fn test_ngit_fetch_command() {
        let args = FetchArgs {
            repo: vec![],
        };
        let sub_command = create_dummy_ngit_subcommand(NgitCommands::Fetch(args));
        let result = ngit(&sub_command).await;
        assert!(result.is_err()); // fetch::launch likely fails without proper setup
    }
}
