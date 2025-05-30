use crate::ssh::config::server::{load_server_config, ServerConfig};

pub struct State {
    pub server_config: ServerConfig,
}

impl State {
    pub async fn new() -> anyhow::Result<Self> {
        let state = State {
            server_config: load_server_config().await?,
        };

        Ok(state)
    }
}
