#[cfg(test)]
mod tests {
    use crate::config::server::{load_server_config, ServerConfig, ServerUser};
    use crate::config::repo::{load_repo_config_from_path, RepoConfig};
    use crate::state::State;
    use anyhow::Context;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    async fn create_temp_server_config(dir: &PathBuf) -> anyhow::Result<PathBuf> {
        let config_path = dir.join("test_server.toml");
        let content = r#"
name = "Test Server"
port = 2222
hostname = "localhost"
[users]
[users.testuser]
is_admin = true
public_key = "ssh-rsa AAAAtestkey"
"#;
        fs::write(&config_path, content).context("Failed to write temp server config")?;
        Ok(config_path)
    }

    async fn create_temp_repo_config(dir: &PathBuf) -> anyhow::Result<PathBuf> {
        let repo_config_path = dir.join("test_repo.toml");
        let content = r#"
name = "test_repo"
public = true
members = ["testuser"]
"#;
        fs::write(&repo_config_path, content).context("Failed to write temp repo config")?;
        Ok(repo_config_path)
    }

    #[tokio::test]
    async fn test_state_new() -> anyhow::Result<()> {
        let temp_dir = tempdir().context("Failed to create tempdir")?;
        let temp_path = temp_dir.path().to_path_buf();

        let server_config_path = create_temp_server_config(&temp_path).await?;
        let repo_config_path = create_temp_repo_config(&temp_path).await?;

        let state = State::new(server_config_path.clone(), repo_config_path.clone()).await?;

        // Verify server config loaded correctly
        assert_eq!(state.server_config.name, "Test Server");
        assert!(state.server_config.users.contains_key("testuser"));

        // Verify repo config loaded correctly
        assert_eq!(state.repo_config.name, "test_repo");
        assert!(state.repo_config.public);
        assert!(state.repo_config.members.contains(&"testuser".to_string()));

        // Verify config paths are stored
        assert_eq!(state.config_path, server_config_path);
        assert_eq!(state.repo_config_path, repo_config_path);

        Ok(())
    }
}
