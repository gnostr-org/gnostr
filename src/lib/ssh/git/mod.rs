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
        Repo::new(path).await
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
        Repo::new(to).await
    }

    pub async fn new(path: &Path) -> anyhow::Result<Repo> {
        tokio::process::Command::new("git")
            .current_dir(path)
            .arg("config")
            .arg("--local")
            .arg("user.name")
            .arg("Eejit Server")
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

    pub async fn push_changes(&self, branch: &str) -> anyhow::Result<()> {
        tokio::process::Command::new("git")
            .current_dir(&self.dir)
            .arg("push")
            .arg("origin")
            .arg(branch)
            .output()
            .await?
            .status
            .exit_ok()
            .context("Failed to push changes")?;

        Ok(())
    }

    pub async fn add_and_commit(&self, file: &str, message: &str) -> anyhow::Result<()> {
        tokio::process::Command::new("git")
            .current_dir(&self.dir)
            .arg("add")
            .arg(file)
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
        Ok(())
    }

    pub async fn fetch_changes(&self, remote: &str, branch: &str) -> anyhow::Result<()> {
        tokio::process::Command::new("git")
            .current_dir(&self.dir)
            .arg("fetch")
            .arg(remote)
            .arg(branch)
            .output()
            .await?
            .status
            .exit_ok()
            .context("Failed to fetch changes")?;
        Ok(())
    }

    pub async fn checkout_branch(&self, branch: &str) -> anyhow::Result<()> {
        tokio::process::Command::new("git")
            .current_dir(&self.dir)
            .arg("checkout")
            .arg(branch)
            .output()
            .await?
            .status
            .exit_ok()
            .context("Failed to checkout branch")?;
        Ok(())
    }

    pub async fn reset_hard(&self, target: &str) -> anyhow::Result<()> {
        tokio::process::Command::new("git")
            .current_dir(&self.dir)
            .arg("reset")
            .arg("--hard")
            .arg(target)
            .output()
            .await?
            .status
            .exit_ok()
            .context("Failed to reset hard")?;
        Ok(())
    }
}
