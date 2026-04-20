use clap::Parser;
use gnostr_crawler::{dispatch_cli_command, Cli};

#[derive(Parser, Debug)]
struct GnostrCrawlerArgs {
    /// Logging level (error, warn, info, debug, trace, off)
    #[clap(long, default_value = "info")]
    logging: String,

    #[clap(flatten)]
    cli: Cli,
}

fn init_tracing(logging: &str) -> Result<(), Box<dyn std::error::Error>> {
    let filter = tracing_subscriber::EnvFilter::try_new(format!(
        "{logging},nostr_sdk::relay=off,nostr_relay_pool=off,nostr_relay_pool::relay::inner=off"
    ))?;

    tracing_subscriber::fmt().with_env_filter(filter).init();
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = GnostrCrawlerArgs::parse();
    init_tracing(&args.logging)?;

    let client = reqwest::Client::new(); // Centralized client creation

    dispatch_cli_command(args.cli, &client).await?;

    Ok(())
}
