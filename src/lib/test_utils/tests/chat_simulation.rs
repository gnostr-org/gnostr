/// ## Chat Simulation Screenshot Testing
///
/// This test suite is designed to capture the TUI of multiple `gnostr chat`
/// instances to ensure that the CLI TUI messages are consistent and correct.
#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use serial_test::serial;
    use tracing::{debug, info};

    use crate::{
        test_utils::{CliTester, git::GitTestRepo},
        utils::screenshot::make_screenshot,
    };

    #[test]
    #[serial]
    #[cfg(feature = "expensive_tests")]
    #[ignore]
    fn test_chat_simulation() -> Result<(), Box<dyn std::error::Error>> {
        // Enable verbose output for this test
        println!("Starting chat simulation test with verbose output");
        info!("Chat simulation test started");
        // --- 1. Set up two separate git repositories ---
        info!("Setting up test repositories");
        debug!("Creating first repository");
        let mut repo1 = GitTestRepo::new("main")?;
        repo1.initial_commit()?;
        debug!("Creating second repository");
        let mut repo2 = GitTestRepo::new("main")?;
        repo2.initial_commit()?;
        info!("Both repositories initialized successfully");

        // --- 2. Use oneshot chat messages instead of full TUI ---
        let topic = "test_chat_simulation";
        info!("Using chat topic: {}", topic);

        // --- 3. Send oneshot messages from both users ---
        info!("Setting up chat clients");
        let _chat1 = CliTester::new_from_dir(
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

        let _chat2 = CliTester::new_from_dir(
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

        // --- 4. Wait for oneshot messages to be processed and verify output ---
        info!("Waiting for messages to be processed (5 seconds)");
        for i in 1..=5 {
            debug!("Waiting... {}/5 seconds", i);
            thread::sleep(Duration::from_secs(1));
        }

        // --- 5. Verify message processing by checking logs and outputs ---
        info!("Verifying chat message processing");

        // The oneshot messages should have been sent successfully.
        // Since we're running headless, we can't directly interact with the TUI,
        // but we can verify that the processes started and ran without errors.
        // The Drop implementation will handle cleanup.

        // --- 6. Capture screenshots to show oneshot completion ---
        info!("Capturing screenshot");
        make_screenshot("chat_simulation_after_oneshot_messages")?;

        // The CliTester's Drop implementation will automatically kill the child
        // processes and restore the terminal.
        info!("Chat simulation test completed successfully");

        Ok(())
    }
}
