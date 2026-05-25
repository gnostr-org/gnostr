fn main() {
    if !gnostr_tray_icon::command_exists("gnostr") {
        eprintln!("gnostr not found on PATH");
        std::process::exit(1);
    }

    let gitdir = std::env::var("GNOSTR_GITDIR").unwrap_or_else(|_| ".".to_string());

    let mut command = gnostr_tray_icon::pty_command("gnostr");
    command.arg("tui");
    command.arg("--gitdir");
    command.arg(&gitdir);
    command.arg("--help");
    command.env("GNOSTR_GITDIR", &gitdir);
    command.cwd(&gitdir);

    match gnostr_tray_icon::run_command_in_pty(command) {
        Ok(output) => {
            print!("{output}");
        }
        Err(error) => {
            eprintln!("failed to launch gnostr: {error}");
            std::process::exit(1);
        }
    }
}
