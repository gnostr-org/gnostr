use crate::api::{run_api_server, run_api_server_detached};
use crate::commands::{run_nip34, run_sniper, run_watch};
use crate::processor::BOOTSTRAP_RELAYS;
use crate::processor::Processor;
use crate::relay_manager::RelayManager;
use clap::{Parser, Subcommand};
use nostr_sdk::prelude::*;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Runs the sniper mode to find relays supporting a specific NIP
    Sniper {
        /// The NIP number to search for (e.g., 1)
        nip: i32,
        /// Optional: Path to a shitlist file to exclude relays
        #[arg(long, short)]
        shitlist: Option<String>,
    },
    /// Runs the watch mode to monitor relays and print their metadata
    Watch {
        /// Optional: Path to a shitlist file to exclude relays
        #[arg(long, short)]
        shitlist: Option<String>,
    },
    /// Lists relays that are likely to support NIP-34 (Git collaboration)
    Nip34 {
        /// Optional: Path to a shitlist file to exclude relays
        #[arg(long, short)]
        shitlist: Option<String>,
    },
    /// Runs the main gnostr-crawler logic
    Crawl(CliArgs),
    /// Starts a web server to serve relay information
    Serve {
        /// The port to listen on for the API server
        #[arg(long, short, default_value_t = 3000)]
        port: u16,
        /// Run the API server in the background.
        #[arg(long, default_value_t = false)]
        detach: bool,
    },
}

#[allow(clippy::manual_strip)]
#[derive(Parser, Debug, Clone)]
pub struct CliArgs {
    #[arg(long = "git-dir")]
    /// alternative git directory to use
    flag_git_dir: Option<String>,
    #[arg(long, short)]
    /// show commit diff
    flag_patch: bool,
    #[arg(
        value_name = "nsec",
        default_value = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    )]
    arg_nsec: Option<String>,
    #[arg(value_name = "commit")]
    arg_commit: Vec<String>,
    #[arg(value_name = "spec", last = true)]
    arg_spec: Vec<String>,
    #[arg(long)]
    arg_dump: bool,
}

pub async fn run(args: &CliArgs) -> Result<()> {
    let _run_async = async {
        let app_keys = Keys::parse(args.arg_nsec.clone().as_ref().expect("REASON")).unwrap();
        let relay_client = Client::new(app_keys);
        let _ = relay_client
            .send_event_builder(EventBuilder::text_note("#gnostr"))
            .await;
    };

    let app_keys = Keys::parse(args.arg_nsec.clone().as_ref().expect("REASON")).unwrap();
    let processor = Processor::new();
    let mut relay_manager = RelayManager::new(app_keys, processor).await;
    let bootstrap_relay_refs: Vec<&str> = BOOTSTRAP_RELAYS.iter().map(|s| s.as_str()).collect();
    let _run_async = relay_manager.run(bootstrap_relay_refs).await?;

    if args.arg_dump {
        relay_manager.processor.dump();
    }

    Ok(())
}

pub async fn dispatch_cli_command(
    cli: Cli,
    client: &reqwest::Client,
) -> Result<(), Box<dyn std::error::Error>> {
    match &cli.command {
        Commands::Sniper { nip, shitlist } => {
            run_sniper(*nip, shitlist.clone(), client).await?;
        }
        Commands::Watch { shitlist } => {
            run_watch(shitlist.clone(), client).await?;
        }
        Commands::Nip34 { shitlist } => {
            run_nip34(shitlist.clone(), client).await?;
        }
        Commands::Crawl(args) => {
            run(args).await?;
        }
        Commands::Serve { port, detach } => {
            if *detach {
                run_api_server_detached(&["serve"], *port)?;
            } else {
                run_api_server(*port).await?;
            }
        }
    }
    Ok(())
}
