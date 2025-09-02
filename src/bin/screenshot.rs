use clipboard::ClipboardContext;
use clipboard::ClipboardProvider;
use std::io::{self, Write};
use std::process::Command;

fn main() {
    if cfg!(target_os = "macos") {
        macos()
    } else {
        linux()
    }
}

fn linux() {
    // Present the menu to the user.
    println!("\n[ 1 ] for Capturing the whole Screen using gnome-screenshot");
    println!("[ 2 ] for Capturing the Active window using the Scrot");
    println!("[ 3 ] for Capturing the Active window using the gnome-screenshot");
    println!("[ 4 ] for Capturing the Specific Area using the Scrot");
    println!("[ 5 ] for Capturing the Specific Area using the gnome-screenshot\n");

    // Read the user input.
    print!("Enter your choice: ");
    io::stdout().flush().unwrap(); // Ensure the prompt is printed before reading input.

    let mut user_input = String::new();
    io::stdin()
        .read_line(&mut user_input)
        .expect("Failed to read line");

    // Trim whitespace and handle the input.
    let user_input = user_input.trim();

    // Use a match statement to handle the user's choice.
    match user_input {
        "1" => {
            // gnome-screenshot - Captures the whole screen.
            execute_linuxcommand("gnome-screenshot", &[]);
        }
        "2" => {
            // scrot -s - Captures an active window or selected area.
            execute_linuxcommand("scrot", &["-s"]);
        }
        "3" => {
            // gnome-screenshot -w - Captures the active window.
            execute_linuxcommand("gnome-screenshot", &["-w"]);
        }
        "4" => {
            // scrot -s myscreenshot.png - Captures a specific area and names the file.
            execute_linuxcommand("scrot", &["-s", "myscreenshot.png"]);
        }
        "5" => {
            // gnome-screenshot -a - Captures a specific area.
            execute_linuxcommand("gnome-screenshot", &["-a"]);
        }
        _ => {
            // default
            execute_linuxcommand("gnome-screenshot", &[]);
        }
    }
}

// A helper function to execute the external command and handle errors.
fn execute_linuxcommand(program: &str, args: &[&str]) {
    println!("Executing command: {} {}", program, args.join(" "));

    let output = Command::new(program).args(args).output();

    match output {
        Ok(output) => {
            if output.status.success() {
                println!("Command executed successfully.");
                if !output.stdout.is_empty() {
                    println!("{}", String::from_utf8_lossy(&output.stdout));
                }
            } else {
                eprintln!(
                    "Command failed with error: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
        Err(e) => {
            eprintln!("Failed to execute command '{}': {}", program, e);
        }
    }
}

fn macos() {
    // Present the menu to the user.
    println!("\n[ 1 ] for Capturing the whole Screen");
    println!("[ 2 ] for Capturing the Specific Area");
    println!("[ 3 ] for Capturing a specific Window");
    println!("[ 4 ] for Capturing from Clipboard\n");

    // Read the user input.
    print!("Enter your choice: ");
    io::stdout().flush().unwrap(); // Ensure the prompt is printed before reading input.

    let mut user_input = String::new();
    io::stdin()
        .read_line(&mut user_input)
        .expect("Failed to read line");

    // Trim whitespace and handle the input.
    let user_input = user_input.trim();

    // Use a match statement to handle the user's choice.
    match user_input {
        "1" => {
            // Captures the whole screen and saves it to the desktop.
            execute_macoscommand("screencapture", &["-x", "full_screen.png"]);
        }
        "2" => {
            // Captures a specific area. This is an interactive command.
            execute_macoscommand("screencapture", &["-i", "selected_area.png"]);
        }
        "3" => {
            // Captures a specific window. This is an interactive command.
            execute_macoscommand("screencapture", &["-w", "specific_window.png"]);
        }
        "4" => {
            // Captures to the clipboard.
            let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
            println!("Choose an option to capture to the clipboard:");
            println!("  [a] for whole screen");
            println!("  [b] for a specific area");
            println!("  [c] for a specific window");
            print!("Enter your choice: ");
            io::stdout().flush().unwrap();

            let mut clipboard_input = String::new();
            io::stdin()
                .read_line(&mut clipboard_input)
                .expect("Failed to read line");
            let clipboard_input = clipboard_input.trim();

			if Some(ctx.get_contents()).is_some() {
				println!("1:is some!");
            match ctx.get_contents() {
                Ok(contents) => {
				println!("1:match!");
                    println!("1:Clipboard contents:\n{}", contents);
                }
                Err(e) => {
                    eprintln!("1:Failed to get clipboard contents: {}", e);
                }
            }
			}
            match clipboard_input {
                "a" => execute_macoscommand("screencapture", &["-c"]),
                "b" => execute_macoscommand("screencapture", &["-ic"]),
                "c" => execute_macoscommand("screencapture", &["-wc"]),
                // default
                _ => execute_macoscommand("screencapture", &["-c"]),
            }
            //let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();

			if Some(ctx.get_contents()).is_some() {
				println!("2:is some!");
            // Get the text from the clipboard.
            match ctx.get_contents() {
                Ok(contents) => {
				println!("2:match!");
                    println!("2:Clipboard contents:\n{}", contents);
                }
                Err(e) => {
                    eprintln!("2:Failed to get clipboard contents: {}", e);
                }
            }
            }
        }
        _ => {
            // default
            execute_macoscommand("screencapture", &["-x", "full_screen.png"]);
        }
    }
}

// A helper function to execute the external command and handle errors.
fn execute_macoscommand(program: &str, args: &[&str]) {
    println!("\nExecuting command: {} {}", program, args.join(" "));

    let output = Command::new(program).args(args).output();

    match output {
        Ok(output) => {
            if output.status.success() {
                println!("Command executed successfully.");
                if !output.stdout.is_empty() {
                    println!("{}", String::from_utf8_lossy(&output.stdout));
                }
            } else {
                eprintln!(
                    "Command failed with error: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
        Err(e) => {
            eprintln!("Failed to execute command '{}': {}", program, e);
        }
    }
}
