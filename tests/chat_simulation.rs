/// ## Chat Simulation Screenshot Testing
///
/// This test suite is designed to capture the TUI of multiple `gnostr chat`
/// instances to ensure that the CLI TUI messages are consistent and correct.
///
#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::Duration;
    use test_utils::git::GitTestRepo;
    use test_utils::CliTester;
    use gnostr::utils::screenshot::make_screenshot;
    use serial_test::serial;

    #[test]
    #[ignore]
    #[serial]
    fn test_chat_simulation() -> Result<(), Box<dyn std::error::Error>> {
        // --- 1. Set up two separate git repositories ---
        let mut repo1 = GitTestRepo::new("main")?;
        repo1.initial_commit()?;
        let mut repo2 = GitTestRepo::new("main")?;
        repo2.initial_commit()?;

        // --- 2. Launch two `gnostr chat` instances using CliTester ---
        let mut chat1 = CliTester::new_from_dir(&repo1.dir, ["chat"]);
        let mut chat2 = CliTester::new_from_dir(&repo2.dir, ["chat"]);

        // --- 3. Give the TUIs a moment to initialize ---
        thread::sleep(Duration::from_secs(2));

        // --- 4. Simulate a conversation ---
        chat1.send_line("Hello from user 1!")?;
        thread::sleep(Duration::from_secs(1));

        chat2.send_line("Hello back from user 2!")?;
        thread::sleep(Duration::from_secs(1));

        // --- 5. Capture screenshots at different stages ---
        make_screenshot("chat_simulation_after_user1_sends")?;
        thread::sleep(Duration::from_secs(1));

        make_screenshot("chat_simulation_after_user2_replies")?;

        // The CliTester's Drop implementation will automatically kill the child processes
        // and restore the terminal.

        Ok(())
    }
}
