#![allow(unused)]
#![allow(dead_code)]

use std::io::{Result, self};
use std::env;
use std::process::Command;
use std::time::SystemTime;
use std::thread;
use std::convert::TryInto;

use gnostr_legit::{command, gitminer};

fn main() -> io::Result<()> {
    let path = env::current_dir()?;

    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", "git status"])
            .output()
            .expect("failed to execute process")
    } else {
        Command::new("sh")
            .arg("-c")
            .arg("git diff")
            .output()
            .expect("failed to execute process")
    };

    let message = String::from_utf8(output.stdout)
        .map_err(|non_utf8| String::from_utf8_lossy(non_utf8.as_bytes()).into_owned())
        .unwrap();

    let count = thread::available_parallelism()?.get();

    let opts = gitminer::Options {
        threads: count.try_into().unwrap(),
        target: "00000".to_string(),
        message: message,
        repo: path.as_path().display().to_string(),
        timestamp: SystemTime::now(),
    };

    command::run_legit_command(opts)
}