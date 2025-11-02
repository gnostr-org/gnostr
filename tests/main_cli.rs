#[cfg(test)]
mod tests {
    use std::process::Command;
    use assert_cmd::cargo::cargo_bin;
    use predicates::prelude::PredicateBooleanExt;
    use predicates::str;
    use assert_cmd::assert::OutputAssertExt;
    use anyhow::Result;
    use std::error::Error;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use std::env;
    use gnostr::cli::get_app_cache_path;

    //integrate use asyncgit repo actions
    //integrate use asyncgit repo actions
    //integrate use asyncgit repo actions
    use git2::{Repository, Signature};
    use tempfile::TempDir;

    // Helper function to set up a temporary git repository for testing.
    fn setup_test_repo() -> (TempDir, Repository) {
        let tmp_dir = TempDir::new().unwrap();
        let repo_path = tmp_dir.path();
        let repo = Repository::init(repo_path).unwrap();

        // Configure user name and email
        let mut config = repo.config().unwrap();
        config.set_str("user.name", "Test User").unwrap();
        config.set_str("user.email", "test@example.com").unwrap();
        config.set_str("gnostr.relays", "wss://relay.example.com").unwrap();

        // Create an initial commit
        {
            let signature = Signature::now("Test User", "test@example.com").unwrap();
            let tree_id = {
                let mut index = repo.index().unwrap();
                // Create a dummy file to have a non-empty initial commit
                let file_path = repo_path.join("README.md");
                File::create(&file_path)
                    .unwrap()
                    .write_all(b"Initial commit")
                    .unwrap();
                index.add_path(Path::new("README.md")).unwrap();
                let oid = index.write_tree().unwrap();
                repo.find_tree(oid).unwrap().id()
            };
            let tree = repo.find_tree(tree_id).unwrap();
            repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                "Initial commit",
                &tree,
                &[],
            )
            .unwrap();

            // Ensure the working directory is clean after the initial commit
            repo.reset(repo.head().unwrap().peel_to_commit().unwrap().as_object(), git2::ResetType::Hard, None).unwrap();
        }

