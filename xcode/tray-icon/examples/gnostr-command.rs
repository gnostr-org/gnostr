fn main() {
    let command_line = std::env::args().skip(1).collect::<Vec<_>>().join(" ");
    let command_line = if command_line.trim().is_empty() {
        "echo hello from gnostr".to_string()
    } else {
        command_line
    };

    match gnostr_tray_icon::spawn_terminal_command(&command_line) {
        Ok(mut child) => {
            let _ = child.wait();
        }
        Err(error) => {
            eprintln!("failed to launch terminal: {error}");
            std::process::exit(1);
        }
    }
}
