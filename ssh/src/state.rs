use std::path::PathBuf;
use crate::config::server::{load_server_config, ServerConfig};
use crate::config::repo::{load_repo_config_from_path, RepoConfig};

pub struct State {
    pub server_config: ServerConfig,
    pub repo_config: RepoConfig,
    pub config_path: PathBuf,
    pub repo_config_path: PathBuf,
}

impl State {
    pub async fn new(config: PathBuf, repo_config: PathBuf) -> anyhow::Result<Self> {
        let state = State {
            server_config: load_server_config(config.clone()).await?,
            repo_config: load_repo_config_from_path(repo_config.clone()).await?,
            config_path: config,
            repo_config_path: repo_config,
        };

        Ok(state)
    }
}
