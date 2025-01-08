#![cfg_attr(not(test), warn(clippy::pedantic))]
#![cfg_attr(not(test), warn(clippy::expect_used))]

use anyhow::Result;
use clap::{Parser, Subcommand};

pub mod cli_interactor;
pub mod client;
pub mod config;
pub mod git;
pub mod key_handling;
pub mod login;
pub mod repo_ref;
pub mod sub_commands;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    /// nsec or hex private key
    #[arg(short, long, global = true)]
    pub nsec: Option<String>,
    /// password to decrypt nsec
    #[arg(short, long, global = true)]
    pub password: Option<String>,
    /// disable spinner animations
    #[arg(long, action)]
    pub disable_cli_spinners: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// signal you are this repo's maintainer accepting proposals via nostr
    Init(sub_commands::init::SubCommandArgs),
    /// issue commits as a proposal
    Send(sub_commands::send::SubCommandArgs),
    /// list proposals; checkout, apply or download selected
    List,
    /// send proposal revision
    Push(sub_commands::push::SubCommandArgs),
    /// fetch and apply new proposal commits / revisions linked to branch
    Pull,
    /// run with --nsec flag to change npub
    Login(sub_commands::login::SubCommandArgs),
}

// #[tokio::main]
// async fn main() -> Result<()> {
//     let cli = Cli::parse();
//     match &cli.command {
//         Commands::Login(args) => sub_commands::login::launch(&cli, args).await,
//         Commands::Init(args) => sub_commands::init::launch(&cli, args).await,
//         Commands::Send(args) => sub_commands::send::launch(&cli, args).await,
//         Commands::List => sub_commands::list::launch().await,
//         Commands::Pull => sub_commands::pull::launch().await,
//         Commands::Push(args) => sub_commands::push::launch(&cli, args).await,
//     }
// }
