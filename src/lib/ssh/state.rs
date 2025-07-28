use crate::ssh::config::server::{load_server_config, ServerConfig};
use log::info;
#[derive(Debug)]
pub struct State {
    pub server_config: ServerConfig,
}

impl State {
    pub async fn new() -> anyhow::Result<Self> {
        info!("State::new");
        let state = State {
            server_config: load_server_config().await?,
        };

        Ok(state)
    }
}
