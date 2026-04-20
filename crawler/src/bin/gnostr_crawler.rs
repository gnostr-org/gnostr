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
    let hyper_directives = if matches!(logging, "debug" | "trace") {
        "hyper::client::trace=trace,hyper::client::connect=trace"
    } else {
        "hyper::client::trace=off,hyper::client::connect=off"
    };
    let filter = tracing_subscriber::EnvFilter::try_new(format!(
        "{logging},{hyper_directives},hyper::client::connect::http=off,hyper::client::connect::dns=off,hyper::proto=off,nostr_sdk::relay=off,nostr_relay_pool=off,nostr_relay_pool::relay::inner=off"
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
