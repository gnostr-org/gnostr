use std::io;
use std::process::Command;
use clap::{Parser, Subcommand, ValueEnum};


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Capture the full screen
    Full {
        /// (Linux only) The tool to use for screenshot
        #[arg(long, value_enum, default_value_t = Tool::Gnome)]
        tool: Tool,
        /// (macOS only) Output file name
        #[arg(default_value = "full_screen.png")]
        filename: String,
    },
    /// Capture a specific area
    Area {
        /// (Linux only) The tool to use for screenshot
        #[arg(long, value_enum, default_value_t = Tool::Gnome)]
        tool: Tool,
        /// (macOS and Linux/scrot) Output file name
        #[arg(default_value = "selected_area.png")]
        filename: String,
    },
    /// Capture a specific window
    Window {
        /// (Linux only) The tool to use for screenshot
        #[arg(long, value_enum, default_value_t = Tool::Gnome)]
        tool: Tool,
        /// (macOS only) Output file name
        #[arg(default_value = "specific_window.png")]
        filename: String,
    },
    /// Capture to clipboard (macOS only)
    Clipboard {
        #[command(subcommand)]
        command: ClipboardCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum ClipboardCommands {
    /// Capture full screen to clipboard
    Full,
    /// Capture area to clipboard
    Area,
    /// Capture window to clipboard
    Window,
}

#[derive(ValueEnum, Clone, Debug, PartialEq)]
pub enum Tool {
    Gnome,
    Scrot,
}

pub fn run() {
    let cli = Cli::parse();

    if cfg!(target_os = "macos") {
        macos(cli.command);
    } else {
        linux(cli.command);
    }
}

pub fn linux(command: Commands) {
    match command {
        Commands::Full { tool, .. } => {
            if tool == Tool::Gnome {
                execute_and_handle_linux("gnome-screenshot", &[]);
            } else {
                eprintln!("'scrot' does not have a dedicated full screen command. Use 'scrot <filename>' or 'scrot -s' to select the whole screen.");
            }
        }
        Commands::Area { tool, filename } => {
            match tool {
                Tool::Gnome => execute_and_handle_linux("gnome-screenshot", &["-a"]),
                Tool::Scrot => execute_and_handle_linux("scrot", &["-s", &filename]),
            }
        }
        Commands::Window { tool, .. } => {
             match tool {
                Tool::Gnome => execute_and_handle_linux("gnome-screenshot", &["-w"]),
                Tool::Scrot => execute_and_handle_linux("scrot", &["-s"]),
            }
        }
        Commands::Clipboard { .. } => {
            eprintln!("Clipboard capture is not implemented for Linux in this tool. You can pipe the output of scrot to xclip for example: `scrot -s -o /dev/stdout | xclip -selection clipboard -t image/png`");
        }
    }
}

pub fn macos(command: Commands) {
    match command {
        Commands::Full { filename, .. } => {
            execute_and_handle_macos("screencapture", &["-x", &filename]);
        }
        Commands::Area { filename, .. } => {
            execute_and_handle_macos("screencapture", &["-i", &filename]);
        }
        Commands::Window { filename, .. } => {
            execute_and_handle_macos("screencapture", &["-w", &filename]);
        }
        Commands::Clipboard { command } => match command {
            ClipboardCommands::Full => {
                execute_and_handle_macos("screencapture", &["-c"]);
            }
            ClipboardCommands::Area => {
                execute_and_handle_macos("screencapture", &["-ic"]);
            }
            ClipboardCommands::Window => {
                execute_and_handle_macos("screencapture", &["-wc"]);
            }
        },
    }
}

fn execute_and_handle_linux(program: &str, args: &[&str]) {
    println!("Executing command: {} {}", program, args.join(" "));
    match execute_linux_command(program, args) {
        Ok(_) => println!("Command executed successfully."),
        Err(e) => eprintln!("Command failed with error: {}", e),
    }
}

fn execute_and_handle_macos(program: &str, args: &[&str]) {
    println!("\nExecuting command: {} {}", program, args.join(" "));
    match execute_macos_command(program, args) {
        Ok(_) => println!("Command executed successfully."),
        Err(e) => eprintln!("Command failed with error: {}", e),
    }
}


pub fn take_screenshot(output_path: &str) -> io::Result<()> {
    if cfg!(target_os = "macos") {
        macos_simple(output_path)
    } else if cfg!(target_os = "linux") {
        linux_simple(output_path)
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "Unsupported operating system for screenshots",
        ))
    }
}

fn linux_simple(file_path: &str) -> io::Result<()> {
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

fn macos_simple(file_path: &str) -> io::Result<()> {
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

use std::path::PathBuf;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

/// # Captures a screenshot for debugging purposes during a test.
///
/// This function is designed to be called from other tests to capture the UI
/// state at a specific moment. The screenshot is saved in the `test_screenshots`
/// directory with a filename that includes the provided context along with
/// a timestamp.
///
/// ## Platform
///
/// This utility is only available and compiled on **macOS**.
///
/// ## Error Handling
///
/// This function will not fail the test if the screenshot cannot be taken;
/// it will return an `Err` instead. The calling test can then decide how to
/// handle the failure.
///
/// ## File Management
///
/// The screenshot file is not deleted after being taken, so it can be
/// inspected after the test run.
///
/// ## Example
///
/// ```
/// #[test]
/// #[cfg(target_os = "macos")]
/// fn my_tui_test() -> Result<(), Box<dyn std::error::Error>> {
///     let mut cmd = Command::new(cargo_bin("gnostr"));
///     cmd.arg("tui");
///
///     // Spawn the command as a child process
///     let mut child = cmd.spawn().expect("Failed to spawn gnostr command");
///
///     // Give the TUI a moment to initialize
///     std::thread::sleep(std::time::Duration::from_secs(2));
///
///     // Capture the screenshot
///     let screenshot_path_result = gnostr::utils::screenshot::make_screenshot("my_tui_test");
///
///     // Terminate the child process
///     child.kill().expect("Failed to kill gnostr process");
///
///     // Assert that the screenshot was created
///     assert!(screenshot_path_result.is_ok(), "Failed to capture screenshot.");
///
///     Ok(())
/// }
/// ```
#[cfg(target_os = "macos")]
pub fn make_screenshot(context: &str) -> io::Result<PathBuf> {
    let mut screenshot_path = std::env::current_dir()?;
    screenshot_path.push("test_screenshots");
    fs::create_dir_all(&screenshot_path)?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?
        .as_secs();

    let filename = format!("test_screenshot_{}_{}.png", context, timestamp);
    screenshot_path.push(&filename);

    take_screenshot(screenshot_path.to_str().unwrap())?;

    Ok(screenshot_path)
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_make_screenshot_macos() {
        // This test now verifies that our new screenshot utility works correctly.
        let screenshot_path =
            make_screenshot("self_test").expect("Failed to capture screenshot during self-test");

        // --- Verify ---
        let metadata =
            fs::metadata(&screenshot_path).expect("Failed to get screenshot metadata");
        assert!(metadata.is_file(), "Screenshot is not a file");
        assert!(metadata.len() > 0, "Screenshot file is empty");

        // --- Teardown ---
        // DO NOT DELETE THE SCREEN SHOT!
    }
}
