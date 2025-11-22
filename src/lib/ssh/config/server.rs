use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::{fs::{read_to_string, write}, path::PathBuf};

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
    pub users: std::collections::HashMap<String, ServerUser>,
    pub welcome_message: Option<String>,
    pub exta: Option<toml::Table>,
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

pub async fn load_server_config(config_path: PathBuf) -> anyhow::Result<ServerConfig> {
    if !config_path.exists() {
        let default_config = include_str!("default_server.toml");
        write(&config_path, default_config).context("Could not write default server config")?;
    }

    let text = read_to_string(&config_path).context("Couldn't read gnostr-ssh.toml")?;
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
