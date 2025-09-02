use std::io::{self, Write};
use std::process::Command;

fn main() {
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
            execute_command("screencapture", &["-x", "full_screen.png"]);
        }
        "2" => {
            // Captures a specific area. This is an interactive command.
            execute_command("screencapture", &["-i", "selected_area.png"]);
        }
        "3" => {
            // Captures a specific window. This is an interactive command.
            execute_command("screencapture", &["-w", "specific_window.png"]);
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
            io::stdin().read_line(&mut clipboard_input).expect("Failed to read line");
            let clipboard_input = clipboard_input.trim();

            match clipboard_input {
                "a" => execute_command("screencapture", &["-c"]),
                "b" => execute_command("screencapture", &["-ic"]),
                "c" => execute_command("screencapture", &["-wc"]),
                _ => println!("Invalid choice."),
            }
        }
        _ => {
            // Handles any incorrect input.
            execute_command("screencapture", &["-x", "full_screen.png"]);
        }
    }
}

// A helper function to execute the external command and handle errors.
fn execute_command(program: &str, args: &[&str]) {
    println!("\nExecuting command: {} {}", program, args.join(" "));

    let output = Command::new(program)
        .args(args)
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                println!("Command executed successfully.");
                if !output.stdout.is_empty() {
                    println!("{}", String::from_utf8_lossy(&output.stdout));
                }
            } else {
                eprintln!("Command failed with error: {}", String::from_utf8_lossy(&output.stderr));
            }
        }
        Err(e) => {
            eprintln!("Failed to execute command '{}': {}", program, e);
        }
    }
}

