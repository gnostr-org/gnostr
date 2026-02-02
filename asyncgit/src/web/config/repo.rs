use std::{
    fs::{read_to_string, write},
    path::{Path, PathBuf},
};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use tempfile::tempdir;
use toml::Table;

use crate::{git::Repo, vars::*};

#[derive(Serialize, Deserialize)]
pub struct RepoConfig {
    pub name: String,
    pub public: bool,
    pub members: Vec<String>,
    pub failed_push_message: Option<String>,
    pub web_template: Option<String>,
    pub extra: Option<Table>,
}

pub async fn load_repo_config(repo_path: &Path) -> anyhow::Result<RepoConfig> {
    let config_name = PathBuf::from(REPO_CONFIG_FILE);

    let temp_dir = tempdir()?;
    let clone_dir = temp_dir.path().join(repo_path);
    Repo::clone(repo_path, &clone_dir).await?;

    let text = read_to_string(clone_dir.join(&config_name)).context("Couldn't read eejit.toml")?;
    Ok(toml::from_str(&text)?)
}

pub async fn new_repo_config(repo_path: &Path, username: &str) -> anyhow::Result<()> {
    let config_name = PathBuf::from(REPO_CONFIG_FILE);

    let temp_dir = tempdir()?;
    let clone_dir = temp_dir.path().join(repo_path);
    let repo = Repo::clone(repo_path, &clone_dir).await?;

    let config = RepoConfig {
        name: repo_path.to_str().unwrap().to_string(),
        public: false,
        members: vec![username.to_string()],
        failed_push_message: None,
        extra: None,
        web_template: None,
    };

    let text = toml::to_string(&config)?;
    write(clone_dir.join(config_name), text).context("Could not write default repo config")?;
    repo.push_changes("chore: create repo config").await?;
    Ok(())
}
