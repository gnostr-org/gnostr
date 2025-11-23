use std::process::Command;
use std::thread;
use std::time::Duration;
use std::path::PathBuf;

fn main() {
    let screenshot_path: PathBuf = [
        "/Users/randymcmillan/.gemini/tmp/e2231b79f1bd8b2a6ddfa2b8e1288337b83b48b9c931587d9e416bcd03fb2b14",
        "tui_capture.png",
    ]
    .iter()
    .collect();
    let screenshot_path_str = screenshot_path.to_str().expect("Path is not valid UTF-8");

    // Compile the gnostr binary first to avoid delays
    println!("Compiling gnostr TUI...");
    let mut build_cmd = Command::new("cargo");
    build_cmd.arg("build").arg("--bin").arg("gnostr");
    let build_status = build_cmd.status().expect("Failed to build gnostr binary");
    if !build_status.success() {
        panic!("Failed to compile gnostr TUI");
    }

    // Use osascript to open a new terminal and run the pre-compiled TUI
    let script = format!(
        "tell application \"Terminal\" to do script \"cd {} && ./target/debug/gnostr tui\"",
        std::env::current_dir().unwrap().to_str().unwrap()
    );

    let mut cmd = Command::new("osascript");
    cmd.arg("-e").arg(&script);

    let status = cmd.status().expect("Failed to open new terminal with osascript");
    if !status.success() {
        panic!("Failed to execute osascript");
    }

    // Wait for the new window and TUI to initialize
    println!("Waiting for TUI to launch in new window...");
    thread::sleep(Duration::from_secs(10));

    // Find the process ID of the new Terminal window running the TUI
    // This is tricky; for now, we will use screencapture's interactive window selection.
    println!("Please click on the gnostr TUI window to capture it.");
    let mut capture_cmd = Command::new("screencapture");
    capture_cmd.arg("-w")
               .arg(screenshot_path_str);
    
    let capture_status = capture_cmd.status().expect("Failed to execute screencapture");

    if capture_status.success() {
        println!("Screenshot saved to {}", screenshot_path_str);
    } else {
        eprintln!("Failed to capture screenshot.");
    }
}