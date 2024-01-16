// build.rs

use std::process::Command;

fn main() {

    //build bins
	let build_bins_command = if cfg!(target_os = "windows") {
		Command::new("cmd")
			.args(["/C", "cargo", "install", "--path", "bins"])
			.output()
			.expect("failed to execute process")
	} else if cfg!(target_os = "macos") {
		Command::new("cargo")
			.args(["install", "--path", "bins"])
			.output()
			.expect("failed to execute process")
	} else if cfg!(target_os = "linux") {
		Command::new("cargo")
			.args(["install", "--path", "bins"])
			.output()
			.expect("failed to execute process")
	} else {
		Command::new("cargo")
			.args(["install", "--path", "bins"])
			.output()
			.expect("failed to execute process")
	};

	let _build_bins = String::from_utf8(build_bins_command.stdout)
		.map_err(|non_utf8| {
			String::from_utf8_lossy(non_utf8.as_bytes()).into_owned()
		})
		.unwrap();

    //build bits
	let build_bits_command = if cfg!(target_os = "windows") {
		Command::new("cmd")
			.args(["/C", "cargo", "install", "--path", "bits/crates/rqbit"])
			.output()
			.expect("failed to execute process")
	} else if cfg!(target_os = "macos") {
		Command::new("cargo")
			.args(["install", "--path", "bits/crates/rqbit"])
			.output()
			.expect("failed to execute process")
	} else if cfg!(target_os = "linux") {
		Command::new("cargo")
			.args(["install", "--path", "bits/crates/rqbit"])
			.output()
			.expect("failed to execute process")
	} else {
		Command::new("cargo")
			.args(["install", "--path", "bits/crates/rqbit"])
			.output()
			.expect("failed to execute process")
	};

	let _build_bits = String::from_utf8(build_bits_command.stdout)
		.map_err(|non_utf8| {
			String::from_utf8_lossy(non_utf8.as_bytes()).into_owned()
		})
		.unwrap();

    //build tui
	let build_tui_command = if cfg!(target_os = "windows") {
		Command::new("cmd")
			.args(["/C", "cargo", "install", "--path", "tui"])
			.output()
			.expect("failed to execute process")
	} else if cfg!(target_os = "macos") {
		Command::new("cargo")
			.args(["install", "--path", "tui"])
			.output()
			.expect("failed to execute process")
	} else if cfg!(target_os = "linux") {
		Command::new("cargo")
			.args(["install", "--path", "tui"])
			.output()
			.expect("failed to execute process")
	} else {
		Command::new("cargo")
			.args(["install", "--path", "tui"])
			.output()
			.expect("failed to execute process")
	};

	let _build_tui = String::from_utf8(build_tui_command.stdout)
		.map_err(|non_utf8| {
			String::from_utf8_lossy(non_utf8.as_bytes()).into_owned()
		})
		.unwrap();

    //build modal
	let build_modal_command = if cfg!(target_os = "windows") {
		Command::new("cmd")
			.args(["/C", "cargo", "install", "--path", "modal"])
			.output()
			.expect("failed to execute process")
	} else if cfg!(target_os = "macos") {
		Command::new("cargo")
			.args(["install", "--path", "modal"])
			.output()
			.expect("failed to execute process")
	} else if cfg!(target_os = "linux") {
		Command::new("cargo")
			.args(["install", "--path", "modal"])
			.output()
			.expect("failed to execute process")
	} else {
		Command::new("sh")
			.arg("-c")
			.args(["cargo", "install", "--path", "modal"])
			.output()
			.expect("failed to execute process")
	};

	let _build_modal = String::from_utf8(build_modal_command.stdout)
		.map_err(|non_utf8| {
			String::from_utf8_lossy(non_utf8.as_bytes()).into_owned()
		})
		.unwrap();



}

