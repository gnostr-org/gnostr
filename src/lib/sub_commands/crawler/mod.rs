use crate::crawler;

/// Compatibility alias for the crawler subcommand parser.
pub type CrawlerSubCommand = crawler::Cli;

/// Compatibility alias for the inner crawler command set.
pub type InnerCrawlerCommand = crawler::Commands;

/// Dispatches crawler subcommands through the shared crawler implementation.
pub async fn dispatch_crawler_command(
    command: CrawlerSubCommand,
    client: &reqwest::Client,
) -> Result<(), Box<dyn std::error::Error>> {
    crawler::dispatch_cli_command(command, client).await
}
