use std::process::Command;
use std::thread;
use std::time::Duration;
use std::path::PathBuf;

fn main() {
    // Define the path for the screenshot
    let screenshot_path: PathBuf = [
        "/Users/randymcmillan/.gemini/tmp/e2231b79f1bd8b2a6ddfa2b8e1288337b83b48b9c931587d9e416bcd03fb2b14",
        "tui_capture.png",
    ]
    .iter()
    .collect();
    let screenshot_path_str = screenshot_path.to_str().expect("Path is not valid UTF-8");

    // --- 1. Compile the gnostr binary ---
    // This ensures we're testing the latest code without race conditions from `cargo watch`.
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
    // We use a single AppleScript to both launch the TUI and return the new window's ID.
    // This is more reliable than trying to find the window after the fact.
    let project_dir = std::env::current_dir().unwrap().to_str().unwrap().to_string();
    let start_and_get_id_script = format!(
        r#"
        tell application "Terminal"
            activate
            -- Execute the command in a new tab/window
            set theTab to do script "cd {} && {}/target/debug/gnostr tui"
            -- Give the window a moment to be created and assigned an ID
            delay 2
            -- Find the window containing the new tab and return its ID
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
        eprintln!("osascript error: {}", String::from_utf8_lossy(&osascript_output.stderr));
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

    // --- 3. Capture the screenshot ---
    // Wait for the TUI to fully render before taking the screenshot.
    println!("Waiting for TUI to render...");
    thread::sleep(Duration::from_secs(5));

    println!("Capturing screenshot of window {}...", window_id);
    let mut capture_cmd = Command::new("screencapture");
    capture_cmd.arg("-l") // Capture a specific window ID
               .arg(&window_id)
               .arg(screenshot_path_str);
    
    let capture_status = capture_cmd.status().expect("Failed to execute screencapture");

    if capture_status.success() {
        println!("Screenshot saved to {}", screenshot_path_str);
    } else {
        eprintln!("Failed to capture screenshot.");
    }

    // --- 4. Clean up and close the Terminal window ---
    let close_script = format!(
        r#"
        tell application "Terminal"
            close window id {}
        end tell
        "#,
        window_id
    );

    println!("Closing TUI window...");
    Command::new("osascript")
        .arg("-e")
        .arg(&close_script)
        .status()
        .expect("Failed to close terminal window via osascript");

    println!("Automation complete.");
}
