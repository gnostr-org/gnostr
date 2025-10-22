use anyhow::Result;
use gnostr_relay::App;
use std::path::PathBuf;
use tracing::info;

#[derive(clap::Parser, Debug, Clone)]
pub struct RelaySubCommand {
    /// Path to the configuration file.
    #[clap(short, long)]
    pub config: Option<PathBuf>,

    /// Path to the data directory.
    #[clap(short, long)]
    pub data: Option<PathBuf>,

    /// Watch for configuration file changes.
    #[clap(short, long)]
    pub watch: bool,
}

pub async fn relay(args: RelaySubCommand) -> Result<()> {
    info!("Start relay server with args: {:?}", args);
    let app_data = App::create(args.config, args.watch, Some("NOSTR".to_owned()), args.data)?;
    app_data.web_server()?.await?;
    info!("Relay server shutdown");
    Ok(())
}