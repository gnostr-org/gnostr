
use gnostr::sub_commands::crawler::{init_tracing, dispatch_crawler_command, CliArgs, CrawlerSubCommand};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing()?;

    let cli_subcommand = CrawlerSubCommand::parse();
    let client = reqwest::Client::new(); // Centralized client creation

    dispatch_crawler_command(cli_subcommand.command, &client).await?;

    Ok(())
}
