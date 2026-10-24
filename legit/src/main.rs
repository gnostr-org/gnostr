#![allow(unused)]
#![allow(dead_code)]
#[macro_use]
extern crate log;
extern crate env_logger;
use std::convert::TryInto;
use std::fmt::Debug;
use std::time::Instant;
//use std::mem::size_of;
use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};
use crypto::sha2::Sha256;
use gitminer::Gitminer;
use gnostr_asyncgit::git2::*;
use gnostr_asyncgit::types::Keys;
use std::{io, thread};

mod compose;
mod gitminer;
mod repo;
mod worker;

#[derive(Parser)]
#[command(version, about = "Mine git commits or compose NIP-34 events")]
struct Args {
    /// Path to your git repository
    #[arg(long, global = true, default_value = ".")]
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

    /// NIP-34 signing key: 64-character hex (SHA-256) or nsec1... private key
    #[arg(long, global = true, value_name = "KEY")]
    nsec: Option<String>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Compose signed NIP-34 JSON without mining; publish only with --publish
    Compose(compose::Compose),
}

fn parse_args_from<I, T>(iter: I) -> Result<Args, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let mut command = Args::command();
    let matches = command.try_get_matches_from_mut(iter)?;
    let args = Args::from_arg_matches(&matches)?;
    if args.command.is_some()
        && ["prefix", "threads", "message"]
            .iter()
            .any(|name| matches.value_source(name) == Some(clap::parser::ValueSource::CommandLine))
    {
        return Err(command.error(
            clap::error::ErrorKind::ArgumentConflict,
            "Mining options --prefix, --threads, and --message cannot be combined with compose",
        ));
    }
    Ok(args)
}

fn parse_signing_keys(nsec: Option<String>) -> Result<Option<Keys>, &'static str> {
    nsec.map(|value| {
        Keys::parse(value)
            .ok_or("--nsec must be a valid 64-character hex (SHA-256) or nsec1... private key")
    })
    .transpose()
}

//fn convert_to_u32(v: usize) -> Option<i8> {
//    if v > (std::i8::MAX as i32).try_into().unwrap() {
//        None
//    } else {
//        Some(v as i8)
//    }
//}

fn main() -> io::Result<()> {
    let args = parse_args_from(std::env::args_os()).unwrap_or_else(|err| err.exit());
    let signing_keys = parse_signing_keys(args.nsec).unwrap_or_else(|message| {
        Args::command()
            .error(clap::error::ErrorKind::ValueValidation, message)
            .exit()
    });
    env_logger::init();
    info!("Starting gnostr-legit");
    if let Some(Command::Compose(command)) = args.command {
        return command
            .run(signing_keys, &args.repo)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, format!("{:#}", err)));
    }

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
            eprintln!("Failed to start git miner: {}", e);
            std::process::exit(1);
        }
    };
    if let Some(keys) = signing_keys {
        miner = miner.with_signing_keys(keys);
    }

    let hash = match miner.mine() {
        Ok(s) => s,
        Err(e) => {
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
    use super::{parse_args_from, parse_signing_keys, Args};
    use clap::{error::ErrorKind, Parser};
    use gnostr_asyncgit::types::Keys;

    #[test]
    fn cli_defaults() {
        let args = Args::try_parse_from(["gnostr-legit"]).unwrap();
        assert_eq!(args.repo, ".");
        assert_eq!(args.prefix, "000");
        assert_eq!(args.threads, None);
        assert_eq!(args.message, ["default commit message"]);
        assert!(parse_signing_keys(args.nsec).unwrap().is_none());
        assert!(args.command.is_none());
    }

    #[test]
    fn compose_accepts_global_context_without_mining_options() {
        let args = Args::try_parse_from([
            "gnostr-legit",
            "compose",
            "grasp-list",
            "--repo",
            "/tmp/repo",
            "--nsec",
            &"1".repeat(64),
        ])
        .unwrap();
        assert!(args.command.is_some());
        assert_eq!(args.repo, "/tmp/repo");
        assert!(parse_signing_keys(args.nsec).unwrap().is_some());
        let args = parse_args_from([
            "gnostr-legit",
            "--repo",
            "/tmp/repo",
            "--nsec",
            &"1".repeat(64),
            "compose",
            "grasp-list",
        ])
        .unwrap();
        assert!(args.command.is_some());
        assert!(
            parse_args_from(["gnostr-legit", "--prefix", "000", "compose", "grasp-list"]).is_err()
        );
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
                    .err()
                    .unwrap()
                    .kind(),
                kind
            );
        }
    }

    #[test]
    fn cli_accepts_hex_and_nsec_signing_keys() {
        let expected = Keys::generate();
        let mut secret = expected.secret_key().unwrap();
        for value in [secret.as_hex_string(), secret.as_bech32_string()] {
            let args = Args::try_parse_from(["gnostr-legit", "--nsec", &value]).unwrap();
            let actual = parse_signing_keys(args.nsec).unwrap().unwrap();
            assert_eq!(actual.public_key(), expected.public_key());
        }
    }

    #[test]
    fn cli_rejects_invalid_signing_keys_without_echoing_them() {
        for value in [
            "invalid-private-key-value".to_string(),
            "0".repeat(64),
            "f".repeat(64),
            "1".repeat(63),
            "nsec1invalid".to_string(),
        ] {
            let args = Args::try_parse_from(["gnostr-legit", "--nsec", &value]).unwrap();
            let message = parse_signing_keys(args.nsec).unwrap_err();
            assert!(message.contains("--nsec"));
            assert!(!message.contains(&value));
        }
    }
}
