use clipboard::ClipboardContext;
use clipboard::ClipboardProvider;
use gnostr::utils::screenshot::{execute_linux_command, execute_macos_command};
use std::io::{self, Write};

fn main() {
    if cfg!(target_os = "macos") {
        macos();
    } else {
        linux();
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
            execute_and_handle_linux("gnome-screenshot", &[]);
        }
        "2" => {
            // scrot -s - Captures an active window or selected area.
            execute_and_handle_linux("scrot", &["-s"]);
        }
        "3" => {
            // gnome-screenshot -w - Captures the active window.
            execute_and_handle_linux("gnome-screenshot", &["-w"]);
        }
        "4" => {
            // scrot -s myscreenshot.png - Captures a specific area and names the file.
            execute_and_handle_linux("scrot", &["-s", "myscreenshot.png"]);
        }
        "5" => {
            // gnome-screenshot -a - Captures a specific area.
            execute_and_handle_linux("gnome-screenshot", &["-a"]);
        }
        _ => {
            // default
            execute_and_handle_linux("gnome-screenshot", &[]);
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
            execute_and_handle_macos("screencapture", &["-x", "full_screen.png"]);
        }
        "2" => {
            // Captures a specific area. This is an interactive command.
            execute_and_handle_macos("screencapture", &["-i", "selected_area.png"]);
        }
        "3" => {
            // Captures a specific window. This is an interactive command.
            execute_and_handle_macos("screencapture", &["-w", "specific_window.png"]);
        }
        "4" => {
            // Captures to the clipboard.
            let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
            let mut alt_ctx: ClipboardContext = ClipboardProvider::new().unwrap();
            println!("Choose an option to capture to the clipboard:");
            println!("  [a] for whole screen");
            println!("  [b] for a specific area");
            println!("  [c] for a specific window");
            print!("Enter your choice: ");
            io::stdout().flush().unwrap();

            let mut clipboard_input_1 = String::new();
            io::stdin()
                .read_line(&mut clipboard_input_1)
                .expect("Failed to read line");
            let clipboard_input = clipboard_input_1.trim();
            println!("clipboard_input_1={}", clipboard_input_1.trim());

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
                println!("display alt_ctx1:is some!");
                match alt_ctx.get_contents() {
                    Ok(contents) => {
                        println!("display alt_ctx1:match!");
                        println!("display alt_ctx1:Clipboard contents:\n{}", contents);
                    }
                    Err(e) => {
                        eprintln!("display alt_ctx1:Failed to get clipboard contents: {}", e);
                    }
                }
            }
            if Some(ctx.get_contents()).is_some() {
                println!("1b:is some!");
                match ctx.get_contents() {
                    Ok(contents) => {
                        println!("1b:match!");
                        println!("1b:Clipboard contents:\n{}", contents);
                    }
                    Err(e) => {
                        eprintln!("1b:Failed to get clipboard contents: {}", e);
                    }
                }
                println!("display alt_ctx1b:is some!");
                match alt_ctx.get_contents() {
                    Ok(contents) => {
                        println!("display alt_ctx1b:match!");
                        println!("display alt_ctx1b:Clipboard contents:\n{}", contents);
                    }
                    Err(e) => {
                        eprintln!("display alt_ctx1b:Failed to get clipboard contents: {}", e);
                    }
                }
            }

            alt_ctx
                .set_contents(format!(
                    "formatted! {}",
                    clipboard_input_1.clone()
                ))
                .expect("set_result failed!");

            if Some(alt_ctx.get_contents()).is_some() {
                println!("1b_alt_ctx:is some!");
                match alt_ctx.get_contents() {
                    Ok(contents) => {
                        println!("1b_alt_ctx:match!");
                        println!("1b_alt_ctx:Clipboard contents:\n{}", contents);
                    }
                    Err(e) => {
                        eprintln!("1b_alt_ctx:Failed to get clipboard contents: {}", e);
                    }
                }
            }

            match clipboard_input {
                "a" => execute_and_handle_macos("screencapture", &["-c"]),
                "b" => execute_and_handle_macos("screencapture", &["-ic"]),
                "c" => execute_and_handle_macos("screencapture", &["-wc"]),
                // default
                _ => execute_and_handle_macos("screencapture", &["-c"]),
            }
        }
        _ => {
            // default
            execute_and_handle_macos("screencapture", &["-x", "full_screen.png"]);
        }
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