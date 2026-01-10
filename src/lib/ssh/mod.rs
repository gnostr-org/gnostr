pub mod config;
pub mod git;
pub mod server;
pub mod site;
pub mod state;
pub mod utils;
pub mod vars;

#[cfg(test)]
#[path = "./utils_test.rs"]
mod utils_test;

#[cfg(test)]
#[path = "./git_test.rs"]
mod git_test;

#[cfg(test)]
#[path = "./config_test.rs"]
mod config_test;

#[cfg(test)]
#[path = "./state_test.rs"]
mod state_test;

use std::{path::PathBuf, sync::Arc};

use anyhow::anyhow;
use log::info;
use state::State;
use tokio::sync::Mutex;

pub async fn start() -> anyhow::Result<()> {
    let port = 2222;
    let config = PathBuf::from("gnostr-ssh.toml");
    let repo_config = PathBuf::from("gnostr-repo.toml");

    if utils::is_port_in_use(port).await {
        return Err(anyhow!("Port {} is already in use.", port));
    }

    info!("Loading state...");
    let mut state = State::new(config, repo_config).await?;
    state.server_config.port = port;

    let state = Arc::new(Mutex::new(state));

    info!("Starting server on port {}...", port);
    #[cfg(not(target_os = "windows"))]
    let _ = sd_notify::notify(true, &[sd_notify::NotifyState::Ready]);
    server::start_server(state).await?;
    Ok(())
}
