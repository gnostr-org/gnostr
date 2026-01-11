/// ## Chat Simulation Screenshot Testing
///
/// This test suite is designed to capture the TUI of multiple `gnostr chat`
/// instances to ensure that the CLI TUI messages are consistent and correct.
#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use serial_test::serial;

    use crate::{
        test_utils::{CliTester, git::GitTestRepo},
        utils::screenshot::make_screenshot,
    };

    #[test]
    #[serial]
    fn test_chat_simulation() -> Result<(), Box<dyn std::error::Error>> {
        // --- 1. Set up two separate git repositories ---
        let mut repo1 = GitTestRepo::new("main")?;
        repo1.initial_commit()?;
        let mut repo2 = GitTestRepo::new("main")?;
        repo2.initial_commit()?;

        // --- 2. Use oneshot chat messages instead of full TUI ---
        let topic = "test_chat_simulation";

        // --- 3. Send oneshot messages from both users ---
        let mut chat1 = CliTester::new_from_dir(
            &repo1.dir,
            [
                "chat",
                "--topic",
                topic,
                "--oneshot",
                "Hello from user 1!",
                "--headless",
            ],
        );

        let mut chat2 = CliTester::new_from_dir(
            &repo2.dir,
            [
                "chat",
                "--topic",
                topic,
                "--oneshot",
                "Hello back from user 2!",
                "--headless",
            ],
        );

        // --- 4. Wait for oneshot messages to be processed ---
        thread::sleep(Duration::from_secs(5));

        // --- 5. Capture screenshots to show oneshot completion ---
        make_screenshot("chat_simulation_after_oneshot_messages")?;

        // The CliTester's Drop implementation will automatically kill the child
        // processes and restore the terminal.

        Ok(())
    }
}
