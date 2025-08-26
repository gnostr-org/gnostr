#![allow(unused)]
#![allow(dead_code)]
extern crate chrono;
extern crate time;
use chrono::offset::Utc;
use chrono::DateTime;
use gnostr_asyncgit::sync::commit::{padded_commit_id, serialize_commit};

use gnostr::global_rt::global_rt;
use log::debug;
use log::info;
//
use nostr_sdk_0_37_0::prelude::*;
//

use std::process::Command;
//use std::time::SystemTime;
use std::any::type_name;
use std::convert::TryInto;
use std::env;
use std::error::Error;
use std::io::Result;
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use time::{get_time, now};
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

#[tokio::main]
async fn main() -> Result<()> {



	let mut repo: Option<String> = None;
	let mut prefix: Option<String> = None;
	let mut message: Option<String> = None;

    let args: Vec<String> = env::args().collect();

    for arg in env::args() {
        println!("  {}", arg);
        if arg == "--repo" { repo = Some(arg.clone()); }
        if arg == "--prefix" { prefix = Some(arg.clone()); }
        if arg == "--prefix" { message = Some(arg.clone()); }
	}
if let Some(arg) = args.get(1) {
        if arg == "--repo" {
            println!("The second argument is '--repo'.");
            // Do something here...
        } else {
            println!("The second argument is '{}'.", arg);
        }
    } else {
        println!("The second argument was not provided.");
    }



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

    let start = get_time();
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
    hasher.update(format!("{}", pwd.clone().display()));
    //sha256sum <(echo gnostr-legit)
    let pwd_hash: String = format!("{:x}", hasher.finalize());
    println!("pwd_hash={:?}", pwd_hash);




    let mut opts = gitminer::Options {
        threads: count.try_into().unwrap(),
        target: prefix.unwrap_or("00000".to_string()), // Use prefix from CLI
        //gnostr:##:nonce
        //part of the gnostr protocol
        //src/worker.rs adds the nonce
        pwd_hash: pwd_hash.clone(),
        message: message.unwrap_or(cwd.unwrap()), // Use message from CLI
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

    //initialize git repo
    let repo = Repository::discover(".").expect("Failed to discover repository");

    //gather some repo info
    //find HEAD
    let head = repo.head().expect("Failed to get HEAD");
    let obj = head
        .resolve()
        .expect("Failed to resolve HEAD")
        .peel(ObjectType::Commit)
        .expect("Failed to peel to commit");

    //read top commit
    let commit = obj.peel_to_commit().expect("Failed to peel to commit");
    let commit_id = commit.id().to_string();

    let serialized_commit = serialize_commit(&commit).expect("gnostr-async:error!");
    println!("Serialized commit:\n{}", serialized_commit.clone());

    //some info wrangling
    println!("commit_id:\n{}", commit_id);
    let padded_commitid = padded_commit_id(format!("{:0>64}", commit_id.clone()));
    println!("padded_commitid:\n{}", padded_commitid.clone());
    global_rt().spawn(async move {
        //// commit based keys
        //let keys = generate_nostr_keys_from_commit_hash(&commit_id)?;
        //info!("keys.secret_key():\n{:?}", keys.secret_key());
        //info!("keys.public_key():\n{}", keys.public_key());

        //parse keys from sha256 hash
        let padded_keys = Keys::parse(padded_commitid).unwrap();
        //create nostr client with commit based keys
        //let client = Client::new(keys);
        let client = Client::new(padded_keys.clone());
        client.add_relay("wss://relay.damus.io").await.expect("Failed to add relay");
        client.add_relay("wss://e.nos.lol").await.expect("Failed to add relay");
        client.connect().await;

        //build git gnostr event
        let builder = EventBuilder::text_note(serialized_commit.clone());

        //send git gnostr event
        let output = client.send_event_builder(builder).await.expect("Failed to send event");

        //some reporting
        info!("Event ID: {}", output.id());
        info!("Event ID BECH32: {}", output.id().to_bech32().expect("Failed to convert event ID to bech32"));
        info!("Sent to: {:?}", output.success);
        info!("Not sent to: {:?}", output.failed);
    });

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
