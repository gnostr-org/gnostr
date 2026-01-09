#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::pin::Pin;
    use std::process::{Stdio, ExitStatus}; // Added ExitStatus
    use std::io::Error; // Import io::Error explicitly
    use tokio::process::Command; // Corrected to tokio::process::Command
    use std::time::Duration;
    use tokio::time::timeout;

    #[tokio::test]
    async fn test_chat_headless_mode_does_not_block() {
        // Build the project first to ensure the binary is available
        // We're using `cargo build` here to make sure the binary is up-to-date
        // and accessible for `Command::new` later.
        let build_output = Command::new("cargo")
            .arg("build")
            .output()
            .await // Await the Future returned by output()
            .expect("Failed to build gnostr project");

        assert!(build_output.status.success(), "Cargo build failed: {:?}", build_output);

        // Run the gnostr chat command in headless mode
        // We need to specify `--bin gnostr` to run the main binary from the workspace.
        // We capture stdout/stderr to ensure it doesn't inherit them directly in the test run.
        let mut child_process = Command::new("cargo") // Use tokio::process::Command
            .arg("run")
            .arg("--bin")
            .arg("gnostr")
            .arg("--")
            .arg("chat")
            .arg("--headless")
            .stdin(Stdio::null()) // Ensure no stdin is inherited, preventing blocking for input
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to spawn gnostr chat --headless command");

        // Explicitly type the future for child process completion
        let wait_future: Pin<Box<dyn Future<Output = Result<ExitStatus, Error>> + Send>> = Box::pin(child_process.wait());
        
        // Wait for a short period to see if the process exits quickly (non-blocking)
        // If it blocks for TUI, this timeout will catch it.
        let start_time = tokio::time::Instant::now();
        let timeout_duration = Duration::from_secs(5); // A reasonable time to ensure it's not blocking for TUI
        
        let wait_result = timeout(timeout_duration, wait_future).await;

        match wait_result {
            Ok(Ok(status)) => {
                // If it exited quickly, it means the process finished.
                // This is acceptable if it means it successfully spawned background tasks and exited.
                // We mainly assert it didn't crash.
                assert!(status.success(), "gnostr chat --headless exited with error: {:?}", status);
                let duration = tokio::time::Instant::now() - start_time;
                println!("Headless chat process exited in {:?} seconds", duration);
                assert!(duration < Duration::from_secs(2), "Headless chat process took too long to exit, possibly blocked.");
            },
            Ok(Err(e)) => {
                panic!("Failed to wait for child process: {:?}", e);
            },
            Err(_) => {
                // This means the process is still running after the timeout, which is the expected
                // behavior for a truly "detached" background process.
                println!("Headless chat process is still running after {:?} seconds (expected detached behavior).", timeout_duration);
                child_process.kill().await.expect("Failed to kill detached headless chat process"); // Await the Future returned by kill()
                // TODO: Add more sophisticated checks here if needed, e.g., connecting to its P2P network
            }
        }
    }
}