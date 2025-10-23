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
    use std::io::Read;
    use gnostr::cli::{setup_logging, get_app_cache_path};

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
    async fn test_gitsh_command_error_output() -> Result<(), Box<dyn Error>> {
        let mut cmd = Command::new(cargo_bin("gnostr"));
        cmd.arg("gitsh");

        cmd.assert()
            .failure()
            .stdout(str::contains("Mock SSH Start Error"))
            .stdout(str::contains("EXAMPLE:server.toml"))
            .stdout(str::contains("check the port in your server.toml is available!"))
            .stdout(str::contains("EXAMPLE:repo.toml"));

        Ok(())
    }
}
