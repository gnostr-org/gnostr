// build.rs

use std::process::Command;

fn main() {

	let build_tui_command = if cfg!(target_os = "windows") {
		Command::new("cmd")
			.args(["/C", "cargo", "install", "--path", "tui"])
			.output()
			.expect("failed to execute process")
	} else if cfg!(target_os = "macos") {
		Command::new("sh")
			.arg("-c")
			.args(["cargo", "install", "--path", "tui"])
			.output()
			.expect("failed to execute process")
	} else if cfg!(target_os = "linux") {
		Command::new("sh")
			.arg("-c")
			.args(["cargo", "install", "--path", "tui"])
			.output()
			.expect("failed to execute process")
	} else {
		Command::new("sh")
			.arg("-c")
			.args(["cargo", "install", "--path", "tui"])
			.output()
			.expect("failed to execute process")
	};

	let _build_tui = String::from_utf8(build_tui_command.stdout)
		.map_err(|non_utf8| {
			String::from_utf8_lossy(non_utf8.as_bytes()).into_owned()
		})
		.unwrap();
}

