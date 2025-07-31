#![allow(unused)]
#![allow(dead_code)]
extern crate chrono;
use chrono::offset::Utc;
use chrono::DateTime;
use log::debug;
use std::process::Command;
//use std::time::SystemTime;
use std::any::type_name;
use std::convert::TryInto;
use std::env;
use std::io::Result;
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
//use std::mem::size_of;
use argparse::{ArgumentParser, Store};
use git2::*;
use gnostr::get_pwd;
use gnostr::legit::gitminer;
use gnostr::legit::gitminer::Gitminer;
use gnostr::legit::post_event;
use gnostr::legit::repo;
use gnostr::legit::worker;
use gnostr_types::Event;
use gnostr_types::EventV3;
use pad::{Alignment, PadStr};
use sha2::{Digest, Sha256};
use std::{io, thread};

use std::path::PathBuf; //for get_current_dir

//pub mod gitminer;
//pub mod repo;
//pub mod worker;

//fn type_of<T>(_: T) -> &'static str {
//    type_name::<T>()
//}

fn get_epoch_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

fn convert_to_u32(v: usize) -> Option<i8> {
    if v > (std::i8::MAX as i32).try_into().unwrap() {
        None
    } else {
        Some(v as i8)
    }
}

fn get_current_working_dir() -> std::io::Result<PathBuf> {
    env::current_dir()
}

#[cfg(debug_assertions)]
fn example() {
    debug!("Debugging enabled");
    debug!("cwd={:?}", get_current_working_dir());
}

#[cfg(not(debug_assertions))]
fn example() {
    debug!("Debugging disabled");
    debug!("cwd={:?}", get_current_working_dir());
}

