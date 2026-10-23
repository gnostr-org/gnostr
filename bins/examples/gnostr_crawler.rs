use clap::Parser;
use gnostr_crawler::{Cli, dispatch_cli_command, init_tracing};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing()?;

    let cli = Cli::parse();
    let client = gnostr_crawler::api::build_http_client(); // Centralized client creation

    dispatch_cli_command(cli, &client).await?;

    Ok(())
}
