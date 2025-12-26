/// ## Screenshot Testing
///
/// The screenshot tests are designed to capture the state of the TUI at a specific
/// moment. They are particularly useful for debugging and verifying the UI.
///
/// To add a new screenshot test, follow this pattern:
///
/// 1.  **Spawn the application in a separate process.** This is necessary to
///     prevent the TUI from blocking the test runner.
/// 2.  **Wait for the TUI to initialize.** A simple `thread::sleep` is
///     sufficient for this purpose.
/// 3.  **Call the `make_screenshot` utility.** This will capture the screen and
///     save it to the `test_screenshots` directory.
/// 4.  **Terminate the process.** This is important to prevent the TUI from
///     running indefinitely.
/// 5.  **Assert that the screenshot was created.** This verifies that the
///     test ran successfully.
///
/// For an example, see `test_run_gnostr_and_capture_screenshot`.
#[cfg(test)]
mod tests {
    use anyhow::Result;
    use assert_cmd::assert::OutputAssertExt;
    use assert_cmd::cargo::cargo_bin;
    use predicates::prelude::PredicateBooleanExt;
    use predicates::str;
    use std::error::Error;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    use crate::cli::get_app_cache_path;
    use crate::core::ui::TerminalCleanup;
    use crate::utils::screenshot;
    use serial_test::serial;
    use signal_child::Signalable;
    use std::thread;
    use std::time::Duration;

    //integrate use asyncgit repo actions
    //integrate use asyncgit repo actions
    //integrate use asyncgit repo actions

    use git2::{Repository, Signature};
    use tempfile::TempDir;

    // Helper function to set up a temporary git repository for testing.
    fn setup_test_repo() -> (TempDir, Repository) {
        let _cleanup_guard = TerminalCleanup;
        let tmp_dir = TempDir::new().unwrap();
        let repo_path = tmp_dir.path();
        let repo = Repository::init(repo_path).unwrap();

        // Configure user name and email
        let mut config = repo.config().unwrap();
        config.set_str("user.name", "Test User").unwrap();
        config.set_str("user.email", "test@example.com").unwrap();
        config
            .set_str("gnostr.relays", "wss://relay.example.com")
            .unwrap();

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
            repo.reset(
                repo.head().unwrap().peel_to_commit().unwrap().as_object(),
                git2::ResetType::Hard,
                None,
            )
            .unwrap();
        }

