use std::{
    fs::{create_dir_all, read_to_string, write},
    path::{Path, PathBuf},
};

use anyhow::Context as AnyhowContext;
use comrak::{markdown_to_html, ComrakOptions};
use tempfile::tempdir;
use tera::{Context, Tera};

use crate::{ssh::config::repo::RepoConfig, ssh::git::Repo, ssh::state::State, ssh::vars::*};

impl State {
    pub async fn rebuild_site(&self, repo_path: &Path) -> anyhow::Result<()> {
        let config_name = PathBuf::from(REPO_CONFIG_FILE);
        let readmes = ["README.md", "readme.me"];

        let temp_dir = tempdir()?;
        let clone_dir = temp_dir.path().join(repo_path);
        Repo::clone(repo_path, &clone_dir).await?;

        let config: RepoConfig = {
            let text =
                read_to_string(clone_dir.join(&config_name)).context("Couldn't read eejit.toml")?;
            toml::from_str(&text)?
        };

        if !config.public {
            return Ok(());
        }

        let mut readme = None;
        for r in readmes {
            let path = clone_dir.join(r);
            if path.exists() {
                readme = Some(path);
                break;
            }
        }

        if readme.is_none() {
            return Ok(());
        }

        let readme = read_to_string(readme.unwrap())?;
        let body = markdown_to_html(&readme, &ComrakOptions::default());

        let mut context = Context::new();
        context.insert("repo_name", &config.name);
        context.insert("content", &body);
        context.insert(
            "clone_url",
            &format!(
                "ssh://{}:{}/{}",
                self.server_config.hostname,
                self.server_config.port,
                repo_path.to_string_lossy()
            ),
        );

        let template = {
            if let Some(path) = config.web_template {
                read_to_string(clone_dir.join(path)).context("Couldn't read user template")?
            } else {
                include_str!("default.html").to_string()
            }
        };

        let result = Tera::one_off(&template, &context, true)?;

        let mut static_path = PathBuf::from("static").join(repo_path);
        if let Some(ext) = static_path.extension() {
            if ext == "git" {
                static_path.set_extension("");
            }
        }

        if !static_path.exists() {
            create_dir_all(&static_path)?;
        }

        write(static_path.join("index.html"), result)?;

        Ok(())
    }
}
