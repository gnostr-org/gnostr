use clap::{Parser, Subcommand, ValueEnum};
use gnostr::utils::screenshot::{execute_linux_command, execute_macos_command};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
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
enum ClipboardCommands {
    /// Capture full screen to clipboard
    Full,
    /// Capture area to clipboard
    Area,
    /// Capture window to clipboard
    Window,
}

#[derive(ValueEnum, Clone, Debug, PartialEq)]
enum Tool {
    Gnome,
    Scrot,
}


fn main() {
    let cli = Cli::parse();

    if cfg!(target_os = "macos") {
        macos(cli.command);
    } else {
        linux(cli.command);
    }
}

fn linux(command: Commands) {
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

fn macos(command: Commands) {
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
