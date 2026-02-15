use clap::Parser;
use gnostr_crawler::{Cli, Commands, run_sniper, run_watch, run_nip34, run_api_server};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env()
        // .add_directive("nostr_sdk::relay=off".parse()?)
        // .add_directive("hyper=off".parse()?)

        /**/)/**/
        .init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Sniper { nip, shitlist } => {
            run_sniper(*nip, shitlist.clone()).await?;
        }
        Commands::Watch { shitlist } => {
            run_watch(shitlist.clone()).await?;
        }
        Commands::Nip34 { shitlist } => {
            run_nip34(shitlist.clone()).await?;
        }
        Commands::Crawl(args) => {
            gnostr_crawler::run(args).await?;
        }
        Commands::Serve { port } => {
            run_api_server(*port).await?;
        }
    }

    Ok(())
}
