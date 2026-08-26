fn main() {
    if !gnostr_tray_icon::command_exists("gnostr") {
        eprintln!("gnostr not found on PATH");
        std::process::exit(1);
    }

    match std::process::Command::new("gnostr").arg("--help").output() {
        Ok(output) => {
            print!("{}", String::from_utf8_lossy(&output.stdout));
            eprint!("{}", String::from_utf8_lossy(&output.stderr));
            if !output.status.success() {
                eprintln!("gnostr exited with status: {}", output.status);
                std::process::exit(1);
            }
        }
        Err(error) => {
            eprintln!("failed to launch gnostr: {error}");
            std::process::exit(1);
        }
    }
}
