#![cfg_attr(not(test), warn(clippy::pedantic))]
#![cfg_attr(not(test), warn(clippy::expect_used))]

//use crate::sub_commands::chat::Utc;

//migrate carefully
use anyhow::Result;
use gnostr_chat::{chat as chat_impl, run as chat_run, ChatSubCommands as ChatCommandSubCommands};

use gnostr_chat::ChatSubCommands;

/// chat
///
/// # Errors
///
/// This function will return an error if the command fails.
pub async fn chat(sub_command_args: &ChatSubCommands) -> Result<(), anyhow::Error> {
    let args = ChatCommandSubCommands {
        nsec: sub_command_args.nsec.clone(),
        password: sub_command_args.password.clone(),
        name: sub_command_args.name.clone(),
        topic: sub_command_args.topic.clone(),
        hash: sub_command_args.hash.clone(),
        disable_cli_spinners: sub_command_args.disable_cli_spinners,
        info: sub_command_args.info,
        debug: sub_command_args.debug,
        trace: sub_command_args.trace,
        headless: sub_command_args.headless,
        workdir: sub_command_args.workdir.clone(),
        gitdir: sub_command_args.gitdir.clone(),
        oneshot: sub_command_args.oneshot.clone(),
    };
    chat_impl(&args).await?;
    Ok(())
}

/// run
///
/// # Panics
///
/// Panics if the tracing directive cannot be parsed.
///
/// # Errors
///
/// This function will return an error if the command fails.
pub async fn run(sub_command_args: &ChatSubCommands) -> Result<(), anyhow::Error> {
    let args = ChatCommandSubCommands {
        nsec: sub_command_args.nsec.clone(),
        password: sub_command_args.password.clone(),
        name: sub_command_args.name.clone(),
        topic: sub_command_args.topic.clone(),
        hash: sub_command_args.hash.clone(),
        disable_cli_spinners: sub_command_args.disable_cli_spinners,
        info: sub_command_args.info,
        debug: sub_command_args.debug,
        trace: sub_command_args.trace,
        headless: sub_command_args.headless,
        workdir: sub_command_args.workdir.clone(),
        gitdir: sub_command_args.gitdir.clone(),
        oneshot: sub_command_args.oneshot.clone(),
    };
    chat_run(&args).await?;
    Ok(())
}
