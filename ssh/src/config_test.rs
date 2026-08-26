#[cfg(test)]
mod tests {
    use crate::config::server::{load_server_config, ServerUser};
    use anyhow::Context;
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_load_server_config_missing_file() -> anyhow::Result<()> {
        let temp_dir = tempdir().context("Failed to create tempdir")?;
        let config_path = temp_dir.path().join("non_existent_gnostr-ssh.toml");

        // Ensure the file does not exist initially
        assert!(!config_path.exists());

        let server_config = load_server_config(config_path.clone()).await?;

        // Verify that the file was created
        assert!(config_path.exists());
        // Verify default values (assuming default_server.toml has these)
        assert_eq!(server_config.port, 2222);
        assert_eq!(server_config.name, "gnostr-ssh Server");

        Ok(())
    }

    #[tokio::test]
    async fn test_load_server_config_valid_file() -> anyhow::Result<()> {
        let temp_dir = tempdir().context("Failed to create tempdir")?;
        let config_path = temp_dir.path().join("valid_gnostr-ssh.toml");

        let content = r#"
name = "Test Server"
port = 2223
hostname = "test.example.com"

[users.testuser]
is_admin = true
public_key = "ssh-rsa AAAAtestkey"
"#;
        fs::write(&config_path, content).context("Failed to write valid config file")?;

        let server_config = load_server_config(config_path.clone()).await?;

        assert_eq!(server_config.name, "Test Server");
        assert_eq!(server_config.port, 2223);
        assert_eq!(server_config.hostname, "test.example.com");
        assert!(server_config.users.contains_key("testuser"));
        assert!(server_config.users["testuser"].is_admin.unwrap());

        Ok(())
    }

    #[tokio::test]
    async fn test_load_server_config_invalid_file() -> anyhow::Result<()> {
        let temp_dir = tempdir().context("Failed to create tempdir")?;
        let config_path = temp_dir.path().join("invalid_gnostr-ssh.toml");

        let content = r#"
invalid toml
key = "value"
"#;
        fs::write(&config_path, content).context("Failed to write invalid config file")?;

        let result = load_server_config(config_path.clone()).await;

        assert!(result.is_err());
        if let Err(error) = result {
            assert!(error.to_string().contains("TOML parse error"));
        } else {
            panic!("Expected an error, but got Ok");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_add_user_via_config_modification() -> anyhow::Result<()> {
        let temp_dir = tempdir().context("Failed to create tempdir")?;
        let config_path = temp_dir.path().join("user_management_gnostr-ssh.toml");

        // 1. Load initial config (will create default if missing)
        let mut server_config = load_server_config(config_path.clone()).await?;

        // 2. Define new user data
        let username = "new_test_user".to_string();
        let public_key = "ssh-rsa AAAAnewuserkey".to_string();
        let is_admin = true;
        let can_create_repos = false;

        // 3. Modify ServerConfig to add the user
        let new_user = ServerUser {
            public_key: public_key.clone(),
            is_admin: Some(is_admin),
            can_create_repos: Some(can_create_repos),
        };
        server_config.users.insert(username.clone(), new_user);

        // 4. Save the updated config to file
        let toml_string = toml::to_string_pretty(&server_config).context("Failed to serialize server config")?;
        fs::write(&config_path, toml_string).context("Failed to write updated server config")?;

        // 5. Reload the config from file
        let reloaded_config = load_server_config(config_path.clone()).await?;

        // 6. Assert the user was added correctly
        assert!(reloaded_config.users.contains_key(&username));
        let user_data = reloaded_config.users.get(&username).unwrap();
        assert_eq!(user_data.public_key, public_key);
        assert_eq!(user_data.is_admin, Some(is_admin));
        assert_eq!(user_data.can_create_repos, Some(can_create_repos));

        Ok(())
    }
}
