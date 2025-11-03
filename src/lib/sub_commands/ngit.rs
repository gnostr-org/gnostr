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
use crate::sub_commands::query;
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
    #[arg(long, action)]
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
        NgitCommands::Query(args) => query::launch(args).await?,
    }
    Ok(())
}

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::sub_commands::query::QuerySubCommand;
        use gnostr_crawler::processor::BOOTSTRAP_RELAYS;
        use anyhow::Result;
        use test_utils::*;
        use futures::join;
        use test_utils::relay::{shutdown_relay, ListenerReqFunc, Relay};
        use std::fs;
        use std::io::Read;
        use std::io::Write;

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

        #[tokio::test]
        async fn test_ngit_init_command() -> Result<(), Box<dyn StdError>> {
            let git_repo = test_utils::git::GitTestRepo::new("main")?;
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
        async fn test_ngit_send_command() -> Result<(), Box<dyn StdError>> {
            let git_repo = test_utils::git::GitTestRepo::new("main")?;
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

        //#[tokio::test]
        /*async */fn test_ngit_pull_command() -> Result<(), Box<dyn StdError>> {
            let (originating_repo, test_repo) = test_utils::create_proposals_and_repo_with_proposal_pulled_and_checkedout(1)?;
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
        /*async */fn test_ngit_push_command() -> Result<(), Box<dyn StdError>> {
            let (originating_repo, test_repo) = test_utils::create_proposals_with_first_revised_and_repo_with_unrevised_proposal_checkedout()?;
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

        #[tokio::test]
		#[ignore]
        async fn test_ngit_query_multiple_kinds_with_all_bootstrap_relays() -> Result<(), Box<dyn StdError>> {
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
                println!("\nTesting ngit query with kinds {} on relay: {}\n", kinds_string, relay_url);
                let ngit_command = NgitCommands::Query(QuerySubCommand {
                    relay: Some(relay_url.clone()),
                    ..base_query_args.clone()
                });
                let sub_command_args = create_dummy_ngit_subcommand(ngit_command);
                let result = ngit(&sub_command_args).await;
                assert!(result.is_ok(), "ngit query with kinds {} failed for relay {}: {:?}", kinds_string, relay_url, result.err());
            }
            Ok(())
        }

        // Start of tests from tests/ngit/init.rs
        fn expect_msgs_first_init(p: &mut CliTester) -> Result<()> {
            p.expect("searching for profile...\r\n")?;
            p.expect("logged in as fred\r\n")?;
            // // p.expect("searching for existing claims on
            // repository...\r\n")?;
            p.expect("publishing repostory reference...\r\n")?;
            Ok(())
        }

        fn expect_msgs_after_init(p: &mut CliTester) -> Result<()> {
            p.expect_after_whitespace("maintainers.yaml created. commit and push.\r\n")?;
            p.expect(
                "this optional file helps in identifying who the maintainers are over time through the commit history\r\n",
            )?;
            Ok(())
        }

        fn get_cli_args_init() -> Vec<&'static str> {
            vec![
        "--nsec",
        TEST_KEY_1_NSEC,
        "--password",
        TEST_PASSWORD,
        "--disable-cli-spinners",
        "init",
        "--title",
        "example-name",
        "--identifier",
        "example-identifier",
        "--description",
        "example-description",
        "--web",
        "https://exampleproject.xyz",
        "https://gitworkshop.dev/123",
        "--relays",
        "ws://localhost:8055",
        "ws://localhost:8056",
        "--clone-url",
        "https://git.myhosting.com/my-repo.git",
        "--earliest-unique-commit",
        "9ee507fc4357d7ee16a5d8901bedcd103f23c17d",
        "--other-maintainers",
        TEST_KEY_1_NPUB,
            ]
        }

        mod when_repo_not_previously_claimed {
            use super::*;
            use test_utils::git::GitTestRepo;
            use nostr_sdk_0_34_0::Kind;

            mod when_repo_relays_specified_as_arguments {
                use super::*;
                use futures::join;
                use test_utils::relay::{Relay, ListenerReqFunc, shutdown_relay};

                #[cfg(feature = "expensive_tests")]
                fn prep_git_repo() -> Result<GitTestRepo> {
                    let test_repo = GitTestRepo::without_repo_in_git_config();
                    test_repo.populate()?;
                    test_repo.add_remote("origin", "https://localhost:1000")?;
                    Ok(test_repo)
                }

                #[cfg(feature = "expensive_tests")]
                fn cli_tester_init(git_repo: &GitTestRepo) -> CliTester {
                    CliTester::new_from_dir(&git_repo.dir, get_cli_args_init())
                }

                #[cfg(feature = "expensive_tests")]
                async fn prep_run_init() -> Result<(
                    Relay<'static>,
                    Relay<'static>,
                    Relay<'static>,
                    Relay<'static>,
                    Relay<'static>,
                    Relay<'static>,
                ), anyhow::Error> {
                    let git_repo = prep_git_repo()?;
                    // fallback (51,52) user write (53, 55) repo (55, 56)
                    // blaster (57)
                    let (mut r51, mut r52, mut r53, mut r55, mut r56, mut r57) = (
                        Relay::new(
                            8051,
                            None,
                            Some(&|relay, client_id, subscription_id, _| -> Result<(), anyhow::Error> {
                                relay.respond_events(
                                    client_id,
                                    &subscription_id,
                                    &vec![
                                        generate_test_key_1_metadata_event("fred"),
                                        generate_test_key_1_relay_list_event(),
                                    ],
                                )?;
                                Ok(())
                            }),
                        ),
                        Relay::new(8052, None, None),
                        Relay::new(8053, None, None),
                        Relay::new(8055, None, None),
                        Relay::new(8056, None, None),
                        Relay::new(8057, None, None),
                    );

                    // // check relay had the right number of events
                    let cli_tester_handle = std::thread::spawn(move || -> Result<(), anyhow::Error> {
                        let mut p = cli_tester_init(&git_repo);
                        p.expect_end_eventually()?;
                        for p in [51, 52, 53, 55, 56, 57] {
                            relay::shutdown_relay(8000 + p)?;
                        }
                        Ok(())
                    });

                    // launch relay
                    let _ = join!(
                        r51.listen_until_close(),
                        r52.listen_until_close(),
                        r53.listen_until_close(),
                        r55.listen_until_close(),
                        r56.listen_until_close(),
                        r57.listen_until_close(),
                    );
                    cli_tester_handle.join().unwrap()?;
                    Ok((r51, r52, r53, r55, r56, r57))
                }

                mod sent_to_correct_relays {

                    #[tokio::test]
                    #[serial]
                    #[cfg(feature = "expensive_tests")]
                    async fn only_1_repository_kind_event_sent_to_user_relays() -> Result<(), anyhow::Error> {
                        let (_, _, r53, r55, _, _) = prep_run_init().await?;
                        for relay in [&r53, &r55] {
                            assert_eq!(
                                relay
                                    .events
                                    .iter()
                                    .filter(|e| e.kind.eq(&Kind::GitRepoAnnouncement))
                                    .count(),
                                1,
                            );
                        }
                        Ok(())
                    }

                    #[tokio::test]
                    #[serial]
                    #[cfg(feature = "expensive_tests")]
                    async fn only_1_repository_kind_event_sent_to_specified_repo_relays() -> Result<(), anyhow::Error> {
                        let (_, _, _, r55, r56, _) = prep_run_init().await?;
                        for relay in [&r55, &r56] {
                            assert_eq!(
                                relay
                                    .events
                                    .iter()
                                    .filter(|e| e.kind.eq(&Kind::GitRepoAnnouncement))
                                    .count(),
                                1,
                            );
                        }
                        Ok(())
                    }

                    #[tokio::test]
                    #[serial]
                    #[cfg(feature = "expensive_tests")]
                    async fn only_1_repository_kind_event_sent_to_fallback_relays() -> Result<(), anyhow::Error> {
                        let (r51, r52, _, _, _, _) = prep_run_init().await?;
                        for relay in [&r51, &r52] {
                            assert_eq!(
                                relay
                                    .events
                                    .iter()
                                    .filter(|e| e.kind.eq(&Kind::GitRepoAnnouncement))
                                    .count(),
                                1,
                            );
                        }
                        Ok(())
                    }

                    #[tokio::test]
                    #[serial]
                    #[cfg(feature = "expensive_tests")]
                    async fn only_1_repository_kind_event_sent_to_blaster_relays() -> Result<(), anyhow::Error> {
                        let (_, _, _, _, _, r57) = prep_run_init().await?;
                        assert_eq!(
                            r57.events
                                .iter()
                                .filter(|e| e.kind.eq(&Kind::GitRepoAnnouncement))
                                .count(),
                            1,
                        );
                        Ok(())
                    }
                }

                mod yaml_file {
                    use super::*;
                    use std::fs;
                    use std::io::Read;
                    use std::io::Write;

                    #[cfg(feature = "expensive_tests")]
                    async fn async_run_test() -> Result<(), anyhow::Error> {
                        let git_repo = prep_git_repo()?;
                        // fallback (51,52) user write (53, 55) repo (55, 56)
                        // blaster (57)
                        let (mut r51, mut r52, mut r53, mut r55, mut r56, mut r57) = (
                            Relay::new(
                                8051,
                                None,
                                Some(&|relay, client_id, subscription_id, _| -> Result<(), anyhow::Error> {
                                    relay.respond_events(
                                        client_id,
                                        &subscription_id,
                                        &vec![
                                            generate_test_key_1_metadata_event("fred"),
                                            generate_test_key_1_relay_list_event(),
                                        ],
                                    )?;
                                    Ok(())
                                }),
                            ),
                            Relay::new(8052, None, None),
                            Relay::new(8053, None, None),
                            Relay::new(8055, None, None),
                            Relay::new(8056, None, None),
                            Relay::new(8057, None, None),
                        );

                        // // check relay had the right number of events
                        let cli_tester_handle = std::thread::spawn(move || -> Result<(), anyhow::Error> {
                            let mut p = cli_tester_init(&git_repo);
                            p.expect_end_eventually()?;

                            let yaml_path = git_repo.dir.join("maintainers.yaml");

                            assert!(yaml_path.exists());

                            let mut file = fs::File::open(yaml_path).expect("no such file");
                            let mut file_contents = "".to_string();
                            let _ = file.read_to_string(&mut file_contents);

                            for p in [51, 52, 53, 55, 56, 57] {
                                relay::shutdown_relay(8000 + p)?;
                            }
                            assert_eq!(
                                file_contents,
                                format!(
                                    "\
                                identifier: example-identifier\n\
                                maintainers:\n\
                                - {TEST_KEY_1_NPUB}\n\
                                relays:\n\
                                - ws://localhost:8055\n\
                                - ws://localhost:8056\n\
                                "
                                ),
                            );
                            Ok(())
                        });

                        // launch relay
                        let _ = join!(
                            r51.listen_until_close(),
                            r52.listen_until_close(),
                            r53.listen_until_close(),
                            r55.listen_until_close(),
                            r56.listen_until_close(),
                            r57.listen_until_close(),
                        );
                        cli_tester_handle.join().unwrap()?;
                        Ok(())
                    }

                    #[tokio::test]
                    #[serial]
                    #[cfg(feature = "expensive_tests")]
                    async fn contains_identifier_maintainers_and_relays() -> Result<(), anyhow::Error> {
                        async_run_test().await
                    }
                    mod updates_existing_with_missing_identifier {
                        use super::*;

                        #[cfg(feature = "expensive_tests")]
                        async fn async_run_test() -> Result<(), anyhow::Error> {
                            let git_repo = prep_git_repo()?;
                            // fallback (51,52) user write (53, 55) repo (55,
                            // 56) blaster (57)
                            let (mut r51, mut r52, mut r53, mut r55, mut r56, mut r57) = (
                                Relay::new(
                                    8051,
                                    None,
                                    Some(&|relay, client_id, subscription_id, _| -> Result<(), anyhow::Error> {
                                        relay.respond_events(
                                            client_id,
                                            &subscription_id,
                                            &vec![
                                                generate_test_key_1_metadata_event("fred"),
                                                generate_test_key_1_relay_list_event(),
                                            ],
                                        )?;
                                        Ok(())
                                    }),
                                ),
                                Relay::new(8052, None, None),
                                Relay::new(8053, None, None),
                                Relay::new(8055, None, None),
                                Relay::new(8056, None, None),
                                Relay::new(8057, None, None),
                            );

                            // // check relay had the right number of events
                            let cli_tester_handle = std::thread::spawn(move || -> Result<(), anyhow::Error> {
                                let yaml_path = git_repo.dir.join("maintainers.yaml");
                                let mut file = std::fs::File::create(&yaml_path)
                                    .expect("cannot create maintainers.yaml file");
                                write!(
                                    file,
                                    "\
                                    maintainers:\n\
                                    - {TEST_KEY_1_NPUB}\n\
                                    relays:\n\
                                    - ws://localhost:8055\n\
                                    - ws://localhost:8056\n\
                                    "
                                )?;

                                let mut p = cli_tester_init(&git_repo);
                                p.expect_end_eventually()?;

                                assert!(yaml_path.exists());

                                let mut file = fs::File::open(yaml_path).expect("no such file");
                                let mut file_contents = "".to_string();
                                let _ = file.read_to_string(&mut file_contents);

                                for p in [51, 52, 53, 55, 56, 57] {
                                    relay::shutdown_relay(8000 + p)?;
                                }
                                assert_eq!(
                                    file_contents,
                                    format!(
                                        "\
                                    identifier: example-identifier\n\
                                    maintainers:\n\
                                    - {TEST_KEY_1_NPUB}\n\
                                    relays:\n\
                                    - ws://localhost:8055\n\
                                    - ws://localhost:8056\n\
                                    "
                                    ),
                                );
                                Ok(())
                            });

                            // launch relay
                            let _ = join!(
                                r51.listen_until_close(),
                                r52.listen_until_close(),
                                r53.listen_until_close(),
                                r55.listen_until_close(),
                                r56.listen_until_close(),
                                r57.listen_until_close(),
                            );
                            cli_tester_handle.join().unwrap()?;
                            Ok(())
                        }

                        #[tokio::test]
                        #[serial]
                        #[cfg(feature = "expensive_tests")]
                        async fn adds_missing_identifier() -> Result<(), anyhow::Error> {
                            async_run_test().await
                        }
                    }
                }

                mod git_config_updated {
                    use super::*;

                    #[cfg(feature = "expensive_tests")]
                    async fn async_run_test() -> Result<(), anyhow::Error> {
                        let git_repo = prep_git_repo()?;
                        // fallback (51,52) user write (53, 55) repo (55, 56)
                        // blaster (57)
                        let (mut r51, mut r52, mut r53, mut r55, mut r56, mut r57) = (
                            Relay::new(
                                8051,
                                None,
                                Some(&|relay, client_id, subscription_id, _| -> Result<(), anyhow::Error> {
                                    relay.respond_events(
                                        client_id,
                                        &subscription_id,
                                        &vec![
                                            generate_test_key_1_metadata_event("fred"),
                                            generate_test_key_1_relay_list_event(),
                                        ],
                                    )?;
                                    Ok(())
                                }),
                            ),
                            Relay::new(8052, None, None),
                            Relay::new(8053, None, None),
                            Relay::new(8055, None, None),
                            Relay::new(8056, None, None),
                            Relay::new(8057, None, None),
                        );

                        // // check relay had the right number of events
                        let cli_tester_handle = std::thread::spawn(move || -> Result<(), anyhow::Error> {
                            let mut p = cli_tester_init(&git_repo);
                            p.expect_end_eventually()?;
                            for p in [51, 52, 53, 55, 56, 57] {
                                relay::shutdown_relay(8000 + p)?;
                            }
                            assert_eq!(
                                git_repo
                                    .git_repo
                                    .config()?
                                    .get_entry("nostr.repo")?
                                    .value()
                                    .unwrap(),
                                Coordinate {
                                    kind: nostr_sdk_0_34_0::Kind::GitRepoAnnouncement,
                                    identifier: "example-identifier".to_string(),
                                    public_key: TEST_KEY_1_KEYS.public_key(),
                                    relays: vec![],
                                }
                                .to_bech32()?,
                            );

                            Ok(())
                        });

                        // launch relay
                        let _ = join!(
                            r51.listen_until_close(),
                            r52.listen_until_close(),
                            r53.listen_until_close(),
                            r55.listen_until_close(),
                            r56.listen_until_close(),
                            r57.listen_until_close(),
                        );
                        cli_tester_handle.join().unwrap()?;
                        Ok(())
                    }

                    #[tokio::test]
                    #[serial]
                    #[cfg(feature = "expensive_tests")]
                    async fn with_nostr_repo_set_to_user_and_identifer_naddr() -> Result<(), anyhow::Error> {
                        async_run_test().await?;
                        Ok(())
                    }
                }

                mod tags_as_specified_in_args {
                    use super::*;

                    #[tokio::test]
                    #[serial]
                    #[cfg(feature = "expensive_tests")]
                    async fn d_replaceable_event_identifier() -> Result<(), anyhow::Error> {
                        let (_, _, r53, r55, r56, r57) = prep_run_init().await?;
                        for relay in [&r53, &r55, &r56, &r57] {
                            let event: &nostr_0_34_1::Event = relay
                                .events
                                .iter()
                                .find(|e| e.kind.eq(&Kind::GitRepoAnnouncement))
                                .unwrap();

                            assert!(event.tags.iter().any(|t| {
                                t.as_vec()[0].eq("d") && t.as_vec()[1].eq("example-identifier")
                            }));
                        }
                        Ok(())
                    }

                    #[tokio::test]
                    #[serial]
                    #[cfg(feature = "expensive_tests")]
                    async fn earliest_unique_commit_as_reference_with_euc_marker() -> Result<(), anyhow::Error> {
                        let (_, _, r53, r55, r56, r57) = prep_run_init().await?;
                        for relay in [&r53, &r55, &r56, &r57] {
                            let event: &nostr_0_34_1::Event = relay
                                .events
                                .iter()
                                .find(|e| e.kind.eq(&Kind::GitRepoAnnouncement))
                                .unwrap();

                            assert!(event.tags.iter().any(|t| {
                                t.as_vec()[0].eq("r")
                                    && t.as_vec()[1].eq("9ee507fc4357d7ee16a5d8901bedcd103f23c17d")
                                    && t.as_vec()[2].eq("euc")
                            }));
                        }
                        Ok(())
                    }

                    #[tokio::test]
                    #[serial]
                    #[cfg(feature = "expensive_tests")]
                    async fn name() -> Result<(), anyhow::Error> {
                        let (_, _, r53, r55, r56, r57) = prep_run_init().await?;
                        for relay in [&r53, &r55, &r56, &r57] {
                            let event: &nostr_0_34_1::Event = relay
                                .events
                                .iter()
                                .find(|e| e.kind.eq(&Kind::GitRepoAnnouncement))
                                .unwrap();

                            assert!(event
                                .tags
                                .iter()
                                .any(|t| t.as_vec()[0].eq("name") && t.as_vec()[1].eq("example-name")));
                        }
                        Ok(())
                    }

                    #[tokio::test]
                    #[serial]
                    #[cfg(feature = "expensive_tests")]
                    async fn alt() -> Result<(), anyhow::Error> {
                        let (_, _, r53, r55, r56, r57) = prep_run_init().await?;
                        for relay in [&r53, &r55, &r56, &r57] {
                            let event: &nostr_0_34_1::Event = relay
                                .events
                                .iter()
                                .find(|e| e.kind.eq(&Kind::GitRepoAnnouncement))
                                .unwrap();

                            assert!(event.tags.iter().any(|t| {
                                t.as_vec()[0].eq("alt") && t.as_vec()[1].eq("git repository: example-name")
                            }));
                        }
                        Ok(())
                    }

                    #[tokio::test]
                    #[serial]
                    #[cfg(feature = "expensive_tests")]
                    async fn description() -> Result<(), anyhow::Error> {
                        let (_, _, r53, r55, r56, r57) = prep_run_init().await?;
                        for relay in [&r53, &r55, &r56, &r57] {
                            let event: &nostr_0_34_1::Event = relay
                                .events
                                .iter()
                                .find(|e| e.kind.eq(&Kind::GitRepoAnnouncement))
                                .unwrap();

                            assert!(event.tags.iter().any(|t| {
                                t.as_vec()[0].eq("description") && t.as_vec()[1].eq("example-description")
                            }));
                        }
                        Ok(())
                    }

                    #[tokio::test]
                    #[serial]
                    #[cfg(feature = "expensive_tests")]
                    async fn git_server() -> Result<(), anyhow::Error> {
                        let (_, _, r53, r55, r56, r57) = prep_run_init().await?;
                        for relay in [&r53, &r55, &r56, &r57] {
                            let event: &nostr_0_34_1::Event = relay
                                .events
                                .iter()
                                .find(|e| e.kind.eq(&Kind::GitRepoAnnouncement))
                                .unwrap();

                            assert!(
                                event.tags.iter().any(|t| {
                                    t.as_vec()[0].eq("clone")
                                        && t.as_vec()[1].eq("https://git.myhosting.com/my-repo.git")
                                })
                            );
                        }
                        Ok(())
                    }

                    #[tokio::test]
                    #[serial]
                    #[cfg(feature = "expensive_tests")]
                    async fn relays() -> Result<(), anyhow::Error> {
                        let (_, _, r53, r55, r56, r57) = prep_run_init().await?;
                        for relay in [&r53, &r55, &r56, &r57] {
                            let event: &nostr_0_34_1::Event = relay
                                .events
                                .iter()
                                .find(|e| e.kind.eq(&Kind::GitRepoAnnouncement))
                                .unwrap();
                            let relays_tag = event
                                .tags
                                .iter()
                                .find(|t| t.as_vec()[0].eq("relays"))
                                .unwrap()
                                .as_vec();
                            assert_eq!(relays_tag[1], "ws://localhost:8055",);
                            assert_eq!(relays_tag[2], "ws://localhost:8056",);
                        }
                        Ok(())
                    }

                    #[tokio::test]
                    #[serial]
                    #[cfg(feature = "expensive_tests")]
                    async fn web() -> Result<(), anyhow::Error> {
                        let (_, _, r53, r55, r56, r57) = prep_run_init().await?;
                        for relay in [&r53, &r55, &r56, &r57] {
                            let event: &nostr_0_34_1::Event = relay
                                .events
                                .iter()
                                .find(|e| e.kind.eq(&Kind::GitRepoAnnouncement))
                                .unwrap();
                            let web_tag = event
                                .tags
                                .iter()
                                .find(|t| t.as_vec()[0].eq("web"))
                                .unwrap()
                                .as_vec();
                            assert_eq!(web_tag[1], "https://exampleproject.xyz",);
                            assert_eq!(web_tag[2], "https://gitworkshop.dev/123",);
                        }
                        Ok(())
                    }

                    #[tokio::test]
                    #[serial]
                    #[cfg(feature = "expensive_tests")]
                    async fn maintainers() -> Result<(), anyhow::Error> {
                        let (_, _, r53, r55, r56, r57) = prep_run_init().await?;
                        for relay in [&r53, &r55, &r56, &r57] {
                            let event: &nostr_0_34_1::Event = relay
                                .events
                                .iter()
                                .find(|e| e.kind.eq(&Kind::GitRepoAnnouncement))
                                .unwrap();
                            let maintainers_tag = event
                                .tags
                                .iter()
                                .find(|t| t.as_vec()[0].eq("maintainers"))
                                .unwrap()
                                .as_vec();
                            assert_eq!(maintainers_tag[1], TEST_KEY_1_KEYS.public_key().to_string());
                        }
                        Ok(())
                    }
                }

                mod cli_ouput {
                    use super::*;

                    #[tokio::test]
                    #[serial]
                    #[cfg(feature = "expensive_tests")]
                    async fn check_cli_output() -> Result<(), anyhow::Error> {
                        let git_repo = prep_git_repo()?;

                        // fallback (51,52) user write (53, 55) repo (55, 56)
                        // blaster (57)
                        let (mut r51, mut r52, mut r53, mut r55, mut r56, mut r57) = (
                            Relay::new(
                                8051,
                                None,
                                Some(&|relay, client_id, subscription_id, _| -> Result<(), anyhow::Error> {
                                    relay.respond_events(
                                        client_id,
                                        &subscription_id,
                                        &vec![
                                            generate_test_key_1_metadata_event("fred"),
                                            generate_test_key_1_relay_list_event(),
                                        ],
                                    )?;
                                    Ok(())
                                }),
                            ),
                            Relay::new(8052, None, None),
                            Relay::new(8053, None, None),
                            Relay::new(8055, None, None),
                            Relay::new(8056, None, None),
                            Relay::new(8057, None, None),
                        );

                        // // check relay had the right number of events
                        let cli_tester_handle = std::thread::spawn(move || -> Result<(), anyhow::Error> {
                            let mut p = cli_tester_init(&git_repo);
                            expect_msgs_first_init(&mut p)?;
                            relay::expect_send_with_progress(
                                &mut p,
                                vec![
                                    (" [my-relay] [repo-relay] ws://localhost:8055", true, ""),
                                    (" [my-relay] ws://localhost:8053", true, ""),
                                    (" [repo-relay] ws://localhost:8056", true, ""),
                                    (" [default] ws://localhost:8051", true, ""),
                                    (" [default] ws://localhost:8052", true, ""),
                                    (" [default] ws://localhost:8057", true, ""),
                                ],
                                1,
                            )?;
                            expect_msgs_after_init(&mut p)?;
                            p.expect_end()?;
                            for p in [51, 52, 53, 55, 56, 57] {
                                relay::shutdown_relay(8000 + p)?;
                            }
                            Ok(())
                        });

                        // launch relay
                        let _ = join!(
                            r51.listen_until_close(),
                            r52.listen_until_close(),
                            r53.listen_until_close(),
                            r55.listen_until_close(),
                            r56.listen_until_close(),
                            r57.listen_until_close(),
                        );
                        cli_tester_handle.join().unwrap()?;
                        Ok(())
                    }
                }
            }
        }
    }
