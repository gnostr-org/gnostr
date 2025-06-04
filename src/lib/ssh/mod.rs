use log::info;
use state::State;
use std::sync::Arc;
use tokio::sync::Mutex;

mod config;
mod git;
mod site;
mod ssh;
mod state;
mod utils;
mod vars;

pub async fn start() -> anyhow::Result<()> {
    info!("Loading state...");
    let state = State::new().await?;
    let state = Arc::new(Mutex::new(state));

    println!("{:?}", state);

    info!("Starting server...");
    #[cfg(not(target_os = "windows"))]
    let _ = sd_notify::notify(true, &[sd_notify::NotifyState::Ready]);
    ssh::start_server(state).await?;
    Ok(())
}
