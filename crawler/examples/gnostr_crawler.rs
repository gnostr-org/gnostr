use clap::Parser;
use gnostr_crawler::{Cli, init_tracing, dispatch_cli_command};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing()?;

    let cli = Cli::parse();
    let client = reqwest::Client::new(); // Centralized client creation

    dispatch_cli_command(cli, &client).await?;

    Ok(())
}
