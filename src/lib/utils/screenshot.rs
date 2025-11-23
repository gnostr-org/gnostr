use std::process::Command;
use std::io;

pub fn take_screenshot(output_path: &str) -> io::Result<()> {
    if cfg!(target_os = "macos") {
        macos(output_path)
    } else if cfg!(target_os = "linux") {
        linux(output_path)
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "Unsupported operating system for screenshots",
        ))
    }
}

fn linux(file_path: &str) -> io::Result<()> {
    // Attempt to use gnome-screenshot first
    if command_exists("gnome-screenshot") {
        return execute_linux_command("gnome-screenshot", &["-f", file_path]);
    }
    // Fallback to scrot
    if command_exists("scrot") {
        return execute_linux_command("scrot", &[file_path]);
    }
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "No screenshot tool (gnome-screenshot, scrot) found on Linux.",
    ))
}

fn macos(file_path: &str) -> io::Result<()> {
    execute_macos_command("screencapture", &["-x", file_path])
}

fn command_exists(command: &str) -> bool {
    Command::new("which")
        .arg(command)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_or(false, |s| s.success())
}

pub fn execute_linux_command(program: &str, args: &[&str]) -> io::Result<()> {
    let output = Command::new(program).args(args).output()?;
    if output.status.success() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Command failed: {} {}",
                program,
                String::from_utf8_lossy(&output.stderr)
            ),
        ))
    }
}

pub fn execute_macos_command(program: &str, args: &[&str]) -> io::Result<()> {
    let output = Command::new(program).args(args).output()?;
    if output.status.success() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Command failed: {} {}",
                program,
                String::from_utf8_lossy(&output.stderr)
            ),
        ))
    }
}

