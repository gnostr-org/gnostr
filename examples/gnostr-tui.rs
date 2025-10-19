use anyhow::Result;
use clap::{Parser /*, Subcommand*/};
use gnostr::cli::*;
use gnostr::cli::{get_app_cache_path, setup_logging, GnostrCli, GnostrCommands};
use gnostr::sub_commands;
use gnostr_asyncgit::sync::RepoPath;
use sha2::{Digest, Sha256};
use std::env;
use tracing::{debug, trace};
use tracing_core::metadata::LevelFilter;
use tracing_subscriber::FmtSubscriber;

use serde::ser::StdError;

#[tokio::main]
async fn main() -> Result<(), Box<dyn StdError>> {
    env::set_var("WEEBLE", "0");
    env::set_var("BLOCKHEIGHT", "0");
    env::set_var("WOBBLE", "0");
    let mut args: GnostrCli = GnostrCli::parse();

    let app_cache = get_app_cache_path();
    if args.logging {
        let logging = setup_logging();
        trace!("{:?}", logging);
    };
    let level = if args.debug {
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
    let env_args: Vec<String> = env::args().collect();
    for arg in &env_args {
        debug!("24:arg={:?}", arg);
    }

    let subscriber = FmtSubscriber::builder().with_max_level(level).finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    trace!("{:?}", app_cache);

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
    if args.workdir.is_some() {}
    if args.directory.is_some() {}

    // Post event
    match &args.command {
        //
        Some(GnostrCommands::Tui(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::tui::tui(sub_command_args.clone()).await
        }
        //
        None => {
            {
                let gnostr_subcommands = gnostr::gnostr_core::GnostrSubCommands::default();
                let _ = sub_commands::tui::tui(gnostr_subcommands).await;
            };
            Ok(())
        }
        &Some(_) => todo!(),
    }
}