        (tmp_dir, repo)
    }

    // Helper to get clap error message for conflicting flags
    fn get_clap_conflict_error(flag1: &str, flag2: &str) -> impl predicates::Predicate<str> {
        let error_msg1 = format!("error: the argument '{}' cannot be used with '{}'", flag1, flag2);
        let error_msg2 = format!("error: the argument '{}' cannot be used with '{}'", flag2, flag1);
        str::contains(error_msg1.clone()).or(str::contains(error_msg2.clone()))
    }

    #[test]
    fn test_logging_flags_conflict() {
        // Test invalid combination: --debug and --logging
        let mut cmd_debug_logging = Command::new(cargo_bin("gnostr"));
        cmd_debug_logging.arg("--debug").arg("--logging").arg("--hash").arg("test");
        cmd_debug_logging.assert()
            .failure()
            .stderr(get_clap_conflict_error("--debug", "--logging"));

        // Test invalid combination: --trace and --logging
        let mut cmd_trace_logging = Command::new(cargo_bin("gnostr"));
        cmd_trace_logging.arg("--trace").arg("--logging").arg("--hash").arg("test");
        cmd_trace_logging.assert()
            .failure()
            .stderr(get_clap_conflict_error("--trace", "--logging"));

        // Test invalid combination: --info and --logging
        let mut cmd_info_logging = Command::new(cargo_bin("gnostr"));
        cmd_info_logging.arg("--info").arg("--logging").arg("--hash").arg("test");
        cmd_info_logging.assert()
            .failure()
            .stderr(get_clap_conflict_error("--info", "--logging"));

        // Test invalid combination: --warn and --logging
        let mut cmd_warn_logging = Command::new(cargo_bin("gnostr"));
        cmd_warn_logging.arg("--warn").arg("--logging").arg("--hash").arg("test");
        cmd_warn_logging.assert()
            .failure()
            .stderr(get_clap_conflict_error("--warn", "--logging"));
    }

    #[test]
    fn test_individual_logging_flags() -> Result<(), Box<dyn Error>> {
        let log_file_path = get_app_cache_path()?.join("gnostr.log");
        let _ = fs::remove_file(&log_file_path); // Clean up previous log file

        // Test valid: --debug only
        let mut cmd_debug_only = Command::new(cargo_bin("gnostr"));
        cmd_debug_only.arg("--debug").arg("--hash").arg("test");
        cmd_debug_only.assert()
            .success();

        // Test valid: --trace only
        let mut cmd_trace_only = Command::new(cargo_bin("gnostr"));
        cmd_trace_only.arg("--trace").arg("--hash").arg("test");
        cmd_trace_only.assert()
            .success();

        // Test valid: --info only
        let mut cmd_info_only = Command::new(cargo_bin("gnostr"));
        cmd_info_only.arg("--info").arg("--hash").arg("test");
        cmd_info_only.assert()
            .success();

        // Test valid: --warn only
        let mut cmd_warn_only = Command::new(cargo_bin("gnostr"));
        cmd_warn_only.arg("--warn").arg("--hash").arg("test");
        cmd_warn_only.assert()
            .success();

        Ok(())
    }

    #[test]
    fn test_logging_flag_only() -> Result<(), Box<dyn Error>> {
        // Test valid: --logging only
        let mut cmd_logging_only = Command::new(cargo_bin("gnostr"));
        cmd_logging_only.arg("--logging").arg("--hash").arg("test");
        cmd_logging_only.assert()
            .success()
            .stdout(str::contains("Logging enabled.")); // Check stdout for file logging message

        Ok(())
    }

    #[test]
    fn test_no_logging_flags() -> Result<(), Box<dyn Error>> {
        // Test valid: No logging flags
        let mut cmd_no_logging = Command::new(cargo_bin("gnostr"));
        cmd_no_logging.arg("--hash").arg("test");
        cmd_no_logging.assert()
            .success()
            // Ensure stderr does NOT contain specific log level indicators or the file logging message.
            // It might contain other debug output like "40:arg=..."
            .stderr(str::contains("level=").not())
            .stderr(str::contains("Logging enabled.").not());

        Ok(())
    }

    #[test]
    fn test_hash_command_prints_hash_and_exits() -> Result<()> {
        let input_string = "test_string";
        let expected_hash = "4b641e9a923d1ea57e18fe41dcb543e2c4005c41ff210864a710b0fbb2654c11"; // SHA256 of "test_string"

        let mut cmd = Command::new(cargo_bin("gnostr"));
        cmd.arg("--hash").arg(input_string);

        cmd.assert()
            .success()
            .stdout(str::diff(expected_hash))
            .stderr(str::is_empty()); // Assuming no other stderr output for this specific case

        Ok(())
    }

    #[test]
    fn test_hash_command_with_debug_flag_prints_hash_and_exits() -> Result<()> {
        let log_file_path = get_app_cache_path()?.join("gnostr.log");
        let _ = fs::remove_file(&log_file_path); // Clean up previous log file

        let input_string = "another_test";

        // Calculate actual SHA256 for "another_test"
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(input_string.as_bytes());
        let actual_expected_hash = format!("{:x}", hasher.finalize());

        let mut cmd = Command::new(cargo_bin("gnostr"));
        cmd.arg("--debug").arg("--hash").arg(input_string);

        cmd.assert()
            .success()
            .stdout(str::diff(actual_expected_hash.clone()));

        Ok(())
    }

    #[test]
    fn test_subcommand_dispatch_tui_help() -> Result<()> {
        let mut cmd = Command::new(cargo_bin("gnostr"));
        cmd.arg("tui").arg("--help");

        cmd.assert()
            .success()
            .stdout(str::contains("Gnostr sub commands"))
            .stdout(str::contains("Options:")) // Check for general options section
            .stderr(str::is_empty());

        Ok(())
    }

    #[test]
    fn test_subcommand_dispatch_chat_help() -> Result<()> {
        let mut cmd = Command::new(cargo_bin("gnostr"));
        cmd.arg("chat").arg("--help");

        cmd.assert()
            .success()
            .stdout(str::contains("Chat sub commands"))
            .stdout(str::contains("Options:")) // Check for general options section
            .stderr(str::is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_legit_mine_default_command() -> Result<(), Box<dyn Error>> {
        let (_tmp_dir, repo) = setup_test_repo();
        let repo_path = repo.workdir().unwrap().to_str().unwrap().to_string();

        // Explicitly create .gnostr directories for the test
        let gnostr_path = PathBuf::from(&repo_path).join(".gnostr");
        let blobs_path = gnostr_path.join("blobs");
        let reflog_path = gnostr_path.join("reflog");
        fs::create_dir_all(&blobs_path).unwrap();
        fs::create_dir_all(&reflog_path).unwrap();

        let mut cmd = Command::new(cargo_bin("gnostr"));
        cmd.arg("legit").arg("--repo").arg(&repo_path).arg("--message").arg("Test mine commit").arg("--pow").arg("0");

        cmd.assert()
            .success()
            .stdout(str::contains("Mined commit hash:"));

        // Verify that .gnostr directories and files were created
        let repo_path_buf = PathBuf::from(&repo_path);
        assert!(repo_path_buf.join(".gnostr").exists());
        assert!(repo_path_buf.join(".gnostr/blobs").exists());
        assert!(repo_path_buf.join(".gnostr/reflog").exists());

        Ok(())
    }

    #[tokio::test]
    async fn test_main_with_gitdir_env_var() -> Result<(), Box<dyn Error>> {
        let (_tmp_dir, repo) = setup_test_repo();
        let repo_path = repo.path().to_str().unwrap().to_string();

        // Set the GNOSTR_GITDIR environment variable
        env::set_var("GNOSTR_GITDIR", &repo_path);

        let mut cmd = Command::new(cargo_bin("gnostr"));
        //setup process to capture the ratatui screen
        cmd.arg("tui"); // TUI is the default if no other subcommand is given, but we explicitly call it here

        // We expect it to succeed, but the actual TUI interaction is hard to test in a CLI test.
        // We can check for some debug output if --debug is enabled.
        cmd.arg("--debug");

        cmd.assert()
            .code(0) // Expect a successful exit from the TUI
            .stderr(str::contains(format!("333:The GNOSTR_GITDIR environment variable is set to: {}", repo_path.clone())));

        // Unset the environment variable to avoid affecting other tests
        env::remove_var("GNOSTR_GITDIR");

        Ok(())
    }

    #[tokio::test]
    async fn test_main_with_gitdir_cli_arg() -> Result<(), Box<dyn Error>> {
        let (_tmp_dir, repo) = setup_test_repo();
        let repo_path = repo.path().to_str().unwrap().to_string();

        //setup process to capture ratatui screen
        let mut cmd = Command::new(cargo_bin("gnostr"));
        cmd.arg("--gitdir").arg(&repo_path).arg("tui");

        cmd.arg("--debug");

        cmd.assert()
            .code(0) // Expect a successful exit from the TUI
            .stderr(str::contains(format!("339:OVERRIDE!! The git directory is: \"{}\"", repo_path)));

        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_gitsh_command_error_output() -> Result<(), Box<dyn Error>> {
        let mut cmd = Command::new(cargo_bin("gnostr"));
        cmd.arg("gitsh").arg("nostr://test_url");

        cmd.assert()
            .failure()
            .stdout(str::contains("Mock SSH Start Error"))
            .stdout(str::contains("EXAMPLE:server.toml"))
            .stdout(str::contains("check the port in your server.toml is available!"))
            .stdout(str::contains("EXAMPLE:repo.toml"));

        Ok(())
    }
}
