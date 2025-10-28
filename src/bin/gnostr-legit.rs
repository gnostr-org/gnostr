#![allow(unused)]
#![allow(dead_code)]

#![allow(unused)]
#![allow(dead_code)]

use std::io::{Result, self};
use std::env;
use std::process::Command;
use std::time::SystemTime;
use std::thread;
use std::convert::TryInto;

use clap::Parser;
use gnostr_legit::{command, gitminer};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The repository path (defaults to current directory)
    #[arg(default_value = ".")]
    repo: String,

    /// Prefix for the target hash
    #[arg(long, default_value = "000")]
    prefix: String,

    /// Commit message
    #[arg(short, long)]
    message: Option<String>,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let path = env::current_dir()?;

    let message = if let Some(msg) = args.message {
        msg
    } else {
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
        String::from_utf8(output.stdout)
            .map_err(|non_utf8| String::from_utf8_lossy(non_utf8.as_bytes()).into_owned())
            .unwrap()
    };

    let count = thread::available_parallelism()?.get();

    let opts = gitminer::Options {
        threads: count.try_into().unwrap(),
        target: args.prefix,
        message: message,
        repo: path.as_path().display().to_string(),
        timestamp: SystemTime::now(),
    };

    command::run_legit_command(opts)
}