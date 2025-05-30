use std::{
    path::{Path, PathBuf},
    process::ExitStatus,
};

use anyhow::{anyhow, Context};

pub struct Repo {
    dir: PathBuf,
}

trait ExitOK {
    fn exit_ok(self) -> anyhow::Result<()>;
}

impl ExitOK for ExitStatus {
    fn exit_ok(self) -> anyhow::Result<()> {
        if self.success() {
            Ok(())
        } else {
            Err(anyhow!("Command excited unsuccessfully"))
        }
    }
}

impl Repo {
    pub async fn create_bare(path: &Path) -> anyhow::Result<Self> {
        tokio::process::Command::new("git")
            .arg("init")
            .arg("--bare")
            .arg(path)
            .output()
            .await?
            .status
            .exit_ok()
            .context("Failed to create bare repo")?;
        tokio::process::Command::new("git")
            .current_dir(path)
            .arg("branch")
            .arg("-m")
            .arg("main")
            .output()
            .await?
            .status
            .exit_ok()
            .context("Failed to rename branch")?;
        Repo::from(path).await
    }

    pub async fn clone(from: &Path, to: &Path) -> anyhow::Result<Self> {
        tokio::process::Command::new("git")
            .arg("clone")
            .arg(from.to_str().unwrap())
            .arg(to.to_str().unwrap())
            .output()
            .await?
            .status
            .exit_ok()
            .context("Failed to clone repo")?;
        Repo::from(to).await
    }

    pub async fn from(path: &Path) -> anyhow::Result<Repo> {
        tokio::process::Command::new("git")
            .current_dir(path)
            .arg("config")
            .arg("--local")
            .arg("user.name")
            .arg("gnostr-gnit-server")
            .output()
            .await?
            .status
            .exit_ok()
            .context("Failed to set local config for cloned repo")?;

        tokio::process::Command::new("git")
            .current_dir(path)
            .arg("config")
            .arg("--local")
            .arg("user.email")
            .arg("N/A")
            .output()
            .await?
            .status
            .exit_ok()
            .context("Failed to set local config for cloned repo")?;

        Ok(Repo {
            dir: path.to_path_buf(),
        })
    }

    pub async fn push_changes(&self, message: &str) -> anyhow::Result<()> {
        tokio::process::Command::new("git")
            .current_dir(&self.dir)
            .arg("add")
            .arg(".")
            .output()
            .await?
            .status
            .exit_ok()
            .context("Failed to stage changes")?;

        tokio::process::Command::new("git")
            .current_dir(&self.dir)
            .arg("commit")
            .arg("-m")
            .arg(message)
            .output()
            .await?
            .status
            .exit_ok()
            .context("Failed to create commit")?;

        tokio::process::Command::new("git")
            .current_dir(&self.dir)
            .arg("push")
            .output()
            .await?
            .status
            .exit_ok()
            .context("Failed to push changes")?;

        Ok(())
    }
}
