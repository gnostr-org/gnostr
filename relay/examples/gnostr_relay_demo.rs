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
    let data_path = app_data.setting.read().data.path.clone();
    let server = app_data.web_server()?;
    gnostr_relay::write_listen_endpoint(&data_path, &server.addrs())?;
    server.await?;
    info!("Relay server shutdown");
    Ok(())
}
