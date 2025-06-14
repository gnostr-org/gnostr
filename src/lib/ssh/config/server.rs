use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{copy, read_to_string, remove_file},
    path::PathBuf,
};
use tempfile::tempdir;
use toml::Table;

use crate::{ssh::git::Repo, ssh::vars::*};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerUser {
    pub public_key: String,
    pub is_admin: Option<bool>,
    pub can_create_repos: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerConfig {
    pub name: String,
    pub hostname: String,
    pub port: u16,
    pub users: HashMap<String, ServerUser>,
    pub welcome_message: Option<String>,
    pub exta: Option<Table>,
}

// The default for ServerUser is used for guest access.
impl Default for ServerUser {
    fn default() -> Self {
        Self {
            public_key: String::from(""),
            is_admin: Some(false),
            can_create_repos: Some(false),
        }
    }
}

pub fn get_server_config_repo_path() -> PathBuf {
    let home_dir = dirs::home_dir().expect("Could not determine home directory");
    home_dir.join(".gnostr/.git")
}
pub fn get_server_config_file_path() -> PathBuf {
    let home_dir = dirs::home_dir().expect("Could not determine home directory");
    home_dir.join("server.toml")
}

pub async fn load_server_config() -> anyhow::Result<ServerConfig> {

    let repo_name = get_server_config_repo_path();
    let config_name = get_server_config_file_path();

    let mut new = false;
    if !repo_name.exists() {
        if !config_name.exists() {
            return Err(anyhow!(
                "There's no server config, and no initial config to move in!"
            ));
        }
        Repo::create_bare(&repo_name).await?;
        new = true;
    }

    let temp_dir = tempdir()?;
    let clone_dir = temp_dir.path().join(&repo_name);
    let repo = Repo::clone(&repo_name, &clone_dir).await?;

    if new {
        copy(&config_name, clone_dir.join(&config_name))?;
        remove_file(&config_name)?;
        repo.push_changes("chore: move in initial config").await?;
    }

    let text = read_to_string(clone_dir.join(&config_name)).context("Couldn't read server.toml")?;
    Ok(toml::from_str(&text)?)
}

impl ServerConfig {
    pub fn get_user(&self, key: &str) -> Option<(String, ServerUser)> {
        for user in self.users.keys() {
            let key_data = self.users[user].public_key.split(' ').nth(1).unwrap();

            if key == key_data && self.users[user].is_admin.unwrap_or(false) {
                return Some((user.to_string(), self.users[user].clone()));
            }
        }

        None
    }
}
