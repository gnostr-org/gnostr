use std::borrow::Cow;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use tmux_interface::{CapturePane, KillSession, NewSession, SendKeys, Tmux};

fn is_tmux_installed() -> bool {
    Command::new("which")
        .arg("tmux")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

#[test]
fn test_capture_tmux() {
    if !is_tmux_installed() {
        println!("Skipping test_capture_tmux: tmux is not installed.");
        return;
    }

    let session_name = "gnostr-tmux-test";
    let screenshot_path: PathBuf = [
        "/Users/randymcmillan/.gemini/tmp/e2231b79f1bd8b2a6ddfa2b8e1288337b83b48b9c931587d9e416bcd03fb2b14",
        "tmux_test.txt",
    ]
    .iter()
    .collect();

    // Start a new detached tmux session
    let new_session = NewSession {
        detached: true,
        session_name: Some(Cow::from(session_name)),
        ..Default::default()
    };
    let tmux_output = Tmux::with_command(new_session).output().unwrap();
    if !tmux_output.status().success() {
        panic!(
            "Failed to start tmux session: {}",
            String::from_utf8_lossy(&tmux_output.stderr())
        );
    }

    // Send a simple echo command
    let send_keys_cmd = SendKeys {
        target_pane: Some(Cow::from(format!("{}:", session_name))),
        key: Some(Cow::from("echo 'hello tmux'")),
        ..Default::default()
    };
    let tmux_output = Tmux::with_command(send_keys_cmd).output().unwrap();
    if !tmux_output.status().success() {
        panic!(
            "Failed to send command to tmux session: {}",
            String::from_utf8_lossy(&tmux_output.stderr())
        );
    }

    let send_keys_enter = SendKeys {
        target_pane: Some(Cow::from(format!("{}:", session_name))),
        key: Some(Cow::from("C-m")),
        ..Default::default()
    };
    let tmux_output = Tmux::with_command(send_keys_enter).output().unwrap();
    if !tmux_output.status().success() {
        panic!(
            "Failed to send Enter to tmux session: {}",
            String::from_utf8_lossy(&tmux_output.stderr())
        );
    }

    // Wait for the command to execute
    thread::sleep(Duration::from_secs(1));

    // Capture the tmux pane content
    let capture_pane = CapturePane {
        target_pane: Some(Cow::from(format!("{}:", session_name))),
        ..Default::default()
    };
    let captured = Tmux::with_command(capture_pane).output().unwrap();
    if !captured.status().success() {
        panic!(
            "Failed to capture tmux pane: {}",
            String::from_utf8_lossy(&captured.stderr())
        );
    }

    fs::write(&screenshot_path, captured.stdout()).expect("Failed to write screenshot to file");

    // Kill the tmux session
    let kill_session = KillSession {
        target_session: Some(Cow::from(session_name)),
        ..Default::default()
    };
    let tmux_output = Tmux::with_command(kill_session).output().unwrap();
    if !tmux_output.status().success() {
        panic!(
            "Failed to kill tmux session: {}",
            String::from_utf8_lossy(&tmux_output.stderr())
        );
    }

    // Verify that the screenshot was created
    assert!(screenshot_path.exists(), "Screenshot file was not created");
}
