use anyhow::{Error, Result};
use tracing::info;
use tracing_subscriber::{fmt, EnvFilter};

use crate::cli::RelayCli;

pub async fn run(config: RelayCli, setting_path: Option<&str>, app_name: &str) -> Result<()> {
    let filter = EnvFilter::new(config.logging);
    fmt().with_env_filter(filter).init();
    info!("Start relay server");

    let local_set = tokio::task::LocalSet::new();

    local_set
        .run_until(async move {
            let app_data = crate::App::create(
                setting_path,
                true,
                Some(app_name.to_owned()),
                None,
            )
            .map_err(Error::from)?;
            app_data.web_server()?.await.map_err(Error::from)
        })
        .await?;

    info!("Relay server shutdown");
    Ok(())
}
