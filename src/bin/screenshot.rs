use std::io::{self, Write};
use std::process::Command;

fn main() {
    if cfg!(target_os = "macos") {
        println!("This is running on macOS!");
        macos()
    //Ok(())
    } else {
        println!("This is not running on macOS.");
        linux()
        //Ok(())
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
            // Handles any incorrect input.
            println!("Please enter a correct input.\n");
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
    println!("[ 4 ] for Capturing to Clipboard\n");

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

            match clipboard_input {
                "a" => execute_macoscommand("screencapture", &["-c"]),
                "b" => execute_macoscommand("screencapture", &["-ic"]),
                "c" => execute_macoscommand("screencapture", &["-wc"]),
                _ => execute_macoscommand("screencapture", &["-c"]),
            }
        }
        _ => {
            // Handles any incorrect input.
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
