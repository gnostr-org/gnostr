use clap::Parser;
use gnostr_crawler::{run_nip34, run_sniper, run_watch, Cli, Commands};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    //tracing_subscriber::fmt()
    //    .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
    //    .init();

    let cli = Cli::parse();
    let client = reqwest::Client::new();

    match &cli.command {
        Commands::Sniper { nip, shitlist } => {
            run_sniper(*nip, shitlist.clone(), &client).await?;
        }
        Commands::Watch { shitlist } => {
            run_watch(shitlist.clone(), &client).await?;
        }
        Commands::Nip34 { shitlist } => {
            run_nip34(shitlist.clone(), &client).await?;
        }
        Commands::Crawl(args) => {
            gnostr_crawler::run(&args).await?;
        }
        Commands::Serve { port, detach } => {
            if *detach {
                gnostr_crawler::run_api_server_detached(&["serve"], *port)?;
            } else {
                gnostr_crawler::run_api_server(*port).await?;
            }
        }
    }

    Ok(())
}
