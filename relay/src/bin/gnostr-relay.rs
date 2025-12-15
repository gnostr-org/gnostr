use anyhow::Result;
use clap::Parser;
use gnostr_relay::App;
use tracing::info;
use tracing_subscriber::{fmt, EnvFilter};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The logging level
    #[clap(short, long, default_value = "info")]
    logging: String,
}

#[actix_web::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let filter = EnvFilter::new(args.logging);
    fmt().with_env_filter(filter).init();
    info!("Start relay server");

    let local_set = tokio::task::LocalSet::new();

    local_set
        .run_until(async move {
            let config_file_path = "config/gnostr.toml";
            let setting_path = if std::path::Path::new(config_file_path).exists() {
                info!("Load config from file: {}", config_file_path);
                Some(config_file_path)
            } else {
                info!("Config file not found: {}, loading default config.", config_file_path);
                None
            };

            let app_data = gnostr_relay::App::create(
                setting_path,
                true,
                Some("NOSTR".to_owned()),
                None,
            )
            .map_err(anyhow::Error::from)?;
            app_data.web_server()?.await.map_err(anyhow::Error::from)
        })
        .await?;

    info!("Relay server shutdown");
    Ok(())
}