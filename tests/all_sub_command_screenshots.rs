/// ## All Subcommand Screenshot Testing
///
/// This test suite is designed to capture the TUI of each subcommand
/// to ensure that the CLI TUI messages are consistent and correct.
///
/// To add a new screenshot test, simply add a new call to the `screenshot_test`
/// macro with the subcommand name.
///
#[cfg(test)]
mod tests {
    use std::process::Command;
    use assert_cmd::cargo::cargo_bin;
    use std::error::Error;
    use gnostr::utils::screenshot;
    use std::fs;
    use std::thread;
    use std::time::Duration;
    use tempfile::TempDir;
    use git2::{Repository, Signature};
    use std::io::Write;
    use std::path::Path;

    // Helper function to set up a temporary git repository for testing.
    fn setup_test_repo() -> (TempDir, Repository) {
        let tmp_dir = TempDir::new().unwrap();
        let repo_path = tmp_dir.path();
        let repo = Repository::init(repo_path).unwrap();

        // Configure user name and email
        let mut config = repo.config().unwrap();
        config.set_str("user.name", "Test User").unwrap();
        config.set_str("user.email", "test@example.com").unwrap();
        config.set_str("gnostr.relays", "wss://relay.example.com").unwrap();

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
            repo.reset(repo.head().unwrap().peel_to_commit().unwrap().as_object(), git2::ResetType::Hard, None).unwrap();
        }

        (tmp_dir, repo)
    }

    macro_rules! screenshot_test {
        ($name:ident, $subcommand:expr) => {
            #[test]
            #[cfg(target_os = "macos")]
            fn $name() -> Result<(), Box<dyn Error>> {
                let (_tmp_dir, repo) = setup_test_repo();
                let repo_path = repo.path().to_str().unwrap().to_string();

                let mut cmd = Command::new(cargo_bin("gnostr"));
                cmd.arg("--gitdir").arg(&repo_path).arg($subcommand);

                // Spawn the command as a child process
                let mut child = cmd.spawn().expect("Failed to spawn gnostr command");

                // Give the TUI a moment to initialize
                thread::sleep(Duration::from_secs(2));

                // Capture the screenshot
                let screenshot_path_result = screenshot::make_screenshot(concat!("gnostr_", $subcommand, "_run"));

                // Terminate the child process
                child.kill().expect("Failed to kill gnostr process");
                child.wait().expect("Failed to wait for gnostr process");

                // Assert that the screenshot was created
                assert!(screenshot_path_result.is_ok(), "Failed to capture screenshot.");
                let screenshot_path = screenshot_path_result.unwrap();
                let metadata = fs::metadata(&screenshot_path).expect("Failed to get screenshot metadata");
                assert!(metadata.is_file(), "Screenshot is not a file");
                assert!(metadata.len() > 0, "Screenshot file is empty");

                Ok(())
            }
        };
    }

    screenshot_test!(test_award_badge_run_screenshot, "award-badge");
    screenshot_test!(test_bech32_to_any_run_screenshot, "bech32-to-any");
    screenshot_test!(test_broadcast_events_run_screenshot, "broadcast-events");
    screenshot_test!(test_convert_key_run_screenshot, "convert-key");
    screenshot_test!(test_create_badge_run_screenshot, "create-badge");
    screenshot_test!(test_create_public_channel_run_screenshot, "create-public-channel");
    screenshot_test!(test_custom_event_run_screenshot, "custom-event");
    screenshot_test!(test_delete_event_run_screenshot, "delete-event");
    screenshot_test!(test_delete_profile_run_screenshot, "delete-profile");
    screenshot_test!(test_fetch_run_screenshot, "fetch");
    screenshot_test!(test_generate_keypair_run_screenshot, "generate-keypair");
    screenshot_test!(test_git_run_screenshot, "git");
    screenshot_test!(test_hide_public_channel_message_run_screenshot, "hide-public-channel-message");
    screenshot_test!(test_list_events_run_screenshot, "list-events");
    screenshot_test!(test_login_run_screenshot, "login");
    screenshot_test!(test_mute_publickey_run_screenshot, "mute-publickey");
    screenshot_test!(test_note_run_screenshot, "note");
    screenshot_test!(test_profile_badges_run_screenshot, "profile-badges");
    screenshot_test!(test_publish_contactlist_csv_run_screenshot, "publish-contactlist-csv");
    screenshot_test!(test_query_run_screenshot, "query");
    screenshot_test!(test_react_run_screenshot, "react");
    screenshot_test!(test_relay_run_screenshot, "relay");
    screenshot_test!(test_send_channel_message_run_screenshot, "send-channel-message");
    screenshot_test!(test_set_channel_metadata_run_screenshot, "set-channel-metadata");
    screenshot_test!(test_set_metadata_run_screenshot, "set-metadata");
    screenshot_test!(test_sniper_run_screenshot, "sniper");
    screenshot_test!(test_user_status_run_screenshot, "user-status");
    screenshot_test!(test_vanity_run_screenshot, "vanity");
    screenshot_test!(test_privkey_to_bech32_run_screenshot, "privkey-to-bech32");
    screenshot_test!(test_fetch_by_id_run_screenshot, "fetch-by-id");

    //TODO these are ratatui
    //     they need to have a proper ratatui life cycle
    //     and restore terminal when finished
    // screenshot_test!(test_chat_run_screenshot, "chat");
    // ignore screenshot_test!(test_tui_run_screenshot, "tui");

    // TODO ngit
    // screenshot_test!(test_ngit_run_screenshot, "ngit");
    // // screenshot_test!(test_init_run_screenshot, "init");
    // // screenshot_test!(test_push_run_screenshot, "push");
    // // screenshot_test!(test_send_run_screenshot, "send");
    // // screenshot_test!(test_list_run_screenshot, "list");
    // // screenshot_test!(test_pull_run_screenshot, "pull");
}
