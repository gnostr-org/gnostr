use gnostr_relay::App;
use tracing::info;
use anyhow::Result;

#[actix_web::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("Start relay server");

    let local_set = tokio::task::LocalSet::new();

    local_set.run_until(async move {
        let app_data = gnostr_relay::App::create(
            Some("config/gnostr.toml"),
            true,
            Some("NOSTR".to_owned()),
            None,
        ).map_err(anyhow::Error::from)?;
        app_data.web_server()?.await.map_err(anyhow::Error::from)
    }).await?;

    info!("Relay server shutdown");
    Ok(())
}
