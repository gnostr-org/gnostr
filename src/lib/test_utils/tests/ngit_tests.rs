use serial_test::serial;

use crate::test_utils;

#[tokio::test]
#[serial]
async fn test_ngit_login_command() -> anyhow::Result<()> {
    let git_repo = test_utils::git::GitTestRepo::new("main")?;
    let mut p = test_utils::CliTester::new_from_dir(
        &git_repo.dir,
        vec![
            "--disable-cli_spinners",
            "--nsec",
            test_utils::TEST_KEY_1_NSEC,
            "--password",
            test_utils::TEST_PASSWORD,
            "account",
            "login",
        ],
    );

    p.expect_end_eventually()?;
    p.exit()?;
    Ok(())
}

#[serial]
fn test_ngit_init_command() -> anyhow::Result<()> {
    let mut git_repo = test_utils::git::GitTestRepo::new("main")?;
    git_repo.initial_commit()?;

    let mut p = test_utils::CliTester::new_from_dir(
        &git_repo.dir,
        vec![
            "--disable-cli_spinners",
            "--nsec",
            test_utils::TEST_KEY_1_NSEC,
            "--password",
            test_utils::TEST_PASSWORD,
            "init",
        ],
    );

    p.expect_end_eventually()?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn test_ngit_send_command() -> anyhow::Result<()> {
    let mut git_repo = test_utils::git::GitTestRepo::new("main")?;
    git_repo.populate()?;

    let mut p = test_utils::CliTester::new_from_dir(
        &git_repo.dir,
        vec![
            "--disable-cli_spinners",
            "--nsec",
            test_utils::TEST_KEY_1_NSEC,
            "--password",
            test_utils::TEST_PASSWORD,
            "send",
            "--no-cover-letter",
            "HEAD~1", // Send the last commit
        ],
    );

    p.expect_end_eventually()?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn test_ngit_list_command() -> anyhow::Result<()> {
    let git_repo = test_utils::cli_tester_create_proposals()?;

    let mut p =
        test_utils::CliTester::new_from_dir(&git_repo.dir, vec!["--disable-cli_spinners", "list"]);

    p.expect_end_eventually()?;
    Ok(())
}
