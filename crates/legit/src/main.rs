#![allow(unused)]
#![allow(dead_code)]
extern crate chrono;
use std::process::Command;
use chrono::offset::Utc;
use chrono::DateTime;
//use std::time::SystemTime;
use std::io::{Result};
use std::env;
use std::time::{Duration, SystemTime};
use std::thread::sleep;
use std::convert::TryInto;
use std::any::type_name;
//use std::mem::size_of;
use std::{io, thread};
use argparse::{ArgumentParser,Store};
use gitminer::Gitminer;
use git2::*;
use sha2::{Sha256, Digest};
use pad::{PadStr, Alignment};

mod worker;
mod gitminer;
mod repo;

fn type_of<T>(_: T) -> &'static str {
    type_name::<T>()
}

//fn convert_to_u32(v: usize) -> Option<i8> {
//    if v > (std::i8::MAX as i32).try_into().unwrap() {
//        None
//    } else {
//        Some(v as i8)
//    }
//}

//fn get_current_working_dir() -> std::io::Result<PathBuf> {
//    env::current_dir()
//}

fn main() -> io::Result<()> {

    let start = time::get_time();
    let system_time = SystemTime::now();

    let datetime: DateTime<Utc> = system_time.into();
    //println!("{}", datetime.format("%d/%m/%Y %T/%s"));
    //println!("{}", datetime.format("%d/%m/%Y %T"));

    let state = repo::state();
    //println!("{:#?}", state);
    //
    let repo_root = std::env::args().nth(1).unwrap_or(".".to_string());
    //println!("repo_root={:?}", repo_root.as_str());
    let repo = Repository::open(repo_root.as_str()).expect("Couldn't open repository");
    //println!("{} state={:?}", repo.path().display(), repo.state());
    //println!("state={:?}", repo.state());
    if repo.state() != RepositoryState::Clean {

    //println!("clean {:?}", repo.state());

    let repo_state =
        if cfg!(target_os = "windows") {
        Command::new("cmd")
                .args(["/C", "git status"])
                .output()
                .expect("failed to execute process")
        } else
        if cfg!(target_os = "macos"){
        Command::new("sh")
                .arg("-c")
                .arg("gnostr-git diff")
                .output()
                .expect("failed to execute process")
        } else
        if cfg!(target_os = "linux"){
        Command::new("sh")
                .arg("-c")
                .arg("gnostr-git diff")
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
    //println!("state={:?}", state);
    }

    let count = thread::available_parallelism()?.get();
    assert!(count >= 1_usize);
    //println!("{}={}", type_of(count), (count as i32));
    //println!("{}={}", type_of(count), (count as i64));
    //let mut hasher = Sha256::new();
    //let data = b"Hello world!";
    //hasher.update(data);
    //// `update` can be called repeatedly and is generic over `AsRef<[u8]>`
    //hasher.update("String data");
    //// Note that calling `finalize()` consumes hasher
    //let hash = hasher.finalize();
    ////println!("Binary hash: {:?}", hash);
    //println!("hash: {:?}", hash);
    //println!("sha256 before write: {:x}", hash);
    //println!("sha256 before write: {:X}", hash);


   let now = SystemTime::now();

   //// we sleep for 2 seconds
   //sleep(Duration::new(2, 0));
   // match now.elapsed() {
   //    Ok(elapsed) => {
   //        // it prints '2'
   //        println!("{}", elapsed.as_secs());
   //    }
   //    Err(e) => {
   //        // an error occurred!
   //        println!("Error: {e:?}");
   //    }
   //}


    let output =
        if cfg!(target_os = "windows") {
        Command::new("cmd")
                .args(["/C", "git status"])
                .output()
                .expect("failed to execute process")
        } else
        if cfg!(target_os = "macos"){
        Command::new("sh")
                .arg("-c")
                .arg("git diff")
                .output()
                .expect("failed to execute process")
        } else
        if cfg!(target_os = "linux"){
        Command::new("sh")
                .arg("-c")
                .arg("git diff")
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

    let path = env::current_dir()?;
        //println!("The current directory is {}", path.display());
        //Ok(());
    let mut opts = gitminer::Options{
        threads: count.try_into().unwrap(),
        target:  "00000".to_string(),//default 00000
        //gnostr:##:nonce
        //part of the gnostr protocol
        //src/worker.rs adds the nonce
        //message: "gnostr".to_string(),
        message: message,
        //message: count.to_string(),
        //repo:    ".".to_string(),
        repo:    path.as_path().display().to_string(),
        timestamp: time::now(),
            //.duration_since(SystemTime::UNIX_EPOCH)
    };

    parse_args_or_exit(&mut opts);

    let mut miner = match Gitminer::new(opts) {
        Ok(m)  => m,
        Err(e) => { panic!("Failed to start git miner: {}", e); }
    };

    let hash = match miner.mine() {
        Ok(s)  => s,
        Err(e) => { panic!("Failed to generate commit: {}", e); }
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
    let touch =
        Command::new("sh")
                .args(["-c", "touch ", &hash])
                .output()
                .expect("failed to execute process");
    let touch_event = String::from_utf8(touch.stdout)
    .map_err(|non_utf8| String::from_utf8_lossy(non_utf8.as_bytes()).into_owned())
    .unwrap();
    let cat =
        Command::new("sh")
                .args(["-c", "touch ", &hash])
                .output()
                .expect("failed to execute process");
    let cat_event = String::from_utf8(cat.stdout)
    .map_err(|non_utf8| String::from_utf8_lossy(non_utf8.as_bytes()).into_owned())
    .unwrap();
//shell test
    //git rev-parse --verify HEAD
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
    println!("{}", gnostr_test);
    Ok(())

}

fn parse_args_or_exit(opts: &mut gitminer::Options) {
    let mut ap = ArgumentParser::new();
    ap.set_description("Generate git commit sha with a custom prefix");

    //ap.stop_on_first_argument(false);

    //ap.refer(&mut opts.repo)
    //    //.add_argument("repository-path", Store, "Path to your git repository (required)");
    //    .add_argument("repository-path", Store, "Path to your git repository");
    //    //.required();
    //ap.refer(&mut opts.repo).add_argument("repository-path", Store, "Path to your git repository");

    ap.refer(&mut opts.target)
        .add_option(&["-p", "--prefix"], Store, "Desired commit prefix (required)");
        //.required();

    ap.refer(&mut opts.threads)
        .add_option(&["-t", "--threads"], Store, "Number of worker threads to use (default 8)");

    ap.refer(&mut opts.message)
        .add_option(&["-m", "--message"], Store, "Commit message to use (required)");
        //.required();

    //ap.refer(&mut opts.timestamp)
    //    .add_option(&["--timestamp"], Store, "Commit timestamp to use (default now)");
    
    ap.refer(&mut opts.repo).add_argument("repository-path", Store, "Path to your git repository");

    ap.parse_args_or_exit();
}
