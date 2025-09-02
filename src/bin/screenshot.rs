use std::io::{self, Write};
use std::process::Command;

fn main() {
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
            execute_command("gnome-screenshot", &[]);
        }
        "2" => {
            // scrot -s - Captures an active window or selected area.
            execute_command("scrot", &["-s"]);
        }
        "3" => {
            // gnome-screenshot -w - Captures the active window.
            execute_command("gnome-screenshot", &["-w"]);
        }
        "4" => {
            // scrot -s myscreenshot.png - Captures a specific area and names the file.
            execute_command("scrot", &["-s", "myscreenshot.png"]);
        }
        "5" => {
            // gnome-screenshot -a - Captures a specific area.
            execute_command("gnome-screenshot", &["-a"]);
        }
        _ => {
            // Handles any incorrect input.
            println!("Please enter a correct input.\n");
        }
    }
}

// A helper function to execute the external command and handle errors.
fn execute_command(program: &str, args: &[&str]) {
    println!("Executing command: {} {}", program, args.join(" "));

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
