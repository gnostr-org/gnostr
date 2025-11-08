use anyhow::Result;
use gnostr_relay::App;
use std::path::PathBuf;
//use tokio::task::LocalSet;
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

#[actix_web::main]
pub async fn relay(args: RelaySubCommand) -> Result<()> {
    info!("Start relay server with args: {:?}", args);

    //tracing_subscriber::fmt::init();
    info!("Start relay server");

    let local_set = tokio::task::LocalSet::new();

    local_set.run_until(async move {
        let app_data = gnostr_relay::App::create(
            Some(args.config).ok_or("").expect("REASON"),
            true,
            Some("NOSTR".to_owned()),
            None,
        ).map_err(anyhow::Error::from)?;
        app_data.web_server()?.await.map_err(anyhow::Error::from)
    }).await?;

    info!("Relay server shutdown");


    //let local_set = tokio::task::LocalSet::new();

    //local_set.run_until(async move {
    //    let app_data = App::create(
    //        args.config,
    //        args.watch,
    //        Some("NOSTR".to_owned()),
    //        args.data,
    //    ).map_err(anyhow::Error::from)?;
    //    app_data.web_server()?.await.map_err(anyhow::Error::from)
    //}).await?;

    info!("Relay server shutdown");
    Ok(())
}
