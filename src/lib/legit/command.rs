#![allow(unused)]
#![allow(dead_code)]
extern crate chrono;
use std::process::Command;
use chrono::offset::Utc;
use chrono::DateTime;
use std::io::{Result};
use std::env;
use std::time::{Duration, SystemTime};
use std::thread::sleep;
use std::convert::TryInto;
use std::any::type_name;
use std::{io, thread};
use argparse::{ArgumentParser,Store};
use gnostr_legit::gitminer::Gitminer;
use git2::*;
use sha2::{Sha256, Digest};
use pad::{PadStr, Alignment};
use time::OffsetDateTime;
use time::macros::datetime;

use gnostr_legit::{gitminer, repo, worker};

fn type_of<T>(_: T) -> &'static str {
    type_name::<T>()
}

pub fn run_legit_command(mut opts: gitminer::Options) -> io::Result<()> {

    let start = SystemTime::now();
    let system_time = SystemTime::now();

    let repo = Repository::open(&opts.repo).expect("Couldn't open repository");

    if repo.state() != RepositoryState::Clean {
        let repo_state =
            if cfg!(target_os = "windows") {
            Command::new("cmd")
                    .args(["/C", "git status"])
                    .output()
                    .expect("failed to execute process")
            } else {
            Command::new("sh")
                    .arg("-c")
                    .arg("gnostr-git diff")
                    .output()
                    .expect("failed to execute process")
            };

        let state = String::from_utf8(repo_state.stdout)
        .map_err(|non_utf8| String::from_utf8_lossy(non_utf8.as_bytes()).into_owned())
        .unwrap();
    }

    if opts.message.is_empty() {
        let output =
            if cfg!(target_os = "windows") {
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
        opts.message = [message.to_string()].to_vec();
    }

    let mut miner = match Gitminer::new(opts.clone()) {
        Ok(m)  => m,
        Err(e) => { panic!("Failed to start git miner: {}", e); }
    };

    let hash = match miner.mine() {
        Ok(s)  => s,
        Err(e) => { panic!("Failed to generate commit: {}", e); }
    };

    let mut hasher = Sha256::new();
    hasher.update(&hash);
    let gnostr_sec: String = format!("{:X}", hasher.finalize());

    let shell_test =
        if cfg!(target_os = "windows") {
        Command::new("cmd")
                .args(["/C", "gnostr --hash 0"])
                .output()
                .expect("failed to execute process")
        } else
        if cfg!(target_os = "macos"){
        Command::new("sh")
                .args(["-c", "gnostr --hash 0"])
                .output()
                .expect("failed to execute process")
        } else
        if cfg!(target_os = "linux"){
        Command::new("sh")
                .args(["-c", "gnostr --hash 0"])
                .output()
                .expect("failed to execute process")
        } else {
        Command::new("sh")
                .args(["-c", "gnostr --hash 0"])
                .output()
                .expect("failed to execute process")
        };

    let gnostr_test = String::from_utf8(shell_test.stdout)
    .map_err(|non_utf8| String::from_utf8_lossy(non_utf8.as_bytes()).into_owned())
    .unwrap();

    let duration = SystemTime::now().duration_since(start).expect("Time went backwards");
    println!("{}", gnostr_test);
    Ok(())

}


