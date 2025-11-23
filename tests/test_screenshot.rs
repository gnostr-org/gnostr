use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use std::path::PathBuf;
use std::fs;

fn is_tmux_installed() -> bool {
    Command::new("which")
        .arg("tmux")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_or(false, |s| s.success())
}

#[test]
fn test_capture_tui() {
    if !is_tmux_installed() {
        println!("Skipping test_capture_tui: tmux is not installed.");
        return;
    }

    let session_name = "gnostr-tui-test";
    let screenshot_path: PathBuf = [
        "/Users/randymcmillan/.gemini/tmp/e2231b79f1bd8b2a6ddfa2b8e1288337b83b48b9c931587d9e416bcd03fb2b14",
        "tui_screenshot.txt",
    ]
    .iter()
    .collect();
    let screenshot_path_str = screenshot_path.to_str().expect("Path is not valid UTF-8");

    // Start a new detached tmux session
    let mut cmd = Command::new("tmux");
    cmd.arg("new-session")
        .arg("-d")
        .arg("-s")
        .arg(session_name)
        .arg("cargo")
        .arg("run")
        .arg("--bin")
        .arg("gnostr")
        .arg("--")
        .arg("tui");

    let status = cmd.status().expect("Failed to start tmux session");
    assert!(status.success(), "Failed to start tmux session with gnostr tui");

    // Wait for the TUI to initialize
    thread::sleep(Duration::from_secs(5));

    // Capture the tmux pane content
    let mut capture_cmd = Command::new("tmux");
    capture_cmd.arg("capture-pane")
        .arg("-p")
        .arg("-t")
        .arg(session_name);

    let output = capture_cmd.output().expect("Failed to capture tmux pane");
    assert!(output.status.success(), "Failed to capture tmux pane content");

    fs::write(&screenshot_path, output.stdout).expect("Failed to write screenshot to file");
    
    // Kill the tmux session
    let mut kill_cmd = Command::new("tmux");
    kill_cmd.arg("kill-session").arg("-t").arg(session_name);
    let kill_status = kill_cmd.status().expect("Failed to kill tmux session");
    assert!(kill_status.success(), "Failed to kill tmux session");

    // Verify that the screenshot was created
    assert!(screenshot_path.exists(), "Screenshot file was not created");
}
