pub mod cli;
pub mod cli_interactor;
pub mod client;
pub mod git;
pub mod git_events;
pub mod repo_ref;
pub mod repo_state;
pub mod remote;
pub mod push;
pub mod utils;

pub mod sub_commands;
use crate::sub_commands::{init, send, login};

use crate::login::login::SubCommandArgs;

use anyhow::{Result, anyhow};
use directories::ProjectDirs;


//#[cfg_attr(not(test), warn(clippy::pedantic))]
//#[cfg_attr(not(test), warn(clippy::expect_used))]

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
    /// nsec or hex private key
    #[arg(short, long, global = true)]
    nsec: Option<String>,
    /// password to decrypt nsec
    #[arg(short, long, global = true)]
    password: Option<String>,
    /// disable spinner animationss
    #[arg(long, action)]
    disable_cli_spinners: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// signal you are this repo's maintainer accepting proposals via nostr
    Init(init::SubCommandArgs),
    /// issue commits as a proposal
    Send(send::SubCommandArgs),
    /// list proposals; checkout, apply or download selected
    List,
    /// send proposal revision
    //Push(SubCommandArgs),
    /// fetch and apply new proposal commits / revisions linked to branch
    Pull,
    /// run with --nsec flag to change npub
    Login(login::login::SubCommandArgs),
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


pub fn get_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from("", "", "ngit").ok_or(anyhow!(
        "should find operating system home directories with rust-directories crate"
    ))
}
