use clap::Parser;
use gnostr::crawler::{run_nip34, run_sniper, run_watch, Cli, Commands};

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
    }

    Ok(())
}
