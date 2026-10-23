use clap::Parser;
use gnostr::sub_commands::sniper::{SniperArgs, run_sniper};

#[derive(Parser, Debug)]
#[command(author, version, about = "gnostr: a git+nostr workflow utility", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser, Debug)]
enum Commands {
    /// Perform actions related to sniping relays
    Sniper(SniperArgs),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Sniper(args) => {
            run_sniper(args).await?;
        }
    }

    Ok(())
}
