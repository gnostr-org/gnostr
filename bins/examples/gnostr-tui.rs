use std::env;

use anyhow::Result;
use clap::{Parser /* , Subcommand */};
use gnostr::{
    cli::{get_app_cache_path, GnostrCli, GnostrCommands},
    sub_commands,
};
use gnostr_asyncgit::sync::RepoPath;
use serde::ser::StdError;
use tracing::debug;
use tracing_core::metadata::LevelFilter;
use tracing_subscriber::prelude::*; // Import SubscriberExt
use tracing_subscriber::{fmt, util::SubscriberInitExt, EnvFilter, Registry};

#[tokio::main]
async fn main() -> Result<(), Box<dyn StdError>> {
    unsafe { env::set_var("WEEBLE", "0") };
    unsafe { env::set_var("BLOCKHEIGHT", "0") };
    unsafe { env::set_var("WOBBLE", "0") };
    let args: GnostrCli = GnostrCli::parse();

    let app_cache = get_app_cache_path();
    debug!("app_cache={:?}", app_cache);

    // Setup tracing subscriber once and globally
    let base_level = if args.debug {
        LevelFilter::DEBUG
    } else if args.trace {
        LevelFilter::TRACE
    } else if args.info {
        LevelFilter::INFO
    } else if args.warn {
        LevelFilter::WARN
    } else {
        LevelFilter::OFF
    };

    let filter = EnvFilter::builder()
        .with_default_directive(base_level.into())
        .from_env() // This reads RUST_LOG and builds the filter
        .expect("Failed to build EnvFilter from environment");

    let subscriber = Registry::default()
        .with(fmt::layer().with_writer(std::io::stderr)) // Direct all logs to stderr
        .with(filter);

    if let Err(e) = subscriber.try_init() {
        eprintln!("Failed to initialize tracing subscriber: {}", e);
    }

    let mut gitdir_value: Option<String> = None;
    let mut workdir_value: Option<String> = None;
    let env_args: Vec<String> = env::args().collect();
    for i in 0..env_args.len() {
        if env_args[i] == "--gitdir" {
            if i + 1 < env_args.len() {
                gitdir_value = Some(env_args[i + 1].clone());
            }
        }
        if env_args[i] == "--workdir" {
            if i + 1 < env_args.len() {
                workdir_value = Some(env_args[i + 1].clone());
            }
        }
    }

    // Post event
    match &args.command {
        //
        Some(GnostrCommands::Tui(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            let mut sub_command_args_mut = sub_command_args.clone();
            if let Some(dir) = gitdir_value.or(workdir_value) {
                sub_command_args_mut.gitdir =
                    Some(gnostr_asyncgit::sync::resolve_repo_path(&RepoPath::from(
                        dir.as_str(),
                    ))?);
            }
            sub_commands::tui::tui(sub_command_args_mut, &GnostrCli::default()).await
        }
        //
        None => {
            let mut gnostr_subcommands = gnostr::core::GnostrSubCommands::default();
            if let Some(dir) = gitdir_value.or(workdir_value) {
                gnostr_subcommands.gitdir =
                    Some(gnostr_asyncgit::sync::resolve_repo_path(&RepoPath::from(
                        dir.as_str(),
                    ))?);
            }
            sub_commands::tui::tui(gnostr_subcommands, &GnostrCli::default()).await
        }
        &Some(_) => todo!(),
    }
}
