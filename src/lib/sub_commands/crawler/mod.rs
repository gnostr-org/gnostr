/// Compatibility alias for the crawler subcommand parser.
pub type CrawlerSubCommand = crate::crawler::Cli;

/// Compatibility alias for the inner crawler command set.
pub type InnerCrawlerCommand = crate::crawler::Commands;

/// Dispatches crawler subcommands through the shared crawler implementation.
pub async fn dispatch_crawler_command(
    command: InnerCrawlerCommand,
    _client: &reqwest::Client,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        crate::crawler::Commands::Sniper { nip, shitlist } => {
            crate::crawler::run_sniper(nip, shitlist).await?
        }
        crate::crawler::Commands::Watch { shitlist } => {
            crate::crawler::run_watch(shitlist).await?
        }
        crate::crawler::Commands::Nip34 { shitlist } => {
            crate::crawler::run_nip34(shitlist).await?
        }
    }

    Ok(())
}
