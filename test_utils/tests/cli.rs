use test_utils::CliTester;

#[test]
fn test_help_command() {
    let mut cli_tester = CliTester::new(["--help"]);
    cli_tester
        .expect_eventually("nostr plugin for git")
        .unwrap();
}

#[test]
fn test_version_command() {
    let mut cli_tester = CliTester::new(["--version"]);
    cli_tester
        .expect_eventually("ngit 1.6.0")
        .unwrap();
}
