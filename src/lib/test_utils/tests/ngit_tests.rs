use std::error::Error as StdError;

use gnostr_crawler::processor::BOOTSTRAP_RELAYS;
use serial_test::serial;

use crate::{
    cli::NgitCommands,
    sub_commands::{
        ngit::{NgitSubCommand, ngit},
        query::QuerySubCommand,
    },
    test_utils,
};

// Helper function to create a dummy NgitSubCommand
fn create_dummy_ngit_subcommand(command: NgitCommands) -> NgitSubCommand {
    NgitSubCommand {
        command,
        nsec: None,
        password: None,
        disable_cli_spinners: true,
    }
}

#[tokio::test]
#[serial]
async fn test_ngit_login_command() -> Result<(), Box<dyn StdError>> {
    let git_repo = test_utils::git::GitTestRepo::new("main")?;
    let mut p = test_utils::CliTester::new_from_dir(
        &git_repo.dir,
        vec![
            "--disable-cli_spinners",
            "--nsec",
            test_utils::TEST_KEY_1_NSEC,
            "--password",
            test_utils::TEST_PASSWORD,
            "login",
        ],
    );

    p.expect_end_eventually()?;
    p.exit()?;
    Ok(())
}

#[serial]
fn test_ngit_init_command() -> Result<(), Box<dyn StdError>> {
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
async fn test_ngit_send_command() -> Result<(), Box<dyn StdError>> {
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
async fn test_ngit_list_command() -> Result<(), Box<dyn StdError>> {
    let git_repo = test_utils::cli_tester_create_proposals()?;

    let mut p =
        test_utils::CliTester::new_from_dir(&git_repo.dir, vec!["--disable-cli_spinners", "list"]);

    p.expect_end_eventually()?;
    Ok(())
}

//#[tokio::test]
#[serial]
/* async */
fn test_ngit_pull_command() -> Result<(), Box<dyn StdError>> {
    let (_originating_repo, test_repo) =
        test_utils::create_proposals_and_repo_with_proposal_pulled_and_checkedout(1)?;
    let mut p = test_utils::CliTester::new_from_dir(
        &test_repo.dir,
        vec![
            "pull",
            "--disable-cli-spinners",
            "--nsec",
            test_utils::TEST_KEY_1_NSEC,
            "--password",
            test_utils::TEST_PASSWORD,
        ],
    );

    p.expect_end_eventually()?;
    p.exit()?;
    Ok(())
}

//#[tokio::test]
#[serial]
/* async */
fn test_ngit_push_command() -> Result<(), Box<dyn StdError>> {
    let (_originating_repo, test_repo) =
        test_utils::create_proposals_with_first_revised_and_repo_with_unrevised_proposal_checkedout(
        )?;
    let mut p = test_utils::CliTester::new_from_dir(
        &test_repo.dir,
        vec![
            "push",
            "--disable-cli-spinners",
            "--nsec",
            test_utils::TEST_KEY_1_NSEC,
            "--password",
            test_utils::TEST_PASSWORD,
        ],
    );

    p.expect_end_eventually()?;
    p.exit()?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn test_ngit_fetch_command() -> Result<(), Box<dyn StdError>> {
    let git_repo = test_utils::cli_tester_create_proposals()?;

    let mut p =
        test_utils::CliTester::new_from_dir(&git_repo.dir, vec!["--disable-cli_spinners", "fetch"]);

    p.expect_end_eventually()?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn test_ngit_query_multiple_kinds_with_all_bootstrap_relays() -> Result<(), Box<dyn StdError>>
{
    let kinds_string = "1630,1632,1621,30618,1633,1631,1617,30617".to_string();
    let base_query_args = QuerySubCommand {
        authors: None,
        ids: None,
        limit: Some(1),
        generic: None,
        hashtag: None,
        mentions: None,
        references: None,
        kinds: Some(kinds_string.clone()),
        search: None,
        relay: None, // This will be set in the loop
    };

    for relay_url in BOOTSTRAP_RELAYS.iter() {
        println!(
            "\nTesting ngit query with kinds {} on relay: {}\n",
            kinds_string, relay_url
        );
        let ngit_command = NgitCommands::Query(QuerySubCommand {
            relay: Some(relay_url.clone()),
            ..base_query_args.clone()
        });
        let sub_command_args = create_dummy_ngit_subcommand(ngit_command);
        let result = ngit(&sub_command_args).await;
        assert!(
            result.is_ok(),
            "ngit query with kinds {} failed for relay {}: {:?}",
            kinds_string,
            relay_url,
            result.err()
        );
    }
    Ok(())
}
