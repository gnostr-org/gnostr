use gnostr_relay::App;
use tracing::info;

#[actix_web::main]
async fn main() -> gnostr_relay::Result<()> {
    tracing_subscriber::fmt::init();
    info!("Start relay server");
    let app_data = App::create(
        Some("config/gnostr.toml"),
        true,
        Some("NOSTR".to_owned()),
        None,
    )?;
    app_data.web_server()?.await?;
    info!("Relay server shutdown");
    Ok(())
}
