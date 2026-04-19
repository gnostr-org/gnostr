use clap::Parser;
use gnostr::crawler::{run_nip34, run_sniper, run_watch, Cli, Commands};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    //tracing_subscriber::fmt()
    //    .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
    //    .init();

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
    }

    Ok(())
}
