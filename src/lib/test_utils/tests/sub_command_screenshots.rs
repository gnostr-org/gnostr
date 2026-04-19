/// ## Subcommand Screenshot Testing
///
/// This test suite is designed to capture the `--help` output of each
/// subcommand to ensure that the CLI help messages are consistent and correct.
///
/// To add a new screenshot test, simply add a new call to the `screenshot_test`
/// macro with the subcommand name.
#[cfg(test)]
mod tests {
    //TODO use anyhow::Error;
    use std::{error::Error, fs, io::Write, path::Path, process::Command};

    use assert_cmd::{assert::OutputAssertExt, cargo::cargo_bin};
    use git2::{Repository, Signature};
    use serial_test::serial;
    use tempfile::TempDir;

    use crate::utils::screenshot;

    // cargo t -j$(nproc) --features expensive_tests -- --no-capture --ignored
    macro_rules! screenshot_test {
        ($name:ident, $subcommand:expr) => {
            #[test]
            #[serial]
            #[ignore]
            #[cfg(feature = "expensive_tests")]
            #[cfg(target_os = "macos")]
            fn $name() -> Result<(), Box<dyn Error>> {
                let mut cmd = Command::new(cargo_bin("gnostr"));
                cmd.arg($subcommand).arg("--help");
                cmd.assert().success();

                let screenshot_path_result =
                    screenshot::make_screenshot(concat!($subcommand, "_help"));

                assert!(
                    screenshot_path_result.is_ok(),
                    "Failed to capture screenshot."
                );
                let screenshot_path = screenshot_path_result.unwrap();
                let metadata =
                    fs::metadata(&screenshot_path).expect("Failed to get screenshot metadata");
                assert!(metadata.is_file(), "Screenshot is not a file");
                assert!(metadata.len() > 0, "Screenshot file is empty");

                Ok(())
            }
        };
    }

    #[cfg(feature = "expensive_tests")]
    #[cfg(target_os = "macos")]
    screenshot_test!(test_award_badge_help_screenshot, "award-badge");
    #[cfg(feature = "expensive_tests")]
    #[cfg(target_os = "macos")]
    screenshot_test!(test_bech32_to_any_help_screenshot, "bech32-to-any");
    #[cfg(feature = "expensive_tests")]
    #[cfg(target_os = "macos")]
    screenshot_test!(test_broadcast_events_help_screenshot, "broadcast-events");
    #[cfg(feature = "expensive_tests")]
    #[cfg(target_os = "macos")]
    screenshot_test!(test_convert_key_help_screenshot, "convert-key");
    #[cfg(feature = "expensive_tests")]
    #[cfg(target_os = "macos")]
    screenshot_test!(test_create_badge_help_screenshot, "create-badge");
    #[cfg(feature = "expensive_tests")]
    #[cfg(target_os = "macos")]
    screenshot_test!(
        test_create_public_channel_help_screenshot,
        "create-public-channel"
    );
    #[cfg(feature = "expensive_tests")]
    #[cfg(target_os = "macos")]
    screenshot_test!(test_custom_event_help_screenshot, "custom-event");
    #[cfg(feature = "expensive_tests")]
    #[cfg(target_os = "macos")]
    screenshot_test!(test_delete_event_help_screenshot, "delete-event");
    #[cfg(feature = "expensive_tests")]
    #[cfg(target_os = "macos")]
    screenshot_test!(test_delete_profile_help_screenshot, "delete-profile");
    #[cfg(feature = "expensive_tests")]
    #[cfg(target_os = "macos")]
    screenshot_test!(test_fetch_by_id_help_screenshot, "fetch-by-id");
    #[cfg(feature = "expensive_tests")]
    #[cfg(target_os = "macos")]
    screenshot_test!(test_generate_keypair_help_screenshot, "generate-keypair");
    #[cfg(feature = "expensive_tests")]
    #[cfg(target_os = "macos")]
    screenshot_test!(test_git_help_screenshot, "git");
    #[cfg(feature = "expensive_tests")]
    #[cfg(target_os = "macos")]
    screenshot_test!(
        test_hide_public_channel_message_help_screenshot,
        "hide-public-channel-message"
    );
    #[cfg(feature = "expensive_tests")]
    #[cfg(target_os = "macos")]
    screenshot_test!(test_list_events_help_screenshot, "list-events");
    #[cfg(feature = "expensive_tests")]
    #[cfg(target_os = "macos")]
    screenshot_test!(test_login_help_screenshot, "legit");
    #[cfg(feature = "expensive_tests")]
    #[cfg(target_os = "macos")]
    screenshot_test!(test_mute_publickey_help_screenshot, "mute-public-key");
    #[cfg(feature = "expensive_tests")]
    #[cfg(target_os = "macos")]
    screenshot_test!(test_ngit_help_screenshot, "ngit");
    #[cfg(feature = "expensive_tests")]
    #[cfg(target_os = "macos")]
    screenshot_test!(test_note_help_screenshot, "note");
    #[cfg(feature = "expensive_tests")]
    #[cfg(target_os = "macos")]
    screenshot_test!(test_profile_badges_help_screenshot, "profile-badges");
    #[cfg(feature = "expensive_tests")]
    #[cfg(target_os = "macos")]
    screenshot_test!(
        test_publish_contactlist_csv_help_screenshot,
        "publish-contact-list-csv"
    );
    #[cfg(feature = "expensive_tests")]
    #[cfg(target_os = "macos")]
    screenshot_test!(test_push_help_screenshot, "git");
    #[cfg(feature = "expensive_tests")]
    #[cfg(target_os = "macos")]
    screenshot_test!(test_query_help_screenshot, "query");
    #[cfg(feature = "expensive_tests")]
    #[cfg(target_os = "macos")]
    screenshot_test!(test_react_help_screenshot, "react");
    #[cfg(feature = "expensive_tests")]
    #[cfg(target_os = "macos")]
    screenshot_test!(test_relay_help_screenshot, "relay");
    #[cfg(feature = "expensive_tests")]
    #[cfg(target_os = "macos")]
    screenshot_test!(
        test_send_channel_message_help_screenshot,
        "send-channel-message"
    );
    #[cfg(feature = "expensive_tests")]
    #[cfg(target_os = "macos")]
    screenshot_test!(
        test_set_channel_metadata_help_screenshot,
        "set-channel-metadata"
    );
    #[cfg(feature = "expensive_tests")]
    #[cfg(target_os = "macos")]
    screenshot_test!(test_set_metadata_help_screenshot, "set-metadata");
    #[cfg(feature = "expensive_tests")]
    #[cfg(target_os = "macos")]
    screenshot_test!(test_sniper_help_screenshot, "sniper");
    #[cfg(feature = "expensive_tests")]
    #[cfg(target_os = "macos")]
    screenshot_test!(test_user_status_help_screenshot, "set-user-status");
    #[cfg(feature = "expensive_tests")]
    #[cfg(target_os = "macos")]
    screenshot_test!(test_vanity_help_screenshot, "vanity");
    #[cfg(feature = "expensive_tests")]
    #[cfg(target_os = "macos")]
    screenshot_test!(test_chat_help_screenshot, "chat");
    #[cfg(feature = "expensive_tests")]
    #[cfg(target_os = "macos")]
    screenshot_test!(test_tui_help_screenshot, "tui");
    #[cfg(feature = "expensive_tests")]
    #[cfg(target_os = "macos")]
    screenshot_test!(test_privkey_to_bech32_help_screenshot, "privkey-to-bech32");
}
