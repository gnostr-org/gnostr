#![cfg_attr(not(test), warn(clippy::pedantic))]
#![cfg_attr(not(test), warn(clippy::expect_used))]
use crate::cli::NgitCommands;
use crate::sub_commands::fetch;
use crate::sub_commands::init;
use crate::sub_commands::list;
use crate::sub_commands::login;
use crate::sub_commands::pull;
use crate::sub_commands::push;
use crate::sub_commands::send;
use clap::Args;
use nostr_sdk_0_34_0::prelude::*;

use serde::ser::StdError;

#[derive(Args, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct NgitSubCommand {
    #[command(subcommand)]
    command: NgitCommands,
    ///// nsec or hex private key
    #[arg(short, long, global = true)]
    nsec: Option<String>,
    ///// password to decrypt nsec
    #[arg(short, long, global = true)]
    password: Option<String>,
    ///// disable spinner animations
    #[arg(long, default_value_t = true)]
    disable_cli_spinners: bool,
}

pub async fn ngit(sub_command_args: &NgitSubCommand) -> Result<(), Box<dyn StdError>> {
    match &sub_command_args.command {
        NgitCommands::Login(args) => login::launch(args).await?,
        NgitCommands::Init(args) => init::launch(args).await?,
        NgitCommands::Send(args) => send::launch(args, true).await?,
        NgitCommands::List => list::launch().await?,
        NgitCommands::Pull => pull::launch().await?,
        NgitCommands::Push(args) => push::launch(args).await?,
        NgitCommands::Fetch(args) => fetch::launch(args).await?,
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
    async fn test_ngit_login_command() -> Result<(), Box<dyn StdError>> {
        let git_repo = test_utils::git::GitTestRepo::new("main")?;
        let mut p = test_utils::CliTester::new_from_dir(
            &git_repo.dir,
            vec![
                "--disable-cli-spinners",
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

    #[tokio::test]
    async fn test_ngit_init_command() -> Result<(), Box<dyn StdError>> {
        let git_repo = test_utils::git::GitTestRepo::new("main")?;
        git_repo.initial_commit()?;

        let mut p = test_utils::CliTester::new_from_dir(
            &git_repo.dir,
            vec![
                "--disable-cli-spinners",
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
    async fn test_ngit_send_command() -> Result<(), Box<dyn StdError>> {
        let git_repo = test_utils::git::GitTestRepo::new("main")?;
        git_repo.populate()?;

        let mut p = test_utils::CliTester::new_from_dir(
            &git_repo.dir,
            vec![
                "--disable-cli-spinners",
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
    async fn test_ngit_list_command() -> Result<(), Box<dyn StdError>> {
        let git_repo = test_utils::cli_tester_create_proposals()?;

        let mut p = test_utils::CliTester::new_from_dir(
            &git_repo.dir,
            vec![
                "--disable-cli-spinners",
                "list",
            ],
        );

        p.expect_end_eventually()?;
        Ok(())
    }

    #[tokio::test]
    async fn test_ngit_pull_command() -> Result<(), Box<dyn StdError>> {
        let (originating_repo, test_repo) = test_utils::create_proposals_and_repo_with_proposal_pulled_and_checkedout(1)?;
        let mut p = test_utils::CliTester::new_from_dir(
            &test_repo.dir,
            vec![
                "--disable-cli-spinners",
                "--nsec",
                test_utils::TEST_KEY_1_NSEC,
                "--password",
                test_utils::TEST_PASSWORD,
                "pull",
            ],
        );

        p.expect_end_eventually()?;
        p.exit()?;
        Ok(())
    }

    #[tokio::test]
    async fn test_ngit_push_command() -> Result<(), Box<dyn StdError>> {
        let (originating_repo, test_repo) = test_utils::create_proposals_with_first_revised_and_repo_with_unrevised_proposal_checkedout()?;

        let mut p = test_utils::CliTester::new_from_dir(
            &test_repo.dir,
            vec![
                "--disable-cli-spinners",
                "push",
                "--nsec",
                test_utils::TEST_KEY_1_NSEC,
                "--password",
                test_utils::TEST_PASSWORD,
            ],
        );

        p.expect_end_eventually()?;
        Ok(())
    }

    #[tokio::test]
    async fn test_ngit_fetch_command() -> Result<(), Box<dyn StdError>> {
        let git_repo = test_utils::cli_tester_create_proposals()?;

        let mut p = test_utils::CliTester::new_from_dir(
            &git_repo.dir,
            vec![
                "--disable-cli-spinners",
                "fetch",
            ],
        );

        p.expect_end_eventually()?;
        Ok(())
    }
}
