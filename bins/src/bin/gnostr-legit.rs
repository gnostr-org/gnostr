use std::{
    convert::TryInto,
    env,
    io::{self},
    process::Command,
    thread,
};

/// #![allow(unused)]
/// #![allow(dead_code)]
///
/// #![allow(unused)]
/// #![allow(dead_code)]
use clap::Parser;
use gnostr::legit::command;
use gnostr_legit::gitminer;
use time::OffsetDateTime;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The repository path (defaults to current directory)
    #[arg(default_value = ".")]
    repo: String,

    /// Prefix for the target hash
    #[arg(long, default_value = "000")]
    prefix: String,

    /// NIP-13 event-ID difficulty in leading zero bits, independent of --prefix
    #[arg(
        long,
        default_value_t = 0,
        value_name = "BITS",
        conflicts_with = "event_prefix"
    )]
    event_difficulty: u8,

    /// Mine a hexadecimal event-ID prefix instead of NIP-13 leading zero bits
    #[arg(long, value_name = "HEX", value_parser = gnostr_asyncgit::types::EventMining::prefix)]
    event_prefix: Option<gnostr_asyncgit::types::EventMining>,

    /// Commit message
    #[arg(short, long)]
    message: Option<Vec<String>>,
}

#[tokio::main]
async fn main() -> io::Result<()> {
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
        [String::from_utf8(output.stdout)
            .map_err(|non_utf8| String::from_utf8_lossy(non_utf8.as_bytes()).into_owned())
            .unwrap()]
        .to_vec()
    };

    let count = thread::available_parallelism()?.get();

    let opts = gitminer::Options {
        threads: count.try_into().unwrap(),
        target: args.prefix,
        message,
        repo: path.as_path().display().to_string(),
        timestamp: OffsetDateTime::now_utc(),
        kind: None,
    };

    command::run_legit_command_with_event_mining(
        opts,
        args.event_prefix
            .unwrap_or(gnostr_asyncgit::types::EventMining::Difficulty(
                args.event_difficulty,
            )),
    )
    .await
}
