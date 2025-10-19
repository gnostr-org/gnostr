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
    use crate::sub_commands::fetch::FetchArgs;
    use crate::sub_commands::init::InitArgs;
    use crate::sub_commands::login::LoginArgs;
    use crate::sub_commands::push::PushArgs;
    use crate::sub_commands::send::SendArgs;

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
        let git_repo = crate::test_utils::git::GitTestRepo::new("main")?;
        let mut p = crate::test_utils::CliTester::new_from_dir(
            &git_repo.dir,
            vec![
                "login",
                "--disable-cli-spinners",
            ],
        );

        p.expect_password("nsec or hex private key")?
            .succeeds_with(crate::test_utils::TEST_KEY_1_NSEC)?;
        p.expect_password("password to decrypt nsec")?
            .succeeds_with(crate::test_utils::TEST_PASSWORD)?;

        p.expect_end_eventually()?;
        Ok(())
    }

    #[tokio::test]
    async fn test_ngit_init_command() -> Result<(), Box<dyn StdError>> {
        let git_repo = crate::test_utils::git::GitTestRepo::new("main")?;
        git_repo.initial_commit()?;

        let mut p = crate::test_utils::CliTester::new_from_dir(
            &git_repo.dir,
            vec![
                "init",
                "--disable-cli-spinners",
                "--nsec",
                crate::test_utils::TEST_KEY_1_NSEC,
                "--password",
                crate::test_utils::TEST_PASSWORD,
            ],
        );

        p.expect_input("name")?.succeeds_with("test-repo")?;
        p.expect_input("identifier")?.succeeds_with("test-repo")?;
        p.expect_input("description")?.succeeds_with("A test repository")?;
        p.expect_input("clone url (for fetch)")?.succeeds_with("https://github.com/test/test-repo.git")?;
        p.expect_input("web")?.succeeds_with("https://test-repo.com")?;
        p.expect_input("relays")?.succeeds_with("wss://relay.example.com")?;
        p.expect_input("maintainers")?.succeeds_with(crate::test_utils::TEST_KEY_1_NPUB)?;
        p.expect_input("earliest unique commit")?.succeeds_with(&git_repo.git_repo.head()?.peel_to_commit()?.id().to_string())?;

        p.expect_end_eventually()?;
        Ok(())
    }

    #[tokio::test]
    async fn test_ngit_send_command() -> Result<(), Box<dyn StdError>> {
        let git_repo = crate::test_utils::git::GitTestRepo::new("main")?;
        git_repo.populate()?;

        let mut p = crate::test_utils::CliTester::new_from_dir(
            &git_repo.dir,
            vec![
                "send",
                "HEAD~1", // Send the last commit
                "--no-cover-letter",
                "--disable-cli-spinners",
                "--nsec",
                crate::test_utils::TEST_KEY_1_NSEC,
                "--password",
                crate::test_utils::TEST_PASSWORD,
            ],
        );

        p.expect_end_eventually()?;
        Ok(())
    }

    #[tokio::test]
    async fn test_ngit_list_command() -> Result<(), Box<dyn StdError>> {
        let git_repo = crate::test_utils::cli_tester_create_proposals()?;

        let mut p = crate::test_utils::CliTester::new_from_dir(
            &git_repo.dir,
            vec![
                "list",
                "--disable-cli-spinners",
            ],
        );

        p.expect("fetching updates...\r\n")?;
        p.expect_eventually("\r\n")?; // Some updates listed here

        let mut c = p.expect_choice("all proposals", vec![
            format!("\"{}\"", crate::test_utils::PROPOSAL_TITLE_3),
            format!("\"{}\"", crate::test_utils::PROPOSAL_TITLE_2),
            format!("\"{}\"", crate::test_utils::PROPOSAL_TITLE_1),
        ])?;
        c.succeeds_with(0, true, None)?; // Select the first proposal (PROPOSAL_TITLE_3)

        let mut c = p.expect_choice("", vec![
            format!("create and checkout proposal branch (2 ahead 0 behind 'main')"),
            format!("apply to current branch with `git am`"),
            format!("download to ./patches"),
            format!("back"),
        ])?;
        c.succeeds_with(3, false, Some(0))?; // Select "back"

        p.expect_end_eventually()?;
        Ok(())
    }

    #[tokio::test]
    async fn test_ngit_pull_command() -> Result<(), Box<dyn StdError>> {
        let (originating_repo, test_repo) = crate::test_utils::create_proposals_and_repo_with_proposal_pulled_and_checkedout(1)?;

        let mut p = crate::test_utils::CliTester::new_from_dir(
            &test_repo.dir,
            vec![
                "pull",
                "--disable-cli-spinners",
                "--nsec",
                crate::test_utils::TEST_KEY_1_NSEC,
                "--password",
                crate::test_utils::TEST_PASSWORD,
            ],
        );

        p.expect_end_eventually()?;
        Ok(())
    }

    #[tokio::test]
    async fn test_ngit_push_command() -> Result<(), Box<dyn StdError>> {
        let (originating_repo, test_repo) = crate::test_utils::create_proposals_with_first_revised_and_repo_with_unrevised_proposal_checkedout()?;

        let mut p = crate::test_utils::CliTester::new_from_dir(
            &test_repo.dir,
            vec![
                "push",
                "--disable-cli-spinners",
                "--nsec",
                crate::test_utils::TEST_KEY_1_NSEC,
                "--password",
                crate::test_utils::TEST_PASSWORD,
            ],
        );

        p.expect_end_eventually()?;
        Ok(())
    }

    #[tokio::test]
    async fn test_ngit_fetch_command() -> Result<(), Box<dyn StdError>> {
        let git_repo = crate::test_utils::cli_tester_create_proposals()?;

        let mut p = crate::test_utils::CliTester::new_from_dir(
            &git_repo.dir,
            vec![
                "fetch",
                "--disable-cli-spinners",
            ],
        );

        p.expect_end_eventually()?;
        Ok(())
    }
}