fn main() -> io::Result<()> {
    #[allow(clippy::if_same_then_else)]
    if cfg!(debug_assertions) {
        debug!("Debugging enabled");
    } else {
        debug!("Debugging disabled");
    }

    #[cfg(debug_assertions)]
    debug!("Debugging enabled");
    #[cfg(not(debug_assertions))]
    debug!("Debugging disabled");
    example();

    let start = time::get_time();
    let epoch = get_epoch_ms();
    println!("epoch:{}", epoch.clone());
    let system_time = SystemTime::now();
    println!("system_time:{:?}", system_time.clone());

    let datetime: DateTime<Utc> = system_time.into();
    println!("{}", datetime.format("%d/%m/%Y %T/%s"));
    println!("{}", datetime.format("%d/%m/%Y %T/%f"));
    println!("{}", datetime.format("%d/%m/%Y %T"));

    //let cwd = get_current_working_dir();
    let cwd = get_pwd();
    #[cfg(debug_assertions)]
    println!("Debugging enabled");
    println!("{:#?}", cwd);
    let state = repo::state();
    println!("{:#?}", state);
    //
    let repo_root = std::env::args().nth(1).unwrap_or(".".to_string());
    println!("repo_root={:?}", repo_root.as_str());
    let repo = Repository::discover(repo_root.as_str()).expect("Couldn't open repository");
    println!("{} state={:?}", repo.path().display(), repo.state());
    println!("state={:?}", repo.state());

    let count = thread::available_parallelism()?.get();
    assert!(count >= 1_usize);

    let now = SystemTime::now();

    let pwd = env::current_dir()?;
    println!("pwd={}", pwd.clone().display());
    let mut hasher = Sha256::new();
    hasher.update(&format!("{}", pwd.clone().display()));
    //sha256sum <(echo gnostr-legit)
    let pwd_hash: String = format!("{:x}", hasher.finalize());
    println!("pwd_hash={:?}", pwd_hash);

    let mut opts = gitminer::Options {
        threads: count.try_into().unwrap(),
        target: "00000".to_string(), //default 00000
        //gnostr:##:nonce
        //part of the gnostr protocol
        //src/worker.rs adds the nonce
        pwd_hash: pwd_hash.clone(),
        message: cwd.unwrap(),
        //message: message,
        //message: count.to_string(),
        //repo:    ".".to_string(),
        repo: repo.path().display().to_string(),
        timestamp: time::now(),
    };

    parse_args_or_exit(&mut opts);

    let mut miner = match Gitminer::new(opts) {
        Ok(m) => m,
        Err(e) => {
            panic!("Failed to start git miner: {}", e);
        }
    };

    let hash = match miner.mine() {
        Ok(s) => s,
        Err(e) => {
            panic!("Failed to generate commit: {}", e);
        }
    };

    let mut hasher = Sha256::new();
    hasher.update(&hash);
    // `update` can be called repeatedly and is generic over `AsRef<[u8]>`
    //hasher.update("String data");
    // Note that calling `finalize()` consumes hasher
    //let gnostr_sec = hasher.finalize();
    let gnostr_sec: String = format!("{:X}", hasher.finalize());
    //println!("Binary hash: {:?}", hash);
    //println!("hash before: {:?}", hash);
    //println!("hash after pad: {:?}", hash);
    //println!("&hash before: {:?}", &hash);
    //println!("&hash after pad: {:?}", &hash);
    //println!("gnostr_sec before pad: {:?}", gnostr_sec);
    //println!("gnostr_sec after pad: {:?}", gnostr_sec.pad(64, '0', Alignment::Right, true));
    //println!("&gnostr_sec before pad: {:?}", &gnostr_sec);
    //println!("&gnostr_sec after pad: {:?}", &gnostr_sec.pad(64, '0', Alignment::Right, true));

    //let s = "12345".pad(64, '0', Alignment::Right, true);
    //println!("s: {:?}", s);
    // echo "000000b64a065760e5441bf47f0571cb690b28fc" | openssl dgst -sha256 | sed 's/SHA2-256(stdin)= //g'
    //
    //
    //shell test
    let touch = Command::new("sh")
        .args(["-c", "touch ", &hash])
        .output()
        .expect("failed to execute process");
    let touch_event = String::from_utf8(touch.stdout)
        .map_err(|non_utf8| String::from_utf8_lossy(non_utf8.as_bytes()).into_owned())
        .unwrap();
    let cat = Command::new("sh")
        .args(["-c", "touch ", &hash])
        .output()
        .expect("failed to execute process");
    let cat_event = String::from_utf8(cat.stdout)
        .map_err(|non_utf8| String::from_utf8_lossy(non_utf8.as_bytes()).into_owned())
        .unwrap();
    //shell test
    //git rev-parse --verify HEAD
    #[allow(clippy::if_same_then_else)]
    let event = if cfg!(target_os = "windows") {
        Command::new("cmd")
                .args(["/C", "gnostr --sec $(gnostr-sha256 $(gnostr-weeble || echo)) -t gnostr --tag weeble $(gnostr-weeble || echo weeble) --tag wobble $(gnostr-wobble || echo wobble) --tag blockheight $(gnostr-blockheight || echo blockheight) --content \"$(gnostr-git diff HEAD~1 || gnostr-git diff)\" "])
                .output()
                .expect("failed to execute process")
    } else if cfg!(target_os = "macos") {
        Command::new("sh")
                .args(["-c", "gnostr --sec $(gnostr-sha256 $(gnostr-weeble || echo)) -t gnostr --tag weeble $(gnostr-weeble || echo weeble) --tag wobble $(gnostr-wobble || echo wobble) --tag blockheight $(gnostr-blockheight || echo blockheight) --content \"$(gnostr-git show HEAD)\" "])
                .output()
                .expect("failed to execute process")
    } else if cfg!(target_os = "linux") {
        Command::new("sh")
                .args(["-c", "gnostr --sec $(gnostr-sha256 $(gnostr-weeble || echo)) -t gnostr --tag weeble $(gnostr-weeble || echo weeble) --tag wobble $(gnostr-wobble || echo wobble) --tag blockheight $(gnostr-blockheight || echo blockheight) --content \"$(gnostr-git diff HEAD~1 || gnostr-git diff)\" "])
                .output()
                .expect("failed to execute process")
    } else {
        Command::new("sh")
                .args(["-c", "gnostr --sec $(gnostr-sha256 $(gnostr-weeble || echo)) -t gnostr --tag weeble $(gnostr-weeble || echo weeble) --tag wobble $(gnostr-wobble || echo wobble) --tag blockheight $(gnostr-blockheight || echo blockheight) --content \"$(gnostr-git diff HEAD~1 || gnostr-git diff)\" "])
                .output()
                .expect("failed to execute process")
    };

    let gnostr_event = String::from_utf8(event.stdout)
        .map_err(|non_utf8| String::from_utf8_lossy(non_utf8.as_bytes()).into_owned())
        .unwrap();

    //assert...
    //echo gnostr|openssl dgst -sha256 | sed 's/SHA2-256(stdin)= //g'

    //gnostr-legit must only return a sha256 generated by the
    //recent commit hash
    //to enable nested commands
    //REF:
    //gnostr --hash $(gnostr legit . -p 00000 -m "git rev-parse --verify HEAD")
    //gnostr --sec $(gnostr --hash $(gnostr legit . -p 00000 -m "git rev-parse --verify HEAD"))
    //Example:
    //gnostr --sec $(gnostr --hash $(gnostr legit . -p 00000 -m "#gnostr will exist!")) --envelope --content "$(gnostr-git log -n 1)" | gnostr-cat -u wss://relay.damus.io
    //
    //
    //
    let duration = time::get_time() - start;
    //println!("Success! Generated commit {} in {} seconds", hash, duration.num_seconds());
    //
    //
    let relay_url = "wss://nos.lol";
    let event: Event = serde_json::from_str(&gnostr_event).unwrap();
    post_event(relay_url, event);

    println!("{}", gnostr_event);
    Ok(())
}

fn parse_args_or_exit(opts: &mut gitminer::Options) {
    let mut ap = ArgumentParser::new();
    ap.set_description("Generate git commit sha with a custom prefix");
    ap.stop_on_first_argument(false);

    //ap.refer(&mut opts.repo)
    //    //.add_argument("repository-path", Store, "Path to your git repository (required)");
    //    .add_argument("repository-path", Store, "Path to your git repository");
    //    //.required();
    ap.refer(&mut opts.repo)
        .add_argument("repository-path", Store, "Path to your git repository");

    ap.refer(&mut opts.target).add_option(
        &["-p", "--prefix"],
        Store,
        "Desired commit prefix (required)",
    );
    //.required();

    ap.refer(&mut opts.threads).add_option(
        &["-t", "--threads"],
        Store,
        "Number of worker threads to use (default 8)",
    );

    ap.refer(&mut opts.message).add_option(
        &["-m", "--message"],
        Store,
        "Commit message to use (required)",
    );
    //.required();

    //ap.refer(&mut opts.timestamp)
    //    .add_option(&["--timestamp"], Store, "Commit timestamp to use (default now)");

    ap.parse_args_or_exit();
}