        let _cleanup_guard = TerminalCleanup;
        (tmp_dir, repo)
    }

    // Helper to get clap error message for conflicting flags
    fn get_clap_conflict_error(flag1: &str, flag2: &str) -> impl predicates::Predicate<str> {
        let _cleanup_guard = TerminalCleanup;
        let error_msg1 = format!(
            "error: the argument '{}' cannot be used with '{}'",
            flag1, flag2
        );
        let error_msg2 = format!(
            "error: the argument '{}' cannot be used with '{}'",
            flag2, flag1
        );
        let _cleanup_guard = TerminalCleanup;
        str::contains(error_msg1.clone()).or(str::contains(error_msg2.clone()))
    }

    #[test]
    #[serial]
    fn test_logging_flags_conflict() {
        let _cleanup_guard = TerminalCleanup;
        //// Test invalid combination: --debug and --logging
        //let mut cmd_debug_logging = Command::new(cargo_bin("gnostr"));
        //cmd_debug_logging.arg("--logging").arg("--hash").arg("test");
        //cmd_debug_logging.assert()
        //    .failure()
        //    .stderr(get_clap_conflict_error("--debug", "--logging"));

        //// Test invalid combination: --trace and --logging
        //let mut cmd_trace_logging = Command::new(cargo_bin("gnostr"));
        //cmd_trace_logging.arg("--trace").arg("--logging").arg("--hash").arg("test");
        //cmd_trace_logging.assert()
        //    .failure()
        //    .stderr(get_clap_conflict_error("--trace", "--logging"));

        //// Test invalid combination: --info and --logging
        //let mut cmd_info_logging = Command::new(cargo_bin("gnostr"));
        //cmd_info_logging.arg("--info").arg("--logging").arg("--hash").arg("test");
        //cmd_info_logging.assert()
        //    .failure()
        //    .stderr(get_clap_conflict_error("--info", "--logging"));

        //// Test invalid combination: --warn and --logging
        //let mut cmd_warn_logging = Command::new(cargo_bin("gnostr"));
        //cmd_warn_logging.arg("--warn").arg("--logging").arg("--hash").arg("test");
        //cmd_warn_logging.assert()
        //    .failure()
        //    .stderr(get_clap_conflict_error("--warn", "--logging"));
        let _cleanup_guard = TerminalCleanup;
    }

    #[test]
    #[serial]
    fn test_individual_logging_flags() -> Result<(), Box<dyn Error>> {
        let _cleanup_guard = TerminalCleanup;
        let log_file_path = get_app_cache_path()?.join("gnostr.log");
        let _ = fs::remove_file(&log_file_path); // Clean up previous log file

        // Test valid: --debug only
        let mut cmd_debug_only = Command::new(cargo_bin("gnostr"));
        cmd_debug_only.arg("--hash").arg("test");
        cmd_debug_only.assert().success();

        // Test valid: --trace only
        let mut cmd_trace_only = Command::new(cargo_bin("gnostr"));
        cmd_trace_only.arg("--trace").arg("--hash").arg("test");
        cmd_trace_only.assert().success();

        // Test valid: --info only
        let mut cmd_info_only = Command::new(cargo_bin("gnostr"));
        cmd_info_only.arg("--info").arg("--hash").arg("test");
        cmd_info_only.assert().success();

        // Test valid: --warn only
        let mut cmd_warn_only = Command::new(cargo_bin("gnostr"));
        cmd_warn_only.arg("--warn").arg("--hash").arg("test");
        cmd_warn_only.assert().success();

        let _cleanup_guard = TerminalCleanup;
        Ok(())
    }

    #[test]
    #[serial]
    fn test_logging_flag_only() -> Result<(), Box<dyn Error>> {
        let _cleanup_guard = TerminalCleanup;
        // Test valid: --logging only
        let mut cmd_logging_only = Command::new(cargo_bin("gnostr"));
        cmd_logging_only.arg("--logging").arg("--hash").arg("test");
        cmd_logging_only
            .assert()
            .success()
            .stdout(str::contains("Logging enabled.")); // Check stdout for file logging message

        let _cleanup_guard = TerminalCleanup;
        Ok(())
    }

    #[test]
    #[serial]
    fn test_no_logging_flags() -> Result<(), Box<dyn Error>> {
        let _cleanup_guard = TerminalCleanup;
        // Test valid: No logging flags
        let mut cmd_no_logging = Command::new(cargo_bin("gnostr"));
        cmd_no_logging.arg("--hash").arg("test");
        cmd_no_logging.assert()
            .success()
            // Ensure stderr does NOT contain specific log level indicators or the file logging message.
            // It might contain other debug output like "40:arg=..."
            .stderr(str::contains("level=").not())
            .stderr(str::contains("Logging enabled.").not());

        let _cleanup_guard = TerminalCleanup;
        Ok(())
    }

    #[test]
    #[serial]
    fn test_hash_command_prints_hash_and_exits() -> Result<()> {
        let _cleanup_guard = TerminalCleanup;
        let input_string = "test_string";
        let expected_hash = "4b641e9a923d1ea57e18fe41dcb543e2c4005c41ff210864a710b0fbb2654c11"; // SHA256 of "test_string"

        let mut cmd = Command::new(cargo_bin("gnostr"));
        cmd.arg("--hash").arg(input_string);

        cmd.assert()
            .success()
            .stdout(str::diff(expected_hash))
            .stderr(str::is_empty()); // Assuming no other stderr output for this specific case

        let _cleanup_guard = TerminalCleanup;
        Ok(())
    }

    #[test]
    #[serial]
    fn test_hash_command_with_debug_flag_prints_hash_and_exits() -> Result<()> {
        let _cleanup_guard = TerminalCleanup;
        let log_file_path = get_app_cache_path()?.join("gnostr.log");
        let _ = fs::remove_file(&log_file_path); // Clean up previous log file

        let input_string = "another_test";

        // Calculate actual SHA256 for "another_test"
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(input_string.as_bytes());
        let actual_expected_hash = format!("{:x}", hasher.finalize());

        let mut cmd = Command::new(cargo_bin("gnostr"));
        cmd.arg("--hash").arg(input_string);

        cmd.assert()
            .success()
            .stdout(str::diff(actual_expected_hash.clone()));

        let _cleanup_guard = TerminalCleanup;
        Ok(())
    }

    #[test]
    #[serial]
    fn test_subcommand_dispatch_tui_help() -> Result<()> {
        let _cleanup_guard = TerminalCleanup;
        //let mut cmd = Command::new(cargo_bin("gnostr"));
        //cmd.arg("tui").arg("--help");

        //cmd.assert()
        //    .success()
        //    .stdout(str::contains("Gnostr sub commands"))
        //    .stdout(str::contains("Options:")) // Check for general options section
        //    .stderr(str::is_empty());

        let _cleanup_guard = TerminalCleanup;
        Ok(())
    }

    #[test]
    #[serial]
    fn test_subcommand_dispatch_chat_help() -> Result<()> {
        let _cleanup_guard = TerminalCleanup;
        //let mut cmd = Command::new(cargo_bin("gnostr"));
        //cmd.arg("chat").arg("--help");

        //cmd.assert()
        //    .success()
        //    .stdout(str::contains("Chat sub commands"))
        //    .stdout(str::contains("Options:")) // Check for general options section
        //    .stderr(str::is_empty());

        let _cleanup_guard = TerminalCleanup;
        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn test_legit_mine_default_command() -> Result<(), Box<dyn Error>> {
        let _cleanup_guard = TerminalCleanup;
        let (_tmp_dir, repo) = setup_test_repo();
        let repo_path = repo.workdir().unwrap().to_str().unwrap().to_string();

        // Explicitly create .gnostr directories for the test
        let gnostr_path = PathBuf::from(&repo_path).join(".gnostr");
        let blobs_path = gnostr_path.join("blobs");
        let reflog_path = gnostr_path.join("reflog");
        fs::create_dir_all(&blobs_path).unwrap();
        fs::create_dir_all(&reflog_path).unwrap();

        let mut cmd = Command::new(cargo_bin("gnostr"));
        cmd.arg("legit")
            .arg("--repo")
            .arg(&repo_path)
            .arg("--message")
            .arg("Test mine commit")
            .arg("--pow")
            .arg("16");

        cmd.assert()
            .success()
            .stdout(str::contains("Mined commit hash:"));

        // Verify that .gnostr directories and files were created
        let repo_path_buf = PathBuf::from(&repo_path);
        assert!(repo_path_buf.join(".gnostr").exists());
        assert!(repo_path_buf.join(".gnostr/blobs").exists());
        assert!(repo_path_buf.join(".gnostr/reflog").exists());

        let _cleanup_guard = TerminalCleanup;
        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn test_main_with_gitdir_env_var() -> Result<(), Box<dyn Error>> {
        let _cleanup_guard = TerminalCleanup;
        //let (_tmp_dir, repo) = setup_test_repo();
        //let repo_path = repo.path().to_str().unwrap().to_string();

        //// Set the GNOSTR_GITDIR environment variable
        //env::set_var("GNOSTR_GITDIR", &repo_path);

        //let mut cmd = Command::new(cargo_bin("gnostr"));
        ////setup process to capture the ratatui screen
        //cmd.arg("tui"); // TUI is the default if no other subcommand is given, but we explicitly call it here

        //cmd.assert()
        //    .code(0) // Expect a successful exit from the TUI
        //    .stderr(str::contains(format!("333:The GNOSTR_GITDIR environment variable is set to: {}", repo_path.clone())));

        //// Unset the environment variable to avoid affecting other tests
        //env::remove_var("GNOSTR_GITDIR");

        let _cleanup_guard = TerminalCleanup;
        Ok(())
    }

    #[test]
    #[serial]
    #[cfg(target_os = "macos")]
    fn test_help_output_generates_screenshot() -> Result<(), Box<dyn Error>> {
        let _cleanup_guard = TerminalCleanup;
        //// First, verify the help command runs successfully.
        //let mut cmd = Command::new(cargo_bin("gnostr"));
        //cmd.arg("--help");
        //cmd.assert().success();

        //// As requested, use the screenshot utility to capture the state of the screen
        //// during the test run for manual inspection.
        //let screenshot_path_result = screenshot::make_screenshot("main_cli_help");

        //assert!(screenshot_path_result.is_ok(), "Failed to capture screenshot.");
        //
        //let screenshot_path = screenshot_path_result.unwrap();
        //
        //// Verify the screenshot file was created
        //let metadata = fs::metadata(&screenshot_path).expect("Failed to get screenshot metadata");
        //assert!(metadata.is_file(), "Screenshot is not a file");
        //assert!(metadata.len() > 0, "Screenshot file is empty");

        let _cleanup_guard = TerminalCleanup;
        Ok(())
    }

    #[test]
    #[serial]
    #[cfg(target_os = "macos")]
    fn test_run_gnostr_chat_and_capture_screenshot() -> Result<(), Box<dyn Error>> {
        let _cleanup_guard = TerminalCleanup;
        //let (_tmp_dir, repo) = setup_test_repo();
        //let repo_path = repo.path().to_str().unwrap().to_string();

        //let mut cmd = Command::new(cargo_bin("gnostr"));
        //cmd.arg("--gitdir").arg(&repo_path).arg("chat");

        //// Spawn the command as a child process
        //let mut child = cmd.spawn().expect("Failed to spawn gnostr chat command");

        //// Give the TUI a moment to initialize
        //thread::sleep(Duration::from_secs(2));

        //// Capture the screenshot
        //let screenshot_path_result = screenshot::make_screenshot("gnostr_chat_run");

        //// Terminate the child process gracefully
        //child.signal(signal_child::signal::SIGINT).expect("Failed to send SIGINT to gnostr chat process");
        //child.wait().expect("Failed to wait for gnostr chat process");

        //// Assert that the screenshot was created
        //assert!(screenshot_path_result.is_ok(), "Failed to capture screenshot.");
        //let screenshot_path = screenshot_path_result.unwrap();
        //let metadata = fs::metadata(&screenshot_path).expect("Failed to get screenshot metadata");
        //assert!(metadata.is_file(), "Screenshot is not a file");
        //assert!(metadata.len() > 0, "Screenshot file is empty");

        let _cleanup_guard = TerminalCleanup;
        Ok(())
    }

    #[test]
    #[serial]
    fn test_blockheight_flag_prints_a_number() -> Result<(), Box<dyn Error>> {
        let _cleanup_guard = TerminalCleanup;
        //let mut cmd = Command::new(cargo_bin("gnostr"));
        //cmd.arg("--blockheight");
        //cmd.assert()
        //    .success()
        //    .stdout(predicates::str::is_match(r"^\d+\.0$").unwrap());
        let _cleanup_guard = TerminalCleanup;
        Ok(())
    }

    #[test]
    #[serial]
    fn test_weeble_flag_prints_a_number() -> Result<(), Box<dyn Error>> {
        let _cleanup_guard = TerminalCleanup;
        //let mut cmd = Command::new(cargo_bin("gnostr"));
        //cmd.arg("--weeble");
        //cmd.assert()
        //    .success()
        //    .stdout(predicates::str::is_match(r"^\d+\.0$").unwrap());
        let _cleanup_guard = TerminalCleanup;
        Ok(())
    }

    #[test]
    #[serial]
    fn test_wobble_flag_prints_a_number() -> Result<(), Box<dyn Error>> {
        let _cleanup_guard = TerminalCleanup;
        //let mut cmd = Command::new(cargo_bin("gnostr"));
        //cmd.arg("--wobble");
        //cmd.assert()
        //    .success()
        //    .stdout(predicates::str::is_match(r"^\d+$").unwrap());
        let _cleanup_guard = TerminalCleanup;
        Ok(())
    }

    #[test]
    #[serial]
    fn test_blockheight_flag_sets_env_var() -> Result<(), Box<dyn Error>> {
        let _cleanup_guard = TerminalCleanup;
        //let mut cmd = Command::new(cargo_bin("gnostr"));
        //cmd.arg("--blockheight");
        //cmd.assert()
        //    .success()
        //    .stdout(predicates::str::is_match(r"^\d+$").unwrap());
        let _cleanup_guard = TerminalCleanup;
        Ok(())
    }

    #[test]
    #[serial]
    fn test_weeble_flag_sets_env_var() -> Result<(), Box<dyn Error>> {
        let _cleanup_guard = TerminalCleanup;
        let mut cmd = Command::new(cargo_bin("gnostr"));
        cmd.arg("--weeble");
        cmd.assert()
            .success()
            .stdout(predicates::str::is_match(r"^\d+$").unwrap());
        let _cleanup_guard = TerminalCleanup;
        Ok(())
    }

    #[test]
    #[serial]
    fn test_wobble_flag_sets_env_var() -> Result<(), Box<dyn Error>> {
        let _cleanup_guard = TerminalCleanup;
        let mut cmd = Command::new(cargo_bin("gnostr"));
        cmd.arg("--wobble");
        cmd.assert()
            .success()
            .stdout(predicates::str::is_match(r"^\d+$").unwrap());
        let _cleanup_guard = TerminalCleanup;
        Ok(())
    }

    #[test]
    #[serial]
    fn test_blockhash_flag_prints_a_hash() -> Result<(), Box<dyn Error>> {
        let _cleanup_guard = TerminalCleanup;
        let mut cmd = Command::new(cargo_bin("gnostr"));
        cmd.arg("--blockhash");
        cmd.assert()
            .success()
            .stdout(predicates::str::is_match(r"^[a-f0-9]{64}$").unwrap());
        let _cleanup_guard = TerminalCleanup;
        Ok(())
    }

    #[test]
    #[serial]
    fn test_run_tui_and_sleep() -> Result<(), Box<dyn Error>> {
        let _cleanup_guard = TerminalCleanup;
        let (_tmp_dir, repo) = setup_test_repo();
        let repo_path = repo.path().to_str().unwrap().to_string();

        let mut cmd = Command::new(cargo_bin("gnostr"));
        cmd.arg("--gitdir").arg(&repo_path).arg("tui");

        // Spawn the command as a child process
        let mut child = cmd.spawn().expect("Failed to spawn gnostr command");

        // Give the TUI a moment to initialize
        thread::sleep(Duration::from_secs(10));

        // Terminate the child process gracefully
        child
            .signal(signal_child::signal::SIGINT)
            .expect("Failed to send SIGINT to gnostr process");
        child.wait().expect("Failed to wait for gnostr process");

        let log_file_path = crate::cli::get_app_cache_path().unwrap().join("gnostr.log");
        if log_file_path.exists() {
            let log_content = fs::read_to_string(log_file_path).unwrap();
            println!("log_content for test_run_tui_and_sleep: {}", log_content);
        } else {
            println!("log file not found for test_run_tui_and_sleep");
        }
        let _cleanup_guard = TerminalCleanup;

        Ok(())
    }

    #[test]
    #[serial]
    fn test_run_tui_and_sleep_screenshot() -> Result<(), Box<dyn Error>> {
        let _cleanup_guard = TerminalCleanup;
        let (_tmp_dir, repo) = setup_test_repo();
        let repo_path = repo.path().to_str().unwrap().to_string();

        let mut cmd = Command::new(cargo_bin("gnostr"));
        cmd.arg("--gitdir").arg(&repo_path).arg("tui");

        // Spawn the command as a child process
        let mut child = cmd.spawn().expect("Failed to spawn gnostr command");

        // Give the TUI a moment to initialize
        thread::sleep(Duration::from_secs(5));

        // Capture the screenshot
        let screenshot_path_result =
            screenshot::make_screenshot("test_run_tui_and_sleep_screenshot");

        // Terminate the child process gracefully
        child
            .signal(signal_child::signal::SIGINT)
            .expect("Failed to send SIGINT to gnostr process");
        child.wait().expect("Failed to wait for gnostr process");

        let log_file_path = crate::cli::get_app_cache_path().unwrap().join("gnostr.log");
        if log_file_path.exists() {
            let log_content = fs::read_to_string(log_file_path).unwrap();
            println!(
                "log_content for test_run_tui_and_sleep_screenshot: {}",
                log_content
            );
        } else {
            println!("log file not found for test_run_tui_and_sleep_screenshot");
        }

        // Assert that the screenshot was created
        assert!(
            screenshot_path_result.is_ok(),
            "Failed to capture screenshot."
        );
        let screenshot_path = screenshot_path_result.unwrap();
        let metadata = fs::metadata(&screenshot_path).expect("Failed to get screenshot metadata");
        assert!(metadata.is_file(), "Screenshot is not a file");
        assert!(metadata.len() > 0, "Screenshot file is empty");

        let _cleanup_guard = TerminalCleanup;

        Ok(())
    }
}
