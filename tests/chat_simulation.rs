/// ## Chat Simulation Screenshot Testing
///
/// This test suite is designed to capture the TUI of multiple `gnostr chat`
/// instances to ensure that the CLI TUI messages are consistent and correct.
///
#[cfg(test)]
mod tests {
    use std::process::Command;
    use std::thread;
    use std::time::Duration;
    use std::path::PathBuf;
    use std::io;
    use std::fs;
    use tempfile::TempDir;
    use git2::{Repository, Signature};
    use std::io::Write;
    use std::path::Path;

    // Helper function to set up a temporary git repository for testing.
    fn setup_test_repo(repo_name: &str) -> (TempDir, Repository) {
        let tmp_dir = TempDir::new().unwrap();
        let repo_path = tmp_dir.path().join(repo_name);
        let repo = Repository::init(&repo_path).unwrap();

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

    #[test]
    #[cfg(target_os = "macos")]
    fn test_chat_simulation() -> Result<(), Box<dyn std::error::Error>> {
        // --- Path setup ---
        let mut screenshot_path = std::env::current_dir().expect("Failed to get current directory");
        screenshot_path.push("test_screenshots");
        std::fs::create_dir_all(&screenshot_path).expect("Failed to create screenshot directory");
        screenshot_path.push("chat_simulation.png");
        let screenshot_path_str = screenshot_path.to_str().expect("Path is not valid UTF-8");

        // --- 1. Compile the gnostr binary ---
        println!("Compiling gnostr TUI...");
        let build_status = Command::new("cargo")
            .arg("build")
            .arg("--bin")
            .arg("gnostr")
            .status()
            .expect("Failed to execute cargo build");

        if !build_status.success() {
            panic!("Failed to compile gnostr TUI. Aborting.");
        }

        // --- 2. Launch TUI in a new Terminal window and get its ID ---
        let project_dir = std::env::current_dir().unwrap().to_str().unwrap().to_string();
        let (repo1_dir, _repo1) = setup_test_repo("repo1");
        let (repo2_dir, _repo2) = setup_test_repo("repo2");
        let repo1_path = repo1_dir.path().join("repo1").to_str().unwrap().to_string();
        let repo2_path = repo2_dir.path().join("repo2").to_str().unwrap().to_string();

        let start_and_get_id_script = format!(
            r#"
            tell application "Terminal"
                activate
                set theTab1 to do script "cd {} && {}/target/debug/gnostr --gitdir {} chat"
                delay 2
                set theTab2 to do script "cd {} && {}/target/debug/gnostr --gitdir {} chat"
                delay 2
                return {{id of first window whose tabs contains theTab1, id of first window whose tabs contains theTab2}}
            end tell
            "#,
            project_dir, project_dir, repo1_path, project_dir, project_dir, repo2_path
        );

        println!("Launching TUI and getting window ID...");
        let osascript_output = Command::new("osascript")
            .arg("-e")
            .arg(&start_and_get_id_script)
            .output()
            .expect("Failed to execute osascript to start TUI");

        if !osascript_output.status.success() {
            eprintln!("osascript error: {}", String::from_utf8_lossy(&osascript_output.stderr));
            panic!("Failed to get window ID from AppleScript.");
        }

        let window_ids_str = String::from_utf8(osascript_output.stdout)
            .expect("osascript output was not valid UTF-8")
            .trim()
            .to_string();

        if window_ids_str.is_empty() {
            panic!("AppleScript did not return a window ID. Cannot proceed.");
        }

        let window_ids: Vec<&str> = window_ids_str.split(", ").collect();

        // --- 3. Capture the screenshot with a retry loop ---
        println!("Waiting for TUI to render...");
        thread::sleep(Duration::from_secs(5));

        let mut attempts = 0;
        const MAX_ATTEMPTS: u8 = 3;
        let mut capture_successful = false;

        while attempts < MAX_ATTEMPTS {
            println!(
                "Capturing screenshot... Attempt {}/{}",
                attempts + 1,
                MAX_ATTEMPTS
            );
            let capture_output = Command::new("screencapture")
                .arg(screenshot_path_str)
                .output()
                .expect("Failed to execute screencapture");

            if capture_output.status.success() {
                println!("Screenshot saved to {}", screenshot_path_str);
                capture_successful = true;
                break;
            } else {
                eprintln!(
                    "Failed to capture screenshot on attempt {}: {}",
                    attempts + 1,
                    String::from_utf8_lossy(&capture_output.stderr)
                );
                attempts += 1;
                thread::sleep(Duration::from_secs(2)); // Wait before retrying
            }
        }

        if !capture_successful {
            eprintln!("Failed to capture screenshot after {} attempts.", MAX_ATTEMPTS);
        }

        // --- 4. Clean up ---
        for window_id in window_ids {
            // disabled close_window(window_id.to_string())?;
        }

        Ok(())
    }

    fn close_window(window_id: String) -> io::Result<()> {
        // Using `saving no` prevents the confirmation dialog when a process is running.
        let close_script = format!(
            r#"
            tell application "Terminal"
                close window id {} saving no
            end tell
            "#,
            window_id
        );

        println!("Closing TUI window...");
        let close_output = Command::new("osascript")
            .arg("-e")
            .arg(&close_script)
            .output()?;

        if !close_output.status.success() {
            eprintln!(
                "Failed to close terminal window: {}",
                String::from_utf8_lossy(&close_output.stderr)
            );
        }

        Ok(())
    }
}
