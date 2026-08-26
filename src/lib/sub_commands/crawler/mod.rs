use log::debug;

/// Compatibility alias for the crawler subcommand parser.
pub type CrawlerSubCommand = crate::crawler::Cli;

/// Compatibility alias for the inner crawler command set.
pub type InnerCrawlerCommand = crate::crawler::Commands;

/// Dispatches crawler subcommands through the shared crawler implementation.
pub async fn dispatch_crawler_command(
    command: InnerCrawlerCommand,
    client: &reqwest::Client,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("dispatch_crawler_command: start");
    match command {
        crate::crawler::Commands::Sniper { nip, shitlist } => {
            debug!("dispatch_crawler_command: sniper nip={nip} shitlist={shitlist:?}");
            crate::crawler::run_sniper(nip, shitlist, client).await?
        }
        crate::crawler::Commands::Watch { shitlist } => {
            debug!("dispatch_crawler_command: watch shitlist={shitlist:?}");
            crate::crawler::run_watch(shitlist, client).await?
        }
        crate::crawler::Commands::Nip34 { shitlist } => {
            debug!("dispatch_crawler_command: nip34 shitlist={shitlist:?}");
            crate::crawler::run_nip34(shitlist, client).await?
        }
        crate::crawler::Commands::Crawl(args) => {
            debug!("dispatch_crawler_command: crawl args={args:?}");
            gnostr_crawler::run(&args).await?;
        }
        crate::crawler::Commands::Serve { port, detach } => {
            debug!("dispatch_crawler_command: serve port={port} detach={detach}");
            if detach {
                crate::utils::detach::spawn_detached_current_exe_named(
                    Some("gnostr-crawler"),
                    vec![
                        "crawler".to_string(),
                        "serve".to_string(),
                        "--port".to_string(),
                        port.to_string(),
                    ],
                )?;
            } else {
                crate::crawler::run_api_server(port).await?;
            }
        }
    }

    debug!("dispatch_crawler_command: done");
    Ok(())
}
