#![allow(unused)]
#![allow(dead_code)]
#[macro_use]
extern crate log;
extern crate env_logger;
use std::convert::TryInto;
use std::fmt::Debug;
use std::time::Instant;
//use std::mem::size_of;
use clap::Parser;
use crypto::sha2::Sha256;
use gitminer::Gitminer;
use gnostr_asyncgit::git2::*;
use std::{io, thread};

mod gitminer;
mod repo;
mod worker;

#[derive(Debug, Parser)]
#[command(version, about = "Generate git commit sha with a custom prefix")]
struct Args {
    /// Path to your git repository
    #[arg(long, default_value = ".")]
    repo: String,

    /// Desired commit prefix
    #[arg(short = 'p', long, default_value = "000")]
    prefix: String,

    /// Number of worker threads (defaults to available parallelism)
    #[arg(short = 't', long, value_parser = clap::value_parser!(u32).range(1..))]
    threads: Option<u32>,

    /// Commit message; repeat for additional lines
    #[arg(short, long, default_value = "default commit message")]
    message: Vec<String>,
}

//fn convert_to_u32(v: usize) -> Option<i8> {
//    if v > (std::i8::MAX as i32).try_into().unwrap() {
//        None
//    } else {
//        Some(v as i8)
//    }
//}

fn main() -> io::Result<()> {
    let args = Args::parse();
    env_logger::init();
    info!("Starting gnostr-legit");

    let start = Instant::now();
    debug!("Mining session started at {}", chrono::Local::now());

    let count = thread::available_parallelism()?.get();
    assert!(count >= 1_usize);
    debug!(
        "Available logical processors: {}; requested worker threads: {:?}",
        count, args.threads
    );
    let _sha256 = Sha256::new();
    //sha256.input_str(count);
    //let ip_address_hash: String = format!("{:X}", sha256.finalize());

    //TODO kind 1617 patch default
    let opts = gitminer::Options {
        threads: args.threads.unwrap_or(count.try_into().unwrap()),
        target: args.prefix,
        message: args.message,
        repo: args.repo,
        timestamp: time_0_3::OffsetDateTime::now_utc(),
        kind: None,
    };

    debug!(
        "Mining configuration: repo={}, prefix={}, threads={}, message parts={}, timestamp={}",
        opts.repo,
        opts.target,
        opts.threads,
        opts.message.len(),
        opts.timestamp
    );
    let state = repo::state(&opts.repo);
    debug!("Repository state before mining: {:?}", state);

    debug!("Initializing git miner");
    let mut miner = match Gitminer::new(opts) {
        Ok(m) => m,
        Err(e) => {
            error!("Failed to start git miner: {}", e);
            eprintln!("Failed to start git miner: {}", e);
            std::process::exit(1);
        }
    };

    let hash = match miner.mine() {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to generate commit: {}", e);
            eprintln!("Failed to generate commit: {}", e);
            std::process::exit(1);
        }
    };

    let duration = start.elapsed();
    println!(
        "Success! Generated commit {} in {} seconds",
        hash,
        duration.as_secs()
    );
    debug!(
        "Mining session completed: hash={}, elapsed={:?}, elapsed_ms={}",
        hash,
        duration,
        duration.as_millis()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::Args;
    use clap::{error::ErrorKind, Parser};

    #[test]
    fn cli_defaults() {
        let args = Args::try_parse_from(["gnostr-legit"]).unwrap();
        assert_eq!(args.repo, ".");
        assert_eq!(args.prefix, "000");
        assert_eq!(args.threads, None);
        assert_eq!(args.message, ["default commit message"]);
    }

    #[test]
    fn cli_flags_and_repeated_messages() {
        let args = Args::try_parse_from([
            "gnostr-legit",
            "--repo",
            "/tmp/my repo",
            "-p",
            "0000",
            "-t",
            "2",
            "-m",
            "subject",
            "--message",
            "body",
        ])
        .unwrap();
        assert_eq!(args.repo, "/tmp/my repo");
        assert_eq!(args.prefix, "0000");
        assert_eq!(args.threads, Some(2));
        assert_eq!(args.message, ["subject", "body"]);
    }

    #[test]
    fn cli_rejects_invalid_worker_counts() {
        for threads in ["0", "-1", "abc", "4294967296"] {
            assert!(Args::try_parse_from(["gnostr-legit", "--threads", threads]).is_err());
        }
    }

    #[test]
    fn cli_help_and_version() {
        for (flag, kind) in [
            ("--help", ErrorKind::DisplayHelp),
            ("--version", ErrorKind::DisplayVersion),
        ] {
            assert_eq!(
                Args::try_parse_from(["gnostr-legit", flag])
                    .unwrap_err()
                    .kind(),
                kind
            );
        }
    }
}
