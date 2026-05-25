fn main() {
    if !gnostr_tray_icon::command_exists("gnostr") {
        eprintln!("gnostr not found on PATH");
        std::process::exit(1);
    }

    match gnostr_tray_icon::system_command("gnostr").arg("--help").spawn() {
        Ok(mut child) => {
            let _ = child.wait();
        }
        Err(error) => {
            eprintln!("failed to launch gnostr: {error}");
            std::process::exit(1);
        }
    }
}
