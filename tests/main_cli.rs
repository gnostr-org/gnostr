use anyhow::Result;
use assert_cmd::cargo::cargo_bin;
use assert_cmd::assert::OutputAssertExt;
use predicates::prelude::PredicateBooleanExt;
use predicates::str;
use std::process::Command;
use std::env;

#[test]
fn test_hash_command_prints_hash_and_exits() -> Result<()> {
    let input_string = "test_string";
    let expected_hash = "4b641e9a923d1ea57e18fe41dcb543e2c4005c41ff210864a710b0fbb2654c11"; // SHA256 of "test_string"

    let mut cmd = Command::new(cargo_bin("gnostr"));
    cmd.arg("--hash").arg(input_string);

    cmd.assert()
        .success()
        .stdout(str::diff(expected_hash))
        .stderr(str::is_empty());

    Ok(())
}

#[test]
fn test_hash_command_with_debug_flag_prints_hash_and_exits() -> Result<()> {
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
        .stdout(str::diff(actual_expected_hash.clone()))
        .stderr(str::contains("40:arg=\"--debug\"").and(str::contains("40:arg=\"--hash\"")).and(str::contains(format!("40:arg=\" {}", input_string))));

    Ok(())
}

/*
#[test]
fn test_gitdir_argument_sets_env_var() -> Result<()> {
    let test_git_dir = "/tmp/test_repo_path";

    // Ensure the environment variable is not set before the test
    env::remove_var("GNOSTR_GITDIR");

    let mut cmd = Command::new(cargo_bin("gnostr"));
    cmd.arg("--gitdir").arg(test_git_dir).arg("tui"); // 'tui' is a subcommand that will be executed

    cmd.assert()
        .success()
        .stderr(str::contains(format!("63:The --gitdir value is: {}", test_git_dir)));

    Ok(())
}
*/

#[test]
fn test_logging_flags_set_level_filter() -> Result<()> {
    // Test --debug
    let mut cmd_debug = Command::new(cargo_bin("gnostr"));
    cmd_debug.arg("--debug").arg("--hash").arg("test");
    cmd_debug.assert()
        .success()
        .stderr(str::contains("level=DEBUG"));

    // Test --trace
    let mut cmd_trace = Command::new(cargo_bin("gnostr"));
    cmd_trace.arg("--trace").arg("--hash").arg("test");
    cmd_trace.assert()
        .success()
        .stderr(str::contains("level=TRACE"));

    // Test --info
    let mut cmd_info = Command::new(cargo_bin("gnostr"));
    cmd_info.arg("--info").arg("--hash").arg("test");
    cmd_info.assert()
        .success()
        .stderr(str::contains("level=INFO"));

    // Test --warn
    let mut cmd_warn = Command::new(cargo_bin("gnostr"));
    cmd_warn.arg("--warn").arg("--hash").arg("test");
    cmd_warn.assert()
        .success()
        .stderr(str::contains("level=WARN"));

    // Test default (OFF)
    let mut cmd_default = Command::new(cargo_bin("gnostr"));
    cmd_default.arg("--hash").arg("test");
    cmd_default.assert()
        .success()
        .stderr(str::is_empty().not()); // Should not contain level=OFF explicitly, but should not contain DEBUG/TRACE/INFO/WARN

    Ok(())
}

// High-level test for subcommand dispatching
#[test]
fn test_subcommand_dispatch_tui_help() -> Result<()> {
    let mut cmd = Command::new(cargo_bin("gnostr"));
    cmd.arg("tui").arg("--help");

    cmd.assert()
        .success()
        .stdout(str::contains("Gnostr sub commands"))
        .stdout(str::contains("OPTIONS:"))
        .stderr(str::is_empty());

    Ok(())
}

#[test]
fn test_subcommand_dispatch_chat_help() -> Result<()> {
    let mut cmd = Command::new(cargo_bin("gnostr"));
    cmd.arg("chat").arg("--help");

    cmd.assert()
        .success()
        .stdout(str::contains("Gnostr sub commands"))
        .stdout(str::contains("OPTIONS:"))
        .stderr(str::is_empty());

    Ok(())
}
