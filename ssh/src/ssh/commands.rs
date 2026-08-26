use std::ffi::OsStr;
use std::path::Component;
use std::str::from_utf8;
use std::{path::PathBuf, process::Stdio};

use anyhow::Context;
use clean_path::Clean;
use log::info;
use russh::{server::Handle, ChannelId, CryptoVec};
use shellwords::split;
use tokio::{io::AsyncReadExt, process::Command};

use crate::config::repo::{load_repo_config, new_repo_config};
use crate::config::server::load_server_config;
use crate::git::Repo;
use crate::utils::CustomContext;
use crate::vars::*;

use super::Handler;

#[derive(Clone)]
pub struct Knob {
    pub handle: Handle,
    pub channel: ChannelId,
}

impl Handler {
    pub async fn handle_command(
        &mut self,
        handle: Handle,
        channel: ChannelId,
        command: &[u8],
    ) -> anyhow::Result<()> {
        let server_config = self.state.lock().await.server_config.clone();
        let knob = Knob { handle, channel };

        let command = from_utf8(command).context("Failed to parse command bytes into a string")?;
        let command = split(command).context("Could not split command into words.")?;

        // Politely decline non-git commands.
        if !GIT_COMMANDS.contains(&command[0].as_str()) {
            knob.close().await?;
            return Ok(());
        }

        // The git plumbing commands give the repo like this: '/repo.git'.
        let mut repo_path = PathBuf::from(&command[1]);
        repo_path = repo_path.strip_prefix("/")?.into();
        repo_path = repo_path.clean();

        // Reject repo paths outside eejit's dir.
        if repo_path.components().next() == Some(Component::ParentDir) {
            knob.close().await?;
            return Ok(());
        }

        // Enforce a .git extension.
        if repo_path.extension().unwrap_or(OsStr::new("")) != "git" {
            repo_path.set_file_name(format!(
                "{}.git",
                repo_path.file_name().unwrap().to_str().unwrap()
            ));
        }

        let command = command[0].clone();

        let user = self.user.clone().unwrap_or_default();
        let username = self.username.clone().unwrap_or(GUEST_USERNAME.to_string());

        let is_admin = user.is_admin.unwrap_or(false);
        let can_create_repos = user.can_create_repos.unwrap_or(false);

        if let Some(welcome_message) = server_config.welcome_message {
            knob.info(&welcome_message.replace('%', &username)).await?;
        }

        // Deny non-admins access to the config repo.
        if !is_admin && repo_path == PathBuf::from(SERVER_CONFIG_REPO) {
            knob.error("Only admins are allowed to access this repository.")
                .await?;
            knob.close().await?;
            return Ok(());
        }

        // Handle non-existent repos, including creating a new one on push for some users.
        let mut new_repo = false;
        if !repo_path.exists() {
            if command == GIT_PUSH_COMMAND && (can_create_repos || is_admin) {
                // Non-admins can only make new repos in thier personal directory.
                if !is_admin {
                    let mut dir = None;
                    if let Some(first_component) = repo_path.components().next() {
                        dir = Some(first_component.as_os_str().to_str().unwrap());
                    }
                    if dir.unwrap_or("") != username {
                        knob.error(
                            "You can only create a new repository under your personal subdirectory.",
                        )
                        .await?;
                        knob.close().await?;
                        return Ok(());
                    }
                }

                knob.info("Creating a new repository...").await?;
                Repo::create_bare(&repo_path).await?;
                new_repo = true;
            } else {
                knob.error("That repository doesn't exist :(").await?;
                knob.close().await?;
                return Ok(());
            }
        }

        if !new_repo && repo_path != PathBuf::from(SERVER_CONFIG_REPO) {
            let repo_config = load_repo_config(&repo_path).await?;

            // Access control.
            // TODO: don't load the repo config on every request.
            if is_admin {
                let is_member = repo_config.members.contains(&username);

                if command == GIT_PUSH_COMMAND && !is_member {
                    knob.error("You don't have permission to push to this repository.")
                        .await?;

                    if let Some(message) = repo_config.failed_push_message {
                        knob.repo_note(&message).await?;
                    }

                    knob.close().await?;
                    return Ok(());
                }

                if !(repo_config.public || is_member) {
                    knob.error("You don't have permission to access this repository.")
                        .await?;

                    knob.close().await?;
                    return Ok(());
                }
            }
        }

        let mut shell = Command::new(&command)
            .arg(&repo_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        let stdin = shell.stdin.take().unwrap();
        self.stdin.insert(channel, stdin);

        let mut shell_stdout = shell.stdout.take().unwrap();

        let state = self.state.clone();
        let fut = async move {
            const BUF_SIZE: usize = 1024 * 32;
            let mut buf = [0u8; BUF_SIZE];
            loop {
                let read = shell_stdout.read(&mut buf).await?;
                if read == 0 {
                    break;
                }
                knob.data(&buf[..read]).await?;
            }

            let status = shell.wait().await?.code().unwrap_or(128) as u32;
            knob.exit_status(status).await?;

            // Rebuild.
            if command == GIT_PUSH_COMMAND && !new_repo {
                if repo_path == PathBuf::from(SERVER_CONFIG_REPO) {
                    info!("Reloading server config...");
                    knob.info("Reloading server config...").await?;
                    state.lock().await.server_config = load_server_config(state.lock().await.config_path.clone()).await?;
                } else {
                    knob.info("Reloading repo information...").await?;
                    state.lock().await.rebuild_site(&repo_path).await?;
                }
            }

            if new_repo {
                new_repo_config(&repo_path, &username).await?;
                knob.info("Created a new repo config - please pull.")
                    .await?;
            }

            knob.eof().await?;
            knob.close().await?;
            Ok::<(), anyhow::Error>(())
        };

        tokio::spawn(fut);
        Ok(())
    }
}

impl Knob {
    async fn close(&self) -> anyhow::Result<()> {
        self.handle
            .close(self.channel)
            .await
            .context("Failed to close handle")?;
        Ok(())
    }
    async fn data(&self, data: &[u8]) -> anyhow::Result<()> {
        let buf = CryptoVec::from_slice(data);
        self.handle
            .data(self.channel, buf)
            .await
            .context("Failed to write data to channel")?;
        Ok(())
    }
    async fn exit_status(&self, status: u32) -> anyhow::Result<()> {
        self.handle
            .exit_status_request(self.channel, status)
            .await
            .context("Failed to set exit status")?;
        Ok(())
    }
    async fn eof(&self) -> anyhow::Result<()> {
        self.handle
            .eof(self.channel)
            .await
            .context("Failed to send EOF")?;
        Ok(())
    }
}
