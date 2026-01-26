use std::error::Error;

use serial_test::serial;

use crate::test_utils::CliTester;

#[test]
#[serial]
fn test_nip34_repo_announcement() -> Result<(), Box<dyn Error>> {
    let mut p = CliTester::new([
        "nip34",
        "repo-announcement",
        "--name",
        "my-repo",
        "--description",
        "My awesome repo",
        "--clone-url",
        "https://github.com/example/repo",
        "--web-url",
        "https://example.com/repo",
        "--relays",
        "wss://relay.example.com",
        "--maintainers",
        "npub1...",
        "--root-commit",
        "abcdef...",
        "--hashtags",
        "cool-project",
    ]);
    p.expect_end_eventually()?;
    Ok(())
}

#[test]
#[serial]
fn test_nip34_repo_state() -> Result<(), Box<dyn Error>> {
    let mut p = CliTester::new([
        "nip34",
        "repo-state",
        "--identifier",
        "my-repo",
        "--refs",
        "refs/heads/main|abcdef...",
    ]);
    p.expect_end_eventually()?;
    Ok(())
}

#[test]
#[serial]
fn test_nip34_patch() -> Result<(), Box<dyn Error>> {
    let mut p = CliTester::new([
        "nip34",
        "patch",
        "--repo",
        "my-repo",
        "--commit",
        "abcdef...",
        "--parent-commit",
        "123456...",
        "--content",
        "--- a/main.rs\n+++ b/main.rs\n@@ -1 +1 @@\n-let x = 5;\n+let x = 10;",
    ]);
    p.expect_end_eventually()?;
    Ok(())
}

#[test]
#[serial]
fn test_nip34_pull_request() -> Result<(), Box<dyn Error>> {
    let mut p = CliTester::new([
        "nip34",
        "pull-request",
        "--repo",
        "my-repo",
        "--subject",
        "Implement new feature X",
        "--branch-name",
        "feature/new-x",
        "--merge-base",
        "abcdef...",
    ]);
    p.expect_end_eventually()?;
    Ok(())
}

#[test]
#[serial]
fn test_nip34_issue() -> Result<(), Box<dyn Error>> {
    let mut p = CliTester::new([
        "nip34",
        "issue",
        "--repo",
        "my-repo",
        "--subject",
        "Login button returns 500 error",
        "--content",
        "The login button is not working on the staging environment.",
    ]);
    p.expect_end_eventually()?;
    Ok(())
}

#[test]
#[serial]
fn test_nip34_status_open() -> Result<(), Box<dyn Error>> {
    let mut p = CliTester::new([
        "nip34",
        "status",
        "open",
        "--event-id",
        "abcdef...",
        "--repo",
        "my-repo",
    ]);
    p.expect_end_eventually()?;
    Ok(())
}

#[test]
#[serial]
fn test_nip34_status_applied() -> Result<(), Box<dyn Error>> {
    let mut p = CliTester::new([
        "nip34",
        "status",
        "applied",
        "--event-id",
        "abcdef...",
        "--repo",
        "my-repo",
        "--applied-as-commits",
        "123456...",
    ]);
    p.expect_end_eventually()?;
    Ok(())
}

#[test]
#[serial]
fn test_nip34_status_closed() -> Result<(), Box<dyn Error>> {
    let mut p = CliTester::new([
        "nip34",
        "status",
        "closed",
        "--event-id",
        "abcdef...",
        "--repo",
        "my-repo",
    ]);
    p.expect_end_eventually()?;
    Ok(())
}

#[test]
#[serial]
fn test_nip34_status_draft() -> Result<(), Box<dyn Error>> {
    let mut p = CliTester::new([
        "nip34",
        "status",
        "draft",
        "--event-id",
        "abcdef...",
        "--repo",
        "my-repo",
    ]);
    p.expect_end_eventually()?;
    Ok(())
}

#[test]
#[serial]
fn test_nip34_repo_announcement_alternate() -> Result<(), Box<dyn Error>> {
    let mut p = CliTester::new([
        "nip34",
        "repo-announcement",
        "--name",
        "alternate-repo",
        "--description",
        "An alternate repo for testing.",
        "--clone-url",
        "https://github.com/example/alternate",
        "--relays",
        "wss://relay.alternate.com",
        "wss://relay.v2.alternate.com",
        "--maintainers",
        "npub1...",
        "--root-commit",
        "fedcba...",
    ]);
    p.expect_end_eventually()?;
    Ok(())
}
