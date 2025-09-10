use gnostr_relay::App;
use tracing::info;

#[actix_web::main]

async fn main() -> gnostr_relay::Result<()> {
    tracing_subscriber::fmt::init();
    info!("Start relay server");
    let mut app_data = App::create(
        Some("demo.toml"),
        true,
        Some("NOSTR".to_owned()),
        None,
    )?;

    #[cfg(feature = "metrics")]
    {
        app_data = app_data.add_extension(gnostr_extensions::Metrics::new());
    }

    app_data = app_data.add_extension(gnostr_extensions::Auth::new());

    #[cfg(feature = "rate_limiter")]
    {
        app_data = app_data.add_extension(gnostr_extensions::Ratelimiter::new());
    }

    app_data.web_server()?.await?;
    info!("Relay server shutdown");
    Ok(())
}
