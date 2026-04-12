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
    env::set_var("GNOSTR_GITDIR", "");
    env::set_var("WEEBLE", "0");
    env::set_var("BLOCKHEIGHT", "0");
    env::set_var("WOBBLE", "0");
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

    if args.gitdir.is_some() {
        // Assuming 'args' and 'gitdir' are correctly defined elsewhere
        let repo_path: RepoPath = args.gitdir.clone().expect("");
        debug!("main:50:repo_path={:?}", repo_path);
        // Convert the RepoPath to an OsStr reference
        let path_os_str = repo_path.as_path().as_os_str();

        // Now set the environment variable
        env::set_var("GNOSTR_GITDIR", path_os_str);

        debug!("main:57:{:?}", args.gitdir.clone().expect(""));
        //env::set_var("GNOSTR_GITDIR", args.gitdir.clone().expect(""));
        debug!("59:{}", env::var("GNOSTR_GITDIR").unwrap().to_string());
        //replace gnostr tui --gitdir
        //std::process::exit(0);
    }
    let _ = args.workdir.is_some();
    let _ = args.directory.is_some();

    // Post event
    match &args.command {
        //
        Some(GnostrCommands::Tui(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::tui::tui(sub_command_args.clone(), &GnostrCli::default()).await
        }
        //
        None => {
            {
                let gnostr_subcommands = gnostr::core::GnostrSubCommands::default();
                let _ = sub_commands::tui::tui(gnostr_subcommands, &GnostrCli::default()).await;
            };
            Ok(())
        }
        &Some(_) => todo!(),
    }
}
