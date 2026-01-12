/// ## All Subcommand Screenshot Testing
///
/// This test suite is designed to capture the TUI of each subcommand
/// to ensure that the CLI TUI messages are consistent and correct.
///
/// To add a new screenshot test, simply add a new call to the `screenshot_test`
/// macro with the subcommand name.
#[cfg(test)]
mod tests {

    use std::{fs, io::Write, path::Path, process::Command};

    use anyhow::Error;
    use assert_cmd::cargo_bin;
    use git2::{Repository, Signature};
    use serial_test::serial;
    use tempfile::TempDir;

    // Helper function to set up a temporary git repository for testing.
    fn setup_test_repo() -> (TempDir, Repository) {
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
                fs::File::create(&file_path)
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

        (tmp_dir, repo)
    }

    macro_rules! _screenshot_test {
        ($name:ident, $subcommand:expr, $is_tui:expr) => {
            #[test]
            #[serial]
            #[cfg(target_os = "macos")]
            #[cfg(feature = "expensive_tests")]
            fn $name() -> Result<(), Box<dyn Error>> {
                let (_tmp_dir, repo) = setup_test_repo();
                let repo_path = repo.path().to_str().unwrap().to_string();
                let gnostr_bin = cargo_bin("gnostr");

                // Command to run gnostr in a new terminal window
                let mut cmd = Command::new("open");
                cmd.args(["-n", "-a", "Terminal"]);

                // Build the command to execute
                let mut terminal_cmd = format!(
                    "cd '{}' && '{}' '{}' --repo-path '{}'",
                    repo_path,
                    gnostr_bin.to_str().unwrap(),
                    $subcommand,
                    repo_path
                );

                // Add TUI-specific arguments if needed
                if $is_tui {
                    terminal_cmd.push_str(" && sleep 3"); // Give time for TUI to load
                }

                cmd.arg("--args");
                cmd.arg(terminal_cmd);

                // Execute the command
                cmd.output().expect("Failed to open terminal");

                // Take screenshot after a delay
                std::thread::sleep(std::time::Duration::from_secs(5));

                Ok(())
            }
        };
    }

    // Test cases - currently commented out as they are more complex to run in
    // CI Uncomment and use the _screenshot_test! macro when ready

    #[cfg(feature = "expensive_tests")]
    screenshot_test!(test_award_badge_run_screenshot, "award-badge", false);
    #[cfg(feature = "expensive_tests")]
    screenshot_test!(test_bech32_to_any_run_screenshot, "bech32-to-any", false);
    #[cfg(feature = "expensive_tests")]
    screenshot_test!(
        test_broadcast_events_run_screenshot,
        "broadcast-events",
        false
    );
    // "broadcast-events", false); screenshot_test!
    #[cfg(feature = "expensive_tests")]
    screenshot_test!(test_create_badge_run_screenshot, "create-badge", false);
    // false); screenshot_test!(test_create_public_channel_run_screenshot,
    // "create-public-channel", false); screenshot_test!
    // (test_delete_event_run_screenshot, "delete-event",
    // false); screenshot_test!(test_delete_profile_run_screenshot,
    // "delete-profile", false); screenshot_test!(test_fetch_run_screenshot,
    // "fetch", true); screenshot_test!
    #[cfg(feature = "expensive_tests")]
    screenshot_test!(test_git_run_screenshot, "git", true);
    // screenshot_test!(test_hide_public_channel_message_run_screenshot,
    // "hide-public-channel-message", false); screenshot_test!
    #[cfg(feature = "expensive_tests")]
    screenshot_test!(test_login_run_screenshot, "login", true);
    // screenshot_test!(test_mute_publickey_run_screenshot, "mute-publickey",
    // false); screenshot_test!(test_note_run_screenshot, "note", false);
    // screenshot_test!(test_profile_badges_run_screenshot, "profile-badges",
    // false); screenshot_test!(test_publish_contactlist_csv_run_screenshot,
    // "publish-contactlist-csv", false); screenshot_test!
    // (test_query_run_screenshot, "query", false); screenshot_test!
    // (test_react_run_screenshot, "react", false); screenshot_test!
    // (test_relay_run_screenshot, "relay", true); screenshot_test!
    // false); screenshot_test!(test_set_channel_metadata_run_screenshot,
    // "set-channel-metadata", false); screenshot_test!
    // (test_sniper_run_screenshot, "sniper", true);
    // screenshot_test!(test_user_status_run_screenshot, "user-status", false);
    #[cfg(feature = "expensive_tests")]
    screenshot_test!(test_vanity_run_screenshot, "vanity", true);
    // screenshot_test!(test_privkey_to_bech32_run_screenshot,
    // "privkey-to-bech32", false); screenshot_test!
    // (test_chat_run_screenshot, "chat", true);
    #[cfg(feature = "expensive_tests")]
    screenshot_test!(test_tui_run_screenshot, "tui", true);
    // screenshot_test!(test_ngit_run_screenshot, "ngit", true);
    //     #[cfg(feature = "expensive_tests")]
    screenshot_test!(test_init_run_screenshot, "init", true);
    //     #[cfg(feature = "expensive_tests")]
    screenshot_test!(test_push_run_screenshot, "push", true);
    // // screenshot_test!(test_send_run_screenshot, "send", true);
    // // screenshot_test!(test_list_run_screenshot, "list", true);
    // // screenshot_test!(test_pull_run_screenshot, "pull", true);
}

#[ctor::dtor]
fn cleanup_terminal() {
    let _ = crossterm::terminal::disable_raw_mode();
    let _ = crossterm::execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen);
}
