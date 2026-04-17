extern crate self as ngit;

pub mod accept_maintainership;
#[path = "../bin/ngit/cli.rs"]
pub mod cli;
pub mod cli_interactor;
pub mod client;
pub mod fetch;
pub mod git;
pub mod git_events;
pub mod list;
pub mod login;
pub mod mbox_parser;
pub mod push;
pub mod repo_ref;
pub mod repo_state;
#[path = "../bin/ngit/sub_commands/mod.rs"]
pub mod sub_commands;
// TEMPORARY: Remove when async-wsocket includes Happy Eyeballs support.
// See src/lib/transport.rs header for full removal instructions.
pub mod transport;
pub mod utils;

use anyhow::{Result, anyhow};
use clap::CommandFactory;
use cli::{AccountCommands, CUSTOMISE_TEMPLATE, Cli, Commands};
use directories::ProjectDirs;
use nostr_sdk::Url;

pub fn get_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from("", "", "ngit").ok_or(anyhow!(
        "should find operating system home directories with rust-directories crate"
    ))
}

pub trait UrlWithoutSlash {
    fn as_str_without_trailing_slash(&self) -> &str;
    fn to_string_without_trailing_slash(&self) -> String;
}

impl UrlWithoutSlash for Url {
    fn as_str_without_trailing_slash(&self) -> &str {
        let url_str = self.as_str();
        if let Some(without) = url_str.strip_suffix('/') {
            without
        } else {
            url_str
        }
    }

    fn to_string_without_trailing_slash(&self) -> String {
        self.as_str_without_trailing_slash().to_string()
    }
}

pub fn install_rustls_crypto_provider() {
    let _ = rustls::crypto::ring::default_provider().install_default();
}

pub async fn run_cli(cli: &Cli) -> Result<()> {
    install_rustls_crypto_provider();

    if cli.interactive {
        std::env::set_var("NGIT_INTERACTIVE_MODE", "1");
    }

    if cli.verbose || std::env::var("NGITTEST").is_ok() {
        std::env::set_var("NGIT_VERBOSE", "1");
    }

    if cli.customize {
        print!("{CUSTOMISE_TEMPLATE}");
        return Ok(());
    }

    let _ = git::utils::set_git_timeout();

    if let Some(command) = &cli.command {
        match command {
            Commands::Account(args) => match &args.account_command {
                AccountCommands::Login(sub_args) => {
                    sub_commands::login::launch(cli, sub_args).await
                }
                AccountCommands::Logout => sub_commands::logout::launch().await,
                AccountCommands::ExportKeys => sub_commands::export_keys::launch().await,
                AccountCommands::Create(sub_args) => {
                    sub_commands::create::launch(cli, sub_args).await
                }
            },
            Commands::Init(args) => sub_commands::init::launch(cli, args).await,
            Commands::Repo(args) => {
                sub_commands::repo::launch(cli, args.repo_command.as_ref(), args.offline).await
            }
            Commands::List {
                status,
                json,
                id,
                offline,
            } => sub_commands::list::launch(status.clone(), *json, id.clone(), *offline).await,
            Commands::Send(args) => sub_commands::send::launch(cli, args, false).await,
            Commands::Sync(args) => sub_commands::sync::launch(args).await,
            Commands::Checkout { id, offline } => {
                sub_commands::checkout::launch(id, *offline).await
            }
            Commands::Apply {
                id,
                stdout,
                offline,
            } => sub_commands::apply::launch(id, *stdout, *offline).await,
        }
    } else {
        let mut cmd = Cli::command();
        cmd.print_help()?;
        println!();
        Ok(())
    }
}
