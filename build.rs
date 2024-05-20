use std::process::Command;
use std::{env, fs};

use include_dir::{include_dir, Dir};
//use std::path::Path;
use markdown::to_html;

//static PROJECT_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR");

fn main() -> std::io::Result<()> {
    let _out_dir = env::var("OUT_DIR").unwrap();

    Command::new("git")
        .args(&[
            "remote",
            "add",
            "randymcmillan/nostril",
            "git@github.com:randymcmillan/nostril.git",
        ])
        .status()
        .unwrap();
    Command::new("git")
        .args(&[
            "remote",
            "add",
            "jb55/nostril",
            "git@github.com:jb55/nostril.git",
        ])
        .status()
        .unwrap();
    Command::new("git")
        .args(&["fetch", "--all"])
        .status()
        .unwrap();
    Command::new("git")
        .args(&["fetch", "--all", "--tags", "--force"])
        .status()
        .unwrap();

    let script_name = "./script.sh";

    // Build the command
    let mut _command = Command::new(script_name);

    // Add arguments if needed (optional)
    // command.arg("argument1");
    // command.arg("argument2");

    Command::new(script_name)
        .current_dir(".")
        .spawn()
        .expect("script.sh command failed to start");
    Ok(())
}
