use crate::cli::NgitCommands;
use crate::sub_commands::ngit::{ngit, NgitSubCommand};
use clap::Parser;

// Import all SubCommandArgs from the sub_commands module
use crate::sub_commands::fetch::SubCommandArgs as FetchSubCommandArgs;
use crate::sub_commands::init::SubCommandArgs as InitSubCommandArgs;
use crate::sub_commands::login::SubCommandArgs as LoginSubCommandArgs;
use crate::sub_commands::push::SubCommandArgs as PushSubCommandArgs;
use crate::sub_commands::send::SubCommandArgs as SendSubCommandArgs;

#[tokio::test]
async fn test_ngit_login_dispatch() {
    let sub_command_args = NgitSubCommand {
        command: NgitCommands::Login(LoginSubCommandArgs {
            offline: false,
            disable_cli_spinners: None,
            password: None,
            nsec: None,
            bunker_app_key: None,
            bunker_uri: None,
        }),
        nsec: None,
        password: None,
        disable_cli_spinners: false,
    };
    let result = ngit(&sub_command_args).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_ngit_init_dispatch() {
    let sub_command_args = NgitSubCommand {
        command: NgitCommands::Init(InitSubCommandArgs { maintainer: false,
            title: None,
            description: None,
            clone_url: vec![],
            web: vec![],
            relays: vec![],
            other_maintainers: vec![],
            earliest_unique_commit: None,
            identifier: None,
            disable_cli_spinners: false,
            password: None,
            nsec: None,
            bunker_app_key: None,
            bunker_uri: None,
        }),
        nsec: None,
        password: None,
        disable_cli_spinners: false,
    };
    let result = ngit(&sub_command_args).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_ngit_send_dispatch() {
    let sub_command_args = NgitSubCommand {
        command: NgitCommands::Send(SendSubCommandArgs {
            since_or_range: String::new(),
            in_reply_to: vec![],
            no_cover_letter: false,
            title: None,
            description: None,
            disable_cli_spinners: false,
            password: None,
            nsec: None,
            bunker_app_key: None,
            bunker_uri: None,
        }),
        nsec: None,
        password: None,
        disable_cli_spinners: false,
    };
    let result = ngit(&sub_command_args).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_ngit_list_dispatch() {
    let sub_command_args = NgitSubCommand {
        command: NgitCommands::List,
        nsec: None,
        password: None,
        disable_cli_spinners: false,
    };
    let result = ngit(&sub_command_args).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_ngit_pull_dispatch() {
    let sub_command_args = NgitSubCommand {
        command: NgitCommands::Pull,
        nsec: None,
        password: None,
        disable_cli_spinners: false,
    };
    let result = ngit(&sub_command_args).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_ngit_push_dispatch() {
    let sub_command_args = NgitSubCommand {
        command: NgitCommands::Push(PushSubCommandArgs {
            force: false,
            disable_cli_spinners: false,
            password: None,
            nsec: None,
            bunker_app_key: None,
            bunker_uri: None,
        }),
        nsec: None,
        password: None,
        disable_cli_spinners: false,
    };
    let result = ngit(&sub_command_args).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_ngit_fetch_dispatch() {
    let sub_command_args = NgitSubCommand {
        command: NgitCommands::Fetch(FetchSubCommandArgs { repo: vec![] }),
        nsec: None,
        password: None,
        disable_cli_spinners: false,
    };
    let result = ngit(&sub_command_args).await;
    assert!(result.is_ok());
}

