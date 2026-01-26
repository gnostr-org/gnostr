use std::{io, process::Command, thread, time::Duration};

fn main() {
    // --- Path setup ---
    // Create a path in the local repo, making it cross-platform compatible.
    let mut screenshot_path = std::env::current_dir().expect("Failed to get current directory");
    screenshot_path.push("test_screenshots");

    // Create the directory if it doesn't exist.
    std::fs::create_dir_all(&screenshot_path).expect("Failed to create screenshot directory");

    screenshot_path.push("tui_capture.png");
    let screenshot_path_str = screenshot_path.to_str().expect("Path is not valid UTF-8");

    //// --- 1. Compile the gnostr binary ---
    //println!("Compiling gnostr TUI...");
    //let build_status = Command::new("cargo")
    //    .arg("build")
    //    .arg("--bin")
    //    .arg("gnostr")
    //    .status()
    //    .expect("Failed to execute cargo build");

    //if !build_status.success() {
    //    panic!("Failed to compile gnostr TUI. Aborting.");
    //}

    // --- 2. Launch TUI in a new Terminal window and get its ID ---
    let project_dir = std::env::current_dir()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let start_and_get_id_script = format!(
        r#"
        tell application "Terminal"
            activate
            set theTab to do script "cd {} && {}/target/debug/gnostr"
            delay 2
            return id of first window whose tabs contains theTab
        end tell
        "#,
        project_dir, project_dir
    );

    println!("Launching TUI and getting window ID...");
    let osascript_output = Command::new("osascript")
        .arg("-e")
        .arg(&start_and_get_id_script)
        .output()
        .expect("Failed to execute osascript to start TUI");

    if !osascript_output.status.success() {
        eprintln!(
            "osascript error: {}",
            String::from_utf8_lossy(&osascript_output.stderr)
        );
        panic!("Failed to get window ID from AppleScript.");
    }

    let window_id = String::from_utf8(osascript_output.stdout)
        .expect("osascript output was not valid UTF-8")
        .trim()
        .to_string();

    if window_id.is_empty() {
        panic!("AppleScript did not return a window ID. Cannot proceed.");
    }
    println!("TUI is running in window ID: {}", window_id);

    // --- 3. Capture the screenshot with a retry loop ---
    println!("Waiting for TUI to render...");
    thread::sleep(Duration::from_secs(5));

    let mut attempts = 0;
    const MAX_ATTEMPTS: u8 = 1;
    let mut capture_successful = false;

    while attempts < MAX_ATTEMPTS {
        println!(
            "Capturing screenshot... Attempt {}/{}",
            attempts + 1,
            MAX_ATTEMPTS
        );
        let capture_output = Command::new("screencapture")
            .arg("-l") // Capture a specific window ID
            .arg(&window_id)
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
        eprintln!(
            "Failed to capture screenshot after {} attempts.",
            MAX_ATTEMPTS
        );
    }

    // --- 4. Clean up ---
    //if let Err(e) = close_window(window_id) {
    //    eprintln!("Error closing window: {}", e);
    //}
    println!("Automation complete.");
}

#[allow(dead_code)]
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
