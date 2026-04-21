pub mod processor;
pub mod pubkeys;
pub mod relay_manager;
pub mod relays;
pub mod query;
pub mod stats;
pub use query::{build_gnostr_query, send, Config, ConfigBuilder};
pub use query::cli;

pub fn init_tracing() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env()
        .add_directive("hyper::client::trace=trace".parse()?)
        .add_directive("hyper::client::connect=trace".parse()?)
        .add_directive("hyper::client::connect::http=off".parse()?)
        .add_directive("hyper::proto=off".parse()?)
        .add_directive("nostr_sdk::relay=off".parse()?)
        .add_directive("nostr_relay_pool=off".parse()?)
        .add_directive("nostr_relay_pool::relay::inner=off".parse()?)
        //.add_directive("hyper=off".parse()?)

        /**/)/**/
        .init();
    Ok(())
}

use clap::{Parser, Subcommand};
use futures::{stream, StreamExt};
use git2::Error;
use git2::{Commit, DiffOptions, Repository, Signature, Time};
use reqwest::header::ACCEPT;
use std::collections::{HashMap, HashSet};
use std::fs as sync_fs;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use std::str;
use std::process::{Command, Stdio};
#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

use serde::{Deserialize, Serialize};

use ::time::at;
use ::time::Timespec;
use nostr_sdk::prelude::*;
use ::url::Url;

use crate::processor::Processor;
use crate::processor::APP_SECRET_KEY;
use crate::relay_manager::RelayManager;

#[allow(unused_imports)]
use crate::processor::LOCALHOST_8080;
use crate::processor::BOOTSTRAP_RELAYS;

use axum::{
    extract::{Path as AxumPath, Query},
    routing::get,
    response::{IntoResponse, Response},
    Router,
    body::Body, // Added for explicit body type
    http::{StatusCode, header::CONTENT_TYPE}, // Changed to axum::http
};
use std::net::SocketAddr;
use std::path::PathBuf;
use tokio::fs; // For async file operations
#[allow(unused_imports)] // Suppress false positive for tokio::task::spawn
use tokio::task::spawn; // Added for spawning async tasks
use tower_http::trace::{self, TraceLayer}; // For logging requests

const CONCURRENT_REQUESTS: usize = 16;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
    //nsec: Option<String>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Runs the sniper mode to find relays supporting a specific NIP
    Sniper {
        /// The NIP number to search for (e.g., 1)
        nip: i32,
        /// Optional: Path to a shitlist file to exclude relays
        #[clap(long, short)]
        shitlist: Option<String>,
    },
    /// Runs the watch mode to monitor relays and print their metadata
    Watch {
        /// Optional: Path to a shitlist file to exclude relays
        #[clap(long, short)]
        shitlist: Option<String>,
    },
    /// Lists relays that are likely to support NIP-34 (Git collaboration)
    Nip34 {
        /// Optional: Path to a shitlist file to exclude relays
        #[clap(long, short)]
        shitlist: Option<String>,
    },
    /// Runs the main gnostr-crawler logic
    Crawl(CliArgs),
    /// Starts a web server to serve relay information
    Serve {
        /// The port to listen on for the API server
        #[clap(long, short, default_value = "3000")]
        port: u16,
        /// Run the API server in the background.
        #[clap(long, default_value_t = false)]
        detach: bool,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Relay {
    pub contact: Option<String>,
    pub description: Option<String>,
    pub name: Option<String>,
    pub software: Option<String>,
    pub supported_nips: Option<Vec<i32>>,
    pub supported_nip_extensions: Option<Vec<String>>,
    pub version: Option<String>,
}

pub fn preprocess_line(line: &str) -> String {
    let mut trimmed_line = line.trim().to_string();
    if let Some(stripped) = trimmed_line.strip_prefix("- ") {
        trimmed_line = stripped.trim().to_string();
    } else if let Some(stripped) = trimmed_line.strip_prefix('-') {
        trimmed_line = stripped.trim().to_string();
    }
    // Truncate at the first comma, if any
    if let Some(comma_idx) = trimmed_line.find(',') {
        trimmed_line.truncate(comma_idx);
        trimmed_line = trimmed_line.trim().to_string(); // Re-trim after truncation
    }
    trimmed_line
}

pub fn load_file(filename: impl AsRef<Path>) -> io::Result<Vec<String>> {
    let base_dir = crate::relays::get_config_dir_path();
    let file_path = base_dir.join(filename.as_ref().file_name().unwrap_or(filename.as_ref().as_os_str()));

    if let Some(parent) = file_path.parent() {
        sync_fs::create_dir_all(parent)?;
    }

    debug!("Loading file: {}", file_path.display());

    let file_content = sync_fs::read_to_string(&file_path)?;

    // Preprocess each line to truncate after a comma and trim whitespace
    let preprocessed_lines: Vec<String> = file_content.lines()
        .map(|line| preprocess_line(line))
        .filter(|line| !line.is_empty())
        .collect();

    let preprocessed_content_for_yaml = preprocessed_lines.join("\n");

    let relays: Vec<String> = match serde_yaml::from_str::<Vec<String>>(&preprocessed_content_for_yaml) {
        Ok(yaml_relays) => yaml_relays,
        Err(e) => {
            // Fallback to line-by-line collection of already preprocessed lines if it's not valid YAML
            warn!("Failed to parse {} as YAML: {}. Falling back to preprocessed lines.", file_path.display(), e);
            preprocessed_lines
        }
    };

    let filtered_relays: Vec<String> = relays.into_iter()
        .filter_map(|line| {
            // Lines are already preprocessed for truncation and trimming.
            // Now, refine filtering to differentiate between actual non-websocket URLs and non-URL lines.
            if line.is_empty() {
                return None;
            }

            let mut final_line = line.clone();

            // Attempt to prepend wss:// if it looks like a hostname without a scheme
            if !final_line.contains("://") {
                let potential_url = format!("wss://{}", final_line);
                match Url::parse(&potential_url) {
                    Ok(url) => {
                        debug!("Prepended 'wss://' to form valid URL: {}", url);
                        final_line = url.to_string();
                    },
                    Err(_) => {
                        // If prepending wss:// doesn't form a valid URL, keep the original line
                        // and let the next checks handle it as a non-URL line.
                        debug!("Attempted to prepend 'wss://' but it's still not a valid URL: {}", potential_url);
                    }
                }
            }

            if final_line.starts_with("wss://") || final_line.starts_with("ws://") {
                match Url::parse(&final_line) {
                    Ok(url) => Some(url.to_string()),
                    Err(_) => {
                        warn!("Skipping invalid WEBSOCKET URL in {}: {}", filename.as_ref().display(), final_line);
                        None
                    }
                }
            } else if final_line.contains("://") { // It's a URL, but not a websocket URL
                warn!("Skipping non-websocket URL scheme in {}: {}", filename.as_ref().display(), final_line);
                None
            } else { // It's not a URL at all (e.g., "Relay URL")
                debug!("Silently skipping non-URL line in {}: {}", filename.as_ref().display(), final_line);
                None
            }
        })
        .collect();

    Ok(filtered_relays)
}

//pub fn load_file(filename: impl AsRef<Path>) -> io::Result<Vec<String>> {
//    BufReader::new(sync_fs::File::open(filename)?).lines().collect()
//}

pub fn load_shitlist(filename: impl AsRef<Path>) -> io::Result<HashSet<String>> {
    BufReader::new(sync_fs::File::open(filename)?).lines().collect()
}

fn load_relays_or_bootstrap() -> Vec<String> {
    match load_file("relays.yaml") {
        Ok(relays) => relays,
        Err(e) => {
            warn!(
                "Failed to load relays.yaml ({}); falling back to bootstrap relays",
                e
            );
            BOOTSTRAP_RELAYS.iter().cloned().collect()
        }
    }
}

#[allow(clippy::manual_strip)]
#[derive(Parser, Debug, Clone)]
pub struct CliArgs {
    //#[clap(name = "topo-order", long)]
    ///// sort commits in topological order
    //flag_topo_order: bool,
    //#[clap(name = "date-order", long)]
    ///// sort commits in date order
    //flag_date_order: bool,
    //#[clap(name = "reverse", long)]
    ///// sort commits in reverse
    //flag_reverse: bool,
    //#[clap(name = "author", long)]
    ///// author to sort by
    //flag_author: Option<String>,
    //#[clap(name = "committer", long)]
    ///// committer to sort by
    //flag_committer: Option<String>,
    //#[clap(name = "pat", long = "grep")]
    ///// pattern to filter commit messages by
    //flag_grep: Option<String>,
    #[clap(name = "dir", long = "git-dir")]
    /// alternative git directory to use
    flag_git_dir: Option<String>,
    //#[clap(name = "skip", long)]
    ///// number of commits to skip
    //flag_skip: Option<usize>,
    //#[clap(name = "max-count", short = 'n', long)]
    ///// maximum number of commits to show
    //flag_max_count: Option<usize>,
    //#[clap(name = "merges", long)]
    ///// only show merge commits
    //flag_merges: bool,
    //#[clap(name = "no-merges", long)]
    ///// don't show merge commits
    //flag_no_merges: bool,
    //#[clap(name = "no-min-parents", long)]
    ///// don't require a minimum number of parents
    //flag_no_min_parents: bool,
    //#[clap(name = "no-max-parents", long)]
    ///// don't require a maximum number of parents
    //flag_no_max_parents: bool,
    //#[clap(name = "max-parents")]
    ///// specify a maximum number of parents for a commit
    //flag_max_parents: Option<usize>,
    //#[clap(name = "min-parents")]
    ///// specify a minimum number of parents for a commit
    //flag_min_parents: Option<usize>,
    #[clap(name = "patch", long, short)]
    /// show commit diff
    flag_patch: bool,
    #[clap(
        name = "nsec",
        default_value = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    )]
    arg_nsec: Option<String>,
    #[clap(name = "commit")]
    arg_commit: Vec<String>,
    #[clap(name = "spec", last = true)]
    arg_spec: Vec<String>,
    #[clap(long)]
    arg_dump: bool,
}

pub async fn run(args: &CliArgs) -> Result<()> {

    let _run_async = async {
        let app_keys = Keys::parse(args.arg_nsec.clone().as_ref().expect("REASON")).unwrap();
        let relay_client = Client::new(app_keys);
        let _ = relay_client
            .send_event_builder(EventBuilder::text_note("#gnostr"))
            .await;
    };

    let app_keys = Keys::parse(args.arg_nsec.clone().as_ref().expect("REASON")).unwrap();
    let processor = Processor::new();
    let mut relay_manager = RelayManager::new(app_keys, processor).await;
    let bootstrap_relay_refs: Vec<&str> = BOOTSTRAP_RELAYS.iter().map(|s| s.as_str()).collect();
    let _run_async = relay_manager.run(bootstrap_relay_refs).await?;

     if args.arg_dump {
        relay_manager.processor.dump();
    }

    Ok(())
}

pub async fn dispatch_cli_command(cli: Cli, client: &reqwest::Client) -> Result<(), Box<dyn std::error::Error>> {
    match &cli.command {
        Commands::Sniper { nip, shitlist } => {
            run_sniper(*nip, shitlist.clone(), client).await?;
        }
        Commands::Watch { shitlist } => {
            run_watch(shitlist.clone(), client).await?;
        }
        Commands::Nip34 { shitlist } => {
            run_nip34(shitlist.clone(), client).await?;
        }
        Commands::Crawl(args) => {
            crate::run(args).await?;
        }
        Commands::Serve { port, detach } => {
            if *detach {
                run_api_server_detached(&["serve"], *port)?;
            } else {
                run_api_server(*port).await?;
            }
        }
    }
    Ok(())
}

pub fn sig_matches(sig: &Signature, arg: &Option<String>) -> bool {
    match *arg {
        Some(ref s) => {
            sig.name().map(|n| n.contains(s)).unwrap_or(false)
                || sig.email().map(|n| n.contains(s)).unwrap_or(false)
        }
        None => true,
    }
}

pub fn log_message_matches(msg: Option<&str>, grep: &Option<String>) -> bool {
    match (grep, msg) {
        (&None, _) => true,
        (&Some(_), None) => false,
        (Some(s), Some(msg)) => msg.contains(s),
    }
}

pub fn print_commit(commit: &Commit) {
    //println!("commit {}", commit.id());

    if commit.parents().len() > 1 {
        print!("Merge:");
        for id in commit.parent_ids() {
            print!(" {:.8}", id);
        }
        println!();
    }

    let author = commit.author();
    println!("Author: {}", author);
    print_time(&author.when(), "Date:   ");
    println!();

    for line in String::from_utf8_lossy(commit.message_bytes()).lines() {
        println!("    {}", line);
    }
    println!();
}

pub fn print_time(time: &Time, prefix: &str) {
    let (offset, sign) = match time.offset_minutes() {
        n if n < 0 => (-n, '-'),
        n => (n, '+'),
    };
    let (hours, minutes) = (offset / 60, offset % 60);
    let ts = Timespec::new(time.seconds() + (time.offset_minutes() as i64) * 60, 0);
    let time = at(ts);

    println!(
        "{}{} {}{:02}{:02}",
        prefix,
        time.strftime("%a %b %e %T %Y").unwrap(),
        sign,
        hours,
        minutes
    );
}

pub fn match_with_parent(
    repo: &Repository,
    commit: &Commit,
    parent: &Commit,
    opts: &mut DiffOptions,
) -> Result<bool, Error> {
    let a = parent.tree()?;
    let b = commit.tree()?;
    let diff = repo.diff_tree_to_tree(Some(&a), Some(&b), Some(opts))?;
    Ok(diff.deltas().len() > 0)
}

pub async fn run_sniper(
    nip_lower: i32,
    shitlist_path: Option<String>,
    client: &reqwest::Client,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("lib::run_sniper");

    //TODO run_watcher populates relays.yaml
    // add async background thread here
    // allow to run for a few seconds
    // giving the sniper a populated list


    // Allow some time for the watcher to populate relays.yaml
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    info!("run_sniper: Finished initial sleep.");

    let relays = load_relays_or_bootstrap();
    info!("run_sniper: Loaded {} relays from relays.yaml.", relays.len());

    let shitlist = if let Some(path) = shitlist_path {
        match load_shitlist(&path) {
            Ok(sl) => sl,
            Err(e) => {
                eprintln!("Failed to load shitlist from {}: {}", path, e);
                return Err(e.into());
            }
        }
    } else {
        std::collections::HashSet::new()
    };
    info!("run_sniper: Shitlist loaded. Contains {} entries.", shitlist.len());

    let initial_relay_count = relays.len();
    let filtered_relays: Vec<String> = relays
        .into_iter()
        .filter(|url| {
            if shitlist.is_empty() {
                true
            } else {
                let is_shitlisted = shitlist
                    .iter()
                    .any(|shitlisted_url| url.contains(shitlisted_url));
                if is_shitlisted {
                    info!("run_sniper: Filtering out shitlisted relay: {}", url);
                }
                !is_shitlisted
            }
        })
        .collect();
    info!("run_sniper: Filtered from {} to {} relays.", initial_relay_count, filtered_relays.len());

    let bodies = stream::iter(filtered_relays)
        .map(|url| {
            info!("run_sniper: Processing URL: {}", url);
            let client = client.clone();
            async move {
                let http_url = url.replace("wss://", "https://").replace("ws://", "http://");
                info!("run_sniper: Sending request to: {}", http_url);
                let resp = client
                    .get(&http_url)
                    .header(ACCEPT, "application/nostr+json")
                    .send()
                    .await?;

                if !resp.status().is_success() {
                    warn!("run_sniper: Failed to fetch NIP-11 document for {}: HTTP Status {}", url, resp.status());
                    return Ok((url, String::new())); // Return empty string to skip JSON parsing
                }

                info!("run_sniper: Received response status: {:?}", resp.status());
                let text = resp.text().await?;
                info!("run_sniper: Raw response text from {}: {}", http_url, text); // Added info log

                let r: Result<(String, String), reqwest::Error> = Ok((url.clone(), text.clone()));
                r
            }
        })
        .buffer_unordered(CONCURRENT_REQUESTS);

    bodies
        .for_each(|b: Result<(String, String), reqwest::Error>| async {
            if let Ok((url, json_string)) = b {
                let data: Result<Relay, _> = serde_json::from_str(&json_string);
                match data {
                    Ok(relay_info) => {
                        info!("run_sniper: Successfully parsed relay info for {}", url);
                        for n in &relay_info.supported_nips.unwrap_or_default() {
                            if n == &nip_lower {
                                info!("run_sniper: Found NIP-{} support on relay: {}", nip_lower, url);
                                info!("contact:{:?}", &relay_info.contact);
                                info!("description:{:?}", &relay_info.description);
                                info!("name:{:?}", &relay_info.name);
                                info!("software:{:?}", &relay_info.software);
                                info!("version:{:?}", &relay_info.version);

                                let parsed_url = match Url::parse(&url) {
                                    Ok(u) => u,
                                    Err(e) => {
                                        error!("Failed to parse URL {}: {}", url, e);
                                        return;
                                    }
                                };
                                let host = parsed_url.host_str().unwrap_or("unknown");
                                info!("run_sniper: Host for {} is {}", url, host);

                                let dir_path = crate::relays::get_config_dir_path().join(format!("{}", nip_lower));
                                if let Err(e) = sync_fs::create_dir_all(&dir_path) {
                                    error!("Failed to create directory {}: {}", dir_path.display(), e);
                                    return;
                                };
                                info!("run_sniper: Ensured directory exists: {}", dir_path.display());

                                let file_name = format!("{}.json", host);
                            let file_path = dir_path.join(&file_name);
                            let file_path_str = file_path.display().to_string();
                            info!("run_sniper: Attempting to write to file: {}\n\n{}", file_path_str, file_path_str);

                                match sync_fs::File::create(&file_path) {
                                    Ok(mut file) => {
                                        info!("run_sniper: File created: {}", &file_path_str);
                                        match file.write_all(json_string.as_bytes()) {
                                            Ok(_) => info!("run_sniper: Wrote relay metadata to: {}", &file_path_str),
                                            Err(e) => {
                                                error!("Failed to write to {}: {}", &file_path_str, e)
                                            }
                                        }
                                    }
                                    Err(e) => error!("Failed to create file {}: {}", &file_path_str, e),
                                }

                                info!(
                                    "run_sniper: Processed NIP {} for relay: {}/{}",
                                    nip_lower,
                                    nip_lower,
                                    url.replace("https://", "")
                                        .replace("wss://", "")
                                        .replace("ws://", "")
                                );
                            } else {
                                trace!("run_sniper: Relay {} does not support NIP-{}", url, nip_lower);
                            }
                        }
                    },
                    Err(e) => {
                        error!("run_sniper: Failed to parse JSON for {}: {}. JSON: {}", url, e, json_string);
                    }
                }
            } else if let Err(e) = b {
                error!("run_sniper: Error fetching relay data: {}", e);
            }
        })
        .await;

    Ok(())
}

pub async fn run_watch(shitlist_path: Option<String>, client: &reqwest::Client) -> Result<(), Box<dyn std::error::Error>> {
    debug!("lib::run_watch");
    let app_secret_key = SecretKey::from_bech32(APP_SECRET_KEY)?;
    let app_keys = Keys::new(app_secret_key);
    let processor = Processor::new();
    let mut relay_manager = RelayManager::new(app_keys, processor).await;

    let bootstrap_relays: Vec<&str> = BOOTSTRAP_RELAYS.iter().map(|s| s.as_str()).collect();
    relay_manager.run(bootstrap_relays).await?;
    let relays: Vec<String> = relay_manager.relays.get_all();

    let shitlist = if let Some(path) = shitlist_path {
        match load_shitlist(&path) {
            Ok(sl) => sl,
            Err(e) => {
                eprintln!("Failed to load shitlist from {}: {}", path, e);
                return Err(e.into());
            }
        }
    } else {
        std::collections::HashSet::new()
    };

    let relays_iterator = relays.into_iter().filter(|url: &String| {
        if shitlist.is_empty() {
            true
        } else {
            !shitlist
                .iter()
                .any(|shitlisted_url| url.contains(shitlisted_url))
        }
    });

    let bodies = stream::iter(relays_iterator)
        .map(|url: String| {
            let client = client.clone();
            async move {
                let resp = client
                    .get(
                        url.replace("wss://", "https://")
                            .replace("ws://", "http://"),
                    )
                    .header(ACCEPT, "application/nostr+json")
                    .send()
                    .await?;
                let text = resp.text().await?;

                //TODO parse response and detect errors
                Ok((url, text))
            }
        })
        .buffer_unordered(CONCURRENT_REQUESTS);

    bodies
        .for_each(|b: Result<(String, String), reqwest::Error>| async {
            if let Ok((url, json_string)) = b {
                //TODO parse json_string data detect errors and add to shitlist
                trace!("{{\"relay\":\"{}\", \"data\":{}}}", url, json_string);
                let data: Result<Relay, serde_json::Error> = serde_json::from_str(&json_string);
                if let Ok(relay_info) = data {
                    //print!("{{\"nips\":\"");
                    let supported_nips = relay_info.supported_nips.unwrap_or_default();
                    let mut nip_count = supported_nips.len();
                    for n in &supported_nips {
                        trace!("nip_count:{}", nip_count);
                        if nip_count > 1 {
                              debug!("run_watch::bodies::nip-count > 1 -- {:0>2} ", n);
                              trace!("LINE::581 lib::run_watch");
                              let _ = run_sniper(*n, None, client).await;
                        } else {
                             trace!("{:0>2}", n);
                             //TODO nip_count < 1 -- add to shitlist? 
                        }
                        nip_count -= 1;
                    }
                    //print!("}}");
                    //println!();
                }
            }
        })
        .await;

    // Add the processor.dump() call here
    //relay_manager.processor.dump();

    Ok(())
}

pub async fn run_nip34(shitlist_path: Option<String>, client: &reqwest::Client) -> Result<(), Box<dyn std::error::Error>> {
    let relays = load_relays_or_bootstrap();

    let shitlist = if let Some(path) = shitlist_path {
        match load_shitlist(&path) {
            Ok(sl) => sl,
            Err(e) => {
                eprintln!("Failed to load shitlist from {}: {}", path, e);
                return Err(e.into());
            }
        }
    } else {
        std::collections::HashSet::new()
    };

    let filtered_relays: Vec<String> = relays
        .into_iter()
        .filter(|url| {
            if shitlist.is_empty() {
                true
            } else {
                !shitlist
                    .iter()
                    .any(|shitlisted_url| url.contains(shitlisted_url))
            }
        })
        .collect();

    let bodies = stream::iter(filtered_relays)
        .map(|url| {
            let client = client.clone();
            async move {
                let resp = client
                    .get(
                        url.replace("wss://", "https://")
                            .replace("ws://", "http://"),
                    )
                    .header(ACCEPT, "application/nostr+json")
                    .send()
                    .await?;
                let text = resp.text().await?;

                let r: Result<(String, String), reqwest::Error> = Ok((url.clone(), text.clone()));
                r
            }
        })
        .buffer_unordered(CONCURRENT_REQUESTS);

    bodies
        .for_each(|b: Result<(String, String), reqwest::Error>| async {
            if let Ok((url, json_string)) = b {
                let data: Result<Relay, _> = serde_json::from_str(&json_string);
                if let Ok(relay_info) = data {
                    let supported_nips = relay_info.supported_nips.unwrap_or_default();
                    let _supports_nip01 = supported_nips.contains(&1);
                    let _supports_nip11 = supported_nips.contains(&11);
                    let supports_nip34 = supported_nips.contains(&34);

                    //if _supports_nip01 && _supports_nip11 {
                    if supports_nip34 {
                        println!("{}", url);
                    }
                }
            }
        })
        .await;

    Ok(())
}

pub async fn run_api_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    debug!("run_api_server: Starting API server on port {}", port);

    let client = reqwest::Client::new();
    if let Err(e) = crate::relays::write_relays_serve_files() {
        warn!("Failed to prepare relay serve files: {}", e);
    }
    crate::relays::prime_live_kinds_from_disk();
    if let Err(e) = crate::relays::write_kinds_serve_files() {
        warn!("Failed to prepare kinds serve files: {}", e);
    }
    if let Err(e) = crate::relays::write_index_html() {
        warn!("Failed to prepare index.html: {}", e);
    }

    // Start the watch process in a separate asynchronous task
    let client_for_watch = client.clone();
    tokio::task::spawn(async move {
        if let Err(e) = run_watch(None, &client_for_watch).await {
            error!("Watch process failed: {}", e);
        }
    });

    let client_for_sniper = client.clone();
    spawn(async move {
        run_sniper_service(client_for_sniper).await;
    });

    let app = Router::new()
        .route("/", get(get_index_html))
        .route("/query", get(get_query))
        .route("/relays.yaml", get(get_relays_yaml))
        .route("/relays.json", get(get_relays_json))
        .route("/relays.txt", get(get_relays_txt))
        .route("/:nip", get(get_nip_index))
        .route("/:nip/query", get(get_nip_query))
        .route("/:nip/relays.yaml", get(get_nip_relays_yaml))
        .route("/:nip/relays.json", get(get_nip_relays_json))
        .route("/:nip/relays.txt", get(get_nip_relays_txt))
        .route("/:nip/:relay.json", get(get_nip_relay_json))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().include_headers(true))
                .on_response(trace::DefaultOnResponse::new().include_headers(true)),
        );

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    info!("run_api_server: listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

pub fn run_api_server_detached(
    command_prefix: &[&str],
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let current_exe = std::env::current_exe()?;
    let mut command = Command::new(current_exe);
    command.args(command_prefix);
    command.arg("--port");
    command.arg(port.to_string());
    command.stdin(Stdio::null());
    command.stdout(Stdio::null());
    command.stderr(Stdio::null());

    let child = command.spawn()?;
    println!("run_api_server_detached: started background server (pid: {})", child.id());
    Ok(())
}

async fn get_relays_yaml() -> Response {
    let config_dir = crate::relays::get_config_dir_path();
    let file_path = config_dir.join("relays.yaml");
    debug!("Attempting to serve relays.yaml from: {}", file_path.display());

    if !file_path.exists() {
        if let Err(e) = crate::relays::write_relays_serve_files() {
            error!("Failed to create relays.yaml: {}", e);
        }
    }

    match fs::read_to_string(&file_path).await {
        Ok(content) => {
            let relays: Vec<String> = content.lines()
                .filter(|line| !line.trim().is_empty())
                .map(String::from)
                .collect();

            match serde_yaml::to_string(&relays) {
                Ok(yaml_content) => {
                    Response::builder()
                        .status(StatusCode::OK)
                        .header(CONTENT_TYPE, "application/x-yaml")
                        .body(Body::from(yaml_content))
                        .unwrap_or_else(|e| {
                            error!("Failed to build YAML response: {}", e);
                            (StatusCode::INTERNAL_SERVER_ERROR, Body::from("Internal Server Error")).into_response()
                        })
                },
                Err(e) => {
                    error!("Failed to serialize relays to YAML: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, Body::from(format!("Failed to serialize relays to YAML: {}", e))).into_response()
                }
            }
        },
        Err(e) => {
            error!("Failed to read relays.yaml: {}. Path: {}", e, file_path.display());
            (StatusCode::INTERNAL_SERVER_ERROR, Body::from(format!("Failed to read relays.yaml: {}", e))).into_response()
        }
    }
}

async fn get_relays_json() -> Response {
    let config_dir = crate::relays::get_config_dir_path();
    let file_path = config_dir.join("relays.json");
    debug!("Attempting to serve relays.json from: {}", file_path.display());

    if !file_path.exists() {
        if let Err(e) = crate::relays::write_relays_serve_files() {
            error!("Failed to create relays.json: {}", e);
        }
    }

    match fs::read_to_string(&file_path).await {
        Ok(content) => {
            Response::builder()
                .status(StatusCode::OK)
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(content))
                .unwrap_or_else(|e| {
                    error!("Failed to build JSON response: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, Body::from("Internal Server Error")).into_response()
                })
        },
        Err(e) => {
            error!("Failed to read relays.json: {}. Path: {}", e, file_path.display());
            (StatusCode::INTERNAL_SERVER_ERROR, Body::from(format!("Failed to read relays.json: {}", e))).into_response()
        }
    }
}

async fn get_relays_txt() -> Response {
    let config_dir = crate::relays::get_config_dir_path();
    let file_path = config_dir.join("relays.txt");
    debug!("Attempting to serve relays.txt from: {}", file_path.display());

    if !file_path.exists() {
        if let Err(e) = crate::relays::write_relays_serve_files() {
            error!("Failed to create relays.txt: {}", e);
        }
    }

    match fs::read_to_string(&file_path).await {
        Ok(content) => {
            Response::builder()
                .status(StatusCode::OK)
                .header(CONTENT_TYPE, "text/plain")
                .body(Body::from(content))
                .unwrap_or_else(|e| {
                    error!("Failed to build TXT response: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, Body::from("Internal Server Error")).into_response()
                })
        },
        Err(e) => {
            error!("Failed to read relays.txt: {}. Path: {}", e, file_path.display());
            (StatusCode::INTERNAL_SERVER_ERROR, Body::from(format!("Failed to read relays.txt: {}", e))).into_response()
        }
    }
}

async fn collect_supported_relays_for_nip(
    nip_lower: i32,
    client: &reqwest::Client,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let relays = load_relays_or_bootstrap();

    let bodies = stream::iter(relays)
        .map(|url| {
            let client = client.clone();
            async move {
                let http_url = url.replace("wss://", "https://").replace("ws://", "http://");
                let resp = client
                    .get(&http_url)
                    .header(ACCEPT, "application/nostr+json")
                    .send()
                    .await?;

                if !resp.status().is_success() {
                    info!(
                        "prime_all_nip_relays_files: skipping {} due to HTTP {}",
                        url,
                        resp.status()
                    );
                    return Ok((url, String::new()));
                }

                let text = resp.text().await?;
                Ok((url, text))
            }
        })
        .buffer_unordered(CONCURRENT_REQUESTS)
        .collect::<Vec<Result<(String, String), reqwest::Error>>>()
        .await;

    let mut supported = Vec::new();
    for item in bodies {
        let (url, json_string) = match item {
            Ok(pair) => pair,
            Err(e) => {
                warn!("Failed to fetch relay metadata for nip {}: {}", nip_lower, e);
                continue;
            }
        };

        let data: Result<Relay, _> = serde_json::from_str(&json_string);
        if let Ok(relay_info) = data {
            if relay_info
                .supported_nips
                .unwrap_or_default()
                .iter()
                .any(|n| *n == nip_lower)
            {
                supported.push(url);
            }
        }
    }

    Ok(supported)
}

async fn prime_all_nip_relays_files(
    client: &reqwest::Client,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("prime_all_nip_relays_files: starting pass");
    let relays = load_relays_or_bootstrap();
    info!(
        "prime_all_nip_relays_files: checking {} relays for NIP support",
        relays.len()
    );
    let bodies = stream::iter(relays)
        .map(|url| {
            let client = client.clone();
            async move {
                info!("prime_all_nip_relays_files: fetching relay metadata for {}", url);
                let http_url = url.replace("wss://", "https://").replace("ws://", "http://");
                let resp = client
                    .get(&http_url)
                    .header(ACCEPT, "application/nostr+json")
                    .send()
                    .await?;

                if !resp.status().is_success() {
                    return Ok((url, String::new()));
                }

                let text = resp.text().await?;
                Ok((url, text))
            }
        })
        .buffer_unordered(CONCURRENT_REQUESTS);

    let mut nip_relays: HashMap<i32, HashSet<String>> = HashMap::new();
    let bodies = bodies.collect::<Vec<Result<(String, String), reqwest::Error>>>().await;
    for item in bodies {
        if let Ok((url, json_string)) = item {
            if json_string.is_empty() {
                info!(
                    "prime_all_nip_relays_files: no metadata body for {}",
                    url
                );
                continue;
            }
            info!(
                "prime_all_nip_relays_files: read metadata for {} ({} bytes)",
                url,
                json_string.len()
            );
            if let Ok(relay_info) = serde_json::from_str::<Relay>(&json_string) {
                let supported_nips = relay_info.supported_nips.unwrap_or_default();
                if supported_nips.is_empty() {
                    info!(
                        "prime_all_nip_relays_files: {} reported no supported_nips",
                        url
                    );
                }
                info!(
                    "prime_all_nip_relays_files: {} supports {:?}",
                    url, supported_nips
                );
                for nip in &supported_nips {
                    let dir_path = crate::relays::get_config_dir_path().join(format!("{}", nip));
                    if let Err(e) = sync_fs::create_dir_all(&dir_path) {
                        warn!("Failed to create nip dir {}: {}", dir_path.display(), e);
                        continue;
                    }
                    if let Ok(parsed_url) = Url::parse(&url) {
                        let host = parsed_url.host_str().unwrap_or("unknown");
                        let file_path = dir_path.join(format!("{}.json", host));
                        info!(
                            "prime_all_nip_relays_files: writing relay metadata to {}",
                            file_path.display()
                        );
                        if let Err(e) = sync_fs::write(&file_path, &json_string) {
                            warn!(
                                "Failed to write individual relay file {}: {}",
                                file_path.display(),
                                e
                            );
                        }
                    }
                }
                for nip in supported_nips {
                    nip_relays.entry(nip).or_default().insert(url.clone());
                }
            } else {
                info!(
                    "prime_all_nip_relays_files: failed to parse relay metadata for {}",
                    url
                );
            }
        } else if let Err(e) = item {
            info!(
                "prime_all_nip_relays_files: request failed while fetching relay metadata: {}",
                e
            );
        }
    }

    for (nip, _) in nip_relays {
        crate::relays::record_live_nips(std::iter::once(nip));
        info!("prime_all_nip_relays_files: rebuilding NIP {} aggregate files", nip);
        if let Err(e) = crate::relays::write_nip_relays_serve_files_from_dir(nip) {
            warn!("Failed to prime nip {} relay files: {}", nip, e);
        }
    }

    info!("prime_all_nip_relays_files: completed pass");
    Ok(())
}

async fn run_sniper_service(client: reqwest::Client) {
    info!("starting sniper service");
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(300));
    interval.tick().await;

    loop {
        info!("run_sniper_service: triggering prime pass");
        if let Err(e) = prime_all_nip_relays_files(&client).await {
            warn!("Sniper service failed: {}", e);
        }
        interval.tick().await;
    }
}

async fn refresh_nip_relays_files(
    nip_lower: i32,
    client: &reqwest::Client,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let relays = collect_supported_relays_for_nip(nip_lower, client).await?;
    let dir = crate::relays::write_nip_relays_serve_files(nip_lower, &relays)?;
    Ok(dir)
}

async fn get_nip_relays_yaml(AxumPath(nip_lower): AxumPath<i32>) -> Response {
    let config_dir = crate::relays::get_config_dir_path().join(nip_lower.to_string());
    let file_path = config_dir.join("relays.yaml");
    debug!("Attempting to serve nip relays.yaml from: {}", file_path.display());

    if !file_path.exists() {
        if let Err(e) = crate::relays::write_nip_relays_serve_files_from_dir(nip_lower) {
            error!("Failed to derive nip relays.yaml from disk: {}", e);
            let client = reqwest::Client::new();
            if let Err(refresh_err) = refresh_nip_relays_files(nip_lower, &client).await {
                error!("Failed to refresh nip {} relay cache: {}", nip_lower, refresh_err);
            }
        }
    }

    info!("get_nip_relays_yaml: reading {}", file_path.display());
    match fs::read_to_string(&file_path).await {
        Ok(content) => Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, "application/x-yaml")
            .body(Body::from(content))
            .unwrap_or_else(|e| {
                error!("Failed to build nip YAML response: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Body::from("Internal Server Error")).into_response()
            }),
        Err(e) => {
            error!("Failed to read nip relays.yaml: {}. Path: {}", e, file_path.display());
            (StatusCode::INTERNAL_SERVER_ERROR, Body::from(format!("Failed to read nip relays.yaml: {}", e))).into_response()
        }
    }
}

async fn get_nip_relays_json(AxumPath(nip_lower): AxumPath<i32>) -> Response {
    let config_dir = crate::relays::get_config_dir_path().join(nip_lower.to_string());
    let file_path = config_dir.join("relays.json");
    debug!("Attempting to serve nip relays.json from: {}", file_path.display());

    if !file_path.exists() {
        if let Err(e) = crate::relays::write_nip_relays_serve_files_from_dir(nip_lower) {
            error!("Failed to derive nip relays.json from disk: {}", e);
            let client = reqwest::Client::new();
            if let Err(refresh_err) = refresh_nip_relays_files(nip_lower, &client).await {
                error!("Failed to refresh nip {} relay cache: {}", nip_lower, refresh_err);
            }
        }
    }

    info!("get_nip_relays_json: reading {}", file_path.display());
    match fs::read_to_string(&file_path).await {
        Ok(content) => Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(content))
            .unwrap_or_else(|e| {
                error!("Failed to build nip JSON response: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Body::from("Internal Server Error")).into_response()
            }),
        Err(e) => {
            error!("Failed to read nip relays.json: {}. Path: {}", e, file_path.display());
            (StatusCode::INTERNAL_SERVER_ERROR, Body::from(format!("Failed to read nip relays.json: {}", e))).into_response()
        }
    }
}

async fn get_nip_relays_txt(AxumPath(nip_lower): AxumPath<i32>) -> Response {
    let config_dir = crate::relays::get_config_dir_path().join(nip_lower.to_string());
    let file_path = config_dir.join("relays.txt");
    debug!("Attempting to serve nip relays.txt from: {}", file_path.display());

    if !file_path.exists() {
        if let Err(e) = crate::relays::write_nip_relays_serve_files_from_dir(nip_lower) {
            error!("Failed to derive nip relays.txt from disk: {}", e);
            let client = reqwest::Client::new();
            if let Err(refresh_err) = refresh_nip_relays_files(nip_lower, &client).await {
                error!("Failed to refresh nip {} relay cache: {}", nip_lower, refresh_err);
            }
        }
    }

    info!("get_nip_relays_txt: reading {}", file_path.display());
    match fs::read_to_string(&file_path).await {
        Ok(content) => Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, "text/plain")
            .body(Body::from(content))
            .unwrap_or_else(|e| {
                error!("Failed to build nip TXT response: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Body::from("Internal Server Error")).into_response()
            }),
        Err(e) => {
            error!("Failed to read nip relays.txt: {}. Path: {}", e, file_path.display());
            (StatusCode::INTERNAL_SERVER_ERROR, Body::from(format!("Failed to read nip relays.txt: {}", e))).into_response()
        }
    }
}

async fn get_nip_index(AxumPath(nip_lower): AxumPath<i32>) -> Response {
    let config_dir = crate::relays::get_config_dir_path().join(nip_lower.to_string());
    let default_kinds = nip_lower.to_string();
    fn escape_html(input: &str) -> String {
        input
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }

    let mut entries = vec![
        format!("<li><a href=\"/{}/relays.json\">relays.json</a></li>", nip_lower),
        format!("<li><a href=\"/{}/relays.yaml\">relays.yaml</a></li>", nip_lower),
        format!("<li><a href=\"/{}/relays.txt\">relays.txt</a></li>", nip_lower),
    ];

    if let Ok(mut dir) = fs::read_dir(&config_dir).await {
        let mut relay_cards = Vec::new();
        while let Ok(Some(entry)) = dir.next_entry().await {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".json") && name != "relays.json" {
                let file_path = entry.path();
                match fs::read_to_string(&file_path).await {
                    Ok(content) => {
                        let pretty = serde_json::from_str::<serde_json::Value>(&content)
                            .ok()
                            .map(|value| {
                                let relay_name = value
                                    .get("name")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or(&name);
                                let nip_links = value
                                    .get("supported_nips")
                                    .and_then(|v| v.as_array())
                                    .map(|nips| {
                                        let links = nips
                                            .iter()
                                            .filter_map(|nip| nip.as_i64())
                                            .map(|nip| {
                                                format!(
                                                    "<a href=\"/{0}\" style=\"margin-right:0.35rem;\">NIP {0}</a>",
                                                    nip
                                                )
                                            })
                                            .collect::<Vec<_>>()
                                            .join("");
                                        if links.is_empty() {
                                            String::new()
                                        } else {
                                            format!("<div style=\"margin:0.25rem 0 0.5rem 0;\">{}</div>", links)
                                        }
                                    })
                                    .unwrap_or_default();
                                let extension_links = value
                                    .get("supported_nip_extensions")
                                    .and_then(|v| v.as_array())
                                    .map(|extensions| {
                                        let links = extensions
                                            .iter()
                                            .filter_map(|extension| extension.as_str())
                                            .map(|extension| {
                                                format!(
                                                    "<code style=\"margin-right:0.35rem;\">{}</code>",
                                                    escape_html(extension)
                                                )
                                            })
                                            .collect::<Vec<_>>()
                                            .join("");
                                        if links.is_empty() {
                                            String::new()
                                        } else {
                                            format!(
                                                "<div style=\"margin:0.25rem 0 0.5rem 0;\"><strong>supported_nip_extensions</strong>: {}</div>",
                                                links
                                            )
                                        }
                                    })
                                    .unwrap_or_default();
                                let icon_html = value
                                    .get("icon")
                                    .and_then(|v| v.as_str())
                                    .map(|icon| {
                                        format!(
                                            "<div style=\"margin:0.5rem 0;\"><img src=\"{}\" alt=\"icon\" style=\"width:48px;height:48px;object-fit:contain;border-radius:0.35rem;background:rgba(255,255,255,0.06);padding:0.25rem;\"></div>",
                                            escape_html(icon)
                                        )
                                    })
                                    .unwrap_or_default();
                                let pretty = serde_json::to_string_pretty(&value).ok().unwrap_or_default();
                                format!(
                                    "<div><strong>{}</strong></div>{}{}{}<pre>{}</pre>",
                                    escape_html(relay_name),
                                    nip_links,
                                    extension_links,
                                    icon_html,
                                    escape_html(&pretty)
                                )
                            })
                            .unwrap_or_else(|| format!("<pre>{}</pre>", escape_html(&content)));
                        relay_cards.push(format!(
                            "<li><details><summary><a href=\"/{}/{}\">{}</a></summary>{}</details></li>",
                            nip_lower,
                            name,
                            escape_html(&name),
                            pretty
                        ));
                    }
                    Err(e) => {
                        relay_cards.push(format!(
                            "<li><a href=\"/{}/{}\">{}</a> <em>(failed to read metadata: {})</em></li>",
                            nip_lower,
                            name,
                            escape_html(&name),
                            escape_html(&e.to_string())
                        ));
                    }
                }
            }
        }
        relay_cards.sort();
        entries.extend(relay_cards);
    }

    let query_href = format!("/{}/query", nip_lower);
    let nav = vec![
        ("/", "gnostr/crawler"),
        ("/relays.json", "relays.json"),
        ("/relays.yaml", "relays.yaml"),
        ("/relays.txt", "relays.txt"),
    ];
    let query_form = crate::query::forms::nip_query_form(
        nip_lower,
        &query_href,
        "",
        Some(default_kinds.as_str()),
    );
    let body = format!(
        "{}\
          <section><p><a href=\"/\">&larr; back to home</a></p>\
           <h2>NIP {}</h2><ul>{}</ul></section>",
        query_form,
        nip_lower,
        entries.join("")
    );
    let html = crate::relays::render_page_shell(&format!("gnostr crawler / NIP {}", nip_lower), &nav, &body);

    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "text/html")
        .body(Body::from(html))
        .unwrap_or_else(|e| {
            error!("Failed to build nip index response: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Body::from("Internal Server Error")).into_response()
        })
}

fn load_nip_query_relays(nip_lower: i32, relay_override: Option<&str>) -> Result<Vec<Url>, Box<dyn std::error::Error>> {
    if let Some(relay) = relay_override {
        return Ok(vec![Url::parse(relay)?]);
    }

    let config_dir = crate::relays::get_config_dir_path().join(nip_lower.to_string());
    let relays_path = config_dir.join("relays.txt");
    if !relays_path.exists() {
        let _ = crate::relays::write_nip_relays_serve_files_from_dir(nip_lower);
    }

    let content = std::fs::read_to_string(&relays_path)?;
    let relays = content
        .split_whitespace()
        .filter_map(|relay| Url::parse(relay).ok())
        .collect::<Vec<_>>();

    if relays.is_empty() {
        return Err(format!("no relays available for NIP {}", nip_lower).into());
    }

    Ok(relays)
}

fn non_empty_param<'a>(params: &'a HashMap<String, String>, key: &str) -> Option<&'a str> {
    params
        .get(key)
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty())
}

async fn execute_query_page(
    title: &str,
    nav: &[(&str, &str)],
    form_html: &str,
    query_string: String,
    relays: Vec<Url>,
    limit: Option<i32>,
) -> Response {
    let results = match crate::send(query_string.clone(), relays, limit.or(Some(100))).await {
        Ok(results) => results,
        Err(e) => {
            let html = crate::relays::render_page_shell(
                title,
                nav,
                &format!("{}<p>Query failed: {}</p>", form_html, e),
            );
            return Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .header(CONTENT_TYPE, "text/html")
                .body(Body::from(html))
                .unwrap_or_else(|build_err| {
                    error!("Failed to build query failure response: {}", build_err);
                    (StatusCode::INTERNAL_SERVER_ERROR, Body::from("Internal Server Error")).into_response()
                });
        }
    };

    let results_html = if results.is_empty() {
        "<p>No results.</p>".to_string()
    } else {
        format!("<pre>{}</pre>", results.join("\n"))
    };

    let body = format!(
        "{}<section><h2>Query results</h2><p><code>{}</code></p>{}</section>",
        form_html,
        query_string,
        results_html
    );
    let html = crate::relays::render_page_shell(title, nav, &body);

    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "text/html")
        .body(Body::from(html))
        .unwrap_or_else(|e| {
            error!("Failed to build query response: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Body::from("Internal Server Error")).into_response()
        })
}

async fn get_query(Query(params): Query<HashMap<String, String>>) -> Response {
    fn escape_html(input: &str) -> String {
        input
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }

    let relay = non_empty_param(&params, "relay");
    let authors = non_empty_param(&params, "authors");
    let ids = non_empty_param(&params, "ids");
    let limit = params.get("limit").and_then(|value| value.parse::<i32>().ok());
    let kinds = non_empty_param(&params, "kinds");
    let search = non_empty_param(&params, "search");
    let generic_tag = non_empty_param(&params, "generic_tag");
    let generic_value = non_empty_param(&params, "generic_value");
    let hashtag = non_empty_param(&params, "hashtag");
    let mentions = non_empty_param(&params, "mentions");
    let references = non_empty_param(&params, "references");

    let generic = match (generic_tag, generic_value) {
        (Some(tag), Some(value)) => Some((tag, value)),
        _ => None,
    };

    let query_string = match crate::build_gnostr_query(
        authors,
        ids,
        limit,
        generic,
        hashtag,
        mentions,
        references,
        kinds,
        search.map(|s| ("search", s)),
    ) {
        Ok(query) => query,
        Err(e) => {
            let html = crate::relays::render_page_shell(
                "gnostr crawler / query",
                &[("/", "gnostr/crawler")],
                &format!("<p>Failed to build query: {}</p>", e),
            );
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header(CONTENT_TYPE, "text/html")
                .body(Body::from(html))
                .unwrap_or_else(|build_err| {
                    error!("Failed to build query error response: {}", build_err);
                    (StatusCode::INTERNAL_SERVER_ERROR, Body::from("Internal Server Error")).into_response()
                });
        }
    };

    let relays = if let Some(relay) = relay {
        match Url::parse(relay) {
            Ok(url) => vec![url],
            Err(e) => {
                let html = crate::relays::render_page_shell(
                    "gnostr crawler / query",
                    &[("/", "gnostr/crawler")],
                    &format!("<p>Invalid relay URL: {}</p>", e),
                );
                return Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .header(CONTENT_TYPE, "text/html")
                    .body(Body::from(html))
                    .unwrap_or_else(|build_err| {
                        error!("Failed to build relay error response: {}", build_err);
                        (StatusCode::INTERNAL_SERVER_ERROR, Body::from("Internal Server Error")).into_response()
                    });
            }
        }
    } else {
        load_relays_or_bootstrap()
            .into_iter()
            .filter_map(|relay| Url::parse(&relay).ok())
            .collect()
    };

    let kinds_value = crate::relays::live_kinds().join(",");
    let query_form = crate::query::forms::generic_query_form("/query", Some(kinds_value.as_str()));
    let nav = [("/", "gnostr/crawler"), ("/query", "query")];
    execute_query_page(
        "gnostr crawler / query",
        &nav,
        &query_form,
        escape_html(&query_string),
        relays,
        limit,
    )
    .await
}

async fn get_nip_query(
    AxumPath(nip_lower): AxumPath<i32>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    fn escape_html(input: &str) -> String {
        input
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }

    let relay = params
        .get("relay")
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty());
    let authors = params
        .get("authors")
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty());
    let ids = params
        .get("ids")
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty());
    let limit = params
        .get("limit")
        .and_then(|value| value.parse::<i32>().ok());
    let kinds = params
        .get("kinds")
        .map(String::as_str)
        .or(Some("1630,1632,1621,30618,1633,1631,1617,30617"));
    let search = params
        .get("search")
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty());
    let generic_tag = params
        .get("generic_tag")
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty());
    let generic_value = params
        .get("generic_value")
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty());
    let hashtag = params
        .get("hashtag")
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty());
    let mentions = params
        .get("mentions")
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty());
    let references = params
        .get("references")
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty());
    let default_kinds = nip_lower.to_string();
    let query_href = format!("/{}/query", nip_lower);
    let back_href = format!("/{}/", nip_lower);

    let generic = match (generic_tag, generic_value) {
        (Some(tag), Some(value)) => Some((tag, value)),
        _ => None,
    };

    let query_string = match crate::build_gnostr_query(
        authors,
        ids,
        limit,
        generic,
        hashtag,
        mentions,
        references,
        kinds.or(Some(default_kinds.as_str())),
        search.map(|s| ("search", s)),
    ) {
        Ok(query) => query,
        Err(e) => {
            let nav = vec![("/", "gnostr/crawler"), (back_href.as_str(), "back")];
            let html = crate::relays::render_page_shell(
                &format!("gnostr crawler / NIP {} query", nip_lower),
                &nav,
                &format!("<p>Failed to build query: {}</p>", escape_html(&e.to_string())),
            );
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header(CONTENT_TYPE, "text/html")
                .body(Body::from(html))
                .unwrap_or_else(|build_err| {
                    error!("Failed to build query error response: {}", build_err);
                    (StatusCode::INTERNAL_SERVER_ERROR, Body::from("Internal Server Error")).into_response()
                });
        }
    };

    let relays = match load_nip_query_relays(nip_lower, relay) {
        Ok(relays) => relays,
        Err(e) => {
            let nav = vec![("/", "gnostr/crawler"), (back_href.as_str(), "back")];
            let html = crate::relays::render_page_shell(
                &format!("gnostr crawler / NIP {} query", nip_lower),
                &nav,
                &format!("<p>Failed to load relays: {}</p>", escape_html(&e.to_string())),
            );
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header(CONTENT_TYPE, "text/html")
                .body(Body::from(html))
                .unwrap_or_else(|build_err| {
                    error!("Failed to build relay error response: {}", build_err);
                    (StatusCode::INTERNAL_SERVER_ERROR, Body::from("Internal Server Error")).into_response()
                });
        }
    };

    let results = match crate::send(query_string.clone(), relays, limit.or(Some(100))).await {
        Ok(results) => results,
        Err(e) => {
            let nav = vec![("/", "gnostr/crawler"), (back_href.as_str(), "back")];
            let html = crate::relays::render_page_shell(
                &format!("gnostr crawler / NIP {} query", nip_lower),
                &nav,
                &format!("<p>Query failed: {}</p>", escape_html(&e.to_string())),
            );
            return Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .header(CONTENT_TYPE, "text/html")
                .body(Body::from(html))
                .unwrap_or_else(|build_err| {
                    error!("Failed to build query failure response: {}", build_err);
                    (StatusCode::INTERNAL_SERVER_ERROR, Body::from("Internal Server Error")).into_response()
                });
        }
    };

    let query_form = crate::query::forms::nip_query_form(
        nip_lower,
        &query_href,
        relay.unwrap_or(""),
        Some(default_kinds.as_str()),
    );
    let results_html = if results.is_empty() {
        "<p>No results.</p>".to_string()
    } else {
        format!(
            "<pre>{}</pre>",
            escape_html(&results.join("\n"))
        )
    };

    let nav = vec![
        ("/", "gnostr/crawler"),
        (query_href.as_str(), "query"),
        (back_href.as_str(), "back"),
    ];
    let body = format!(
        "{}<section><h2>NIP {} query results</h2><p><code>{}</code></p>{}</section>",
        query_form,
        nip_lower,
        escape_html(&query_string),
        results_html
    );
    let html = crate::relays::render_page_shell(&format!("gnostr crawler / NIP {} query", nip_lower), &nav, &body);

    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "text/html")
        .body(Body::from(html))
        .unwrap_or_else(|e| {
            error!("Failed to build nip query response: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Body::from("Internal Server Error")).into_response()
        })
}

async fn get_nip_relay_json(AxumPath((nip_lower, relay_file)): AxumPath<(i32, String)>) -> Response {
    let config_dir = crate::relays::get_config_dir_path().join(nip_lower.to_string());
    let file_path = config_dir.join(&relay_file);
    debug!("Attempting to serve nip relay file from: {}", file_path.display());

    if !relay_file.ends_with(".json") {
        return (StatusCode::BAD_REQUEST, Body::from("Expected a .json relay file")).into_response();
    }

    info!("get_nip_relay_json: reading {}", file_path.display());
    match fs::read_to_string(&file_path).await {
        Ok(content) => Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(content))
            .unwrap_or_else(|e| {
                error!("Failed to build nip relay JSON response: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Body::from("Internal Server Error")).into_response()
            }),
        Err(e) => {
            error!("Failed to read nip relay json: {}. Path: {}", e, file_path.display());
            (StatusCode::NOT_FOUND, Body::from(format!("Failed to read nip relay json: {}", e))).into_response()
        }
    }
}

async fn get_index_html() -> Response {
    let config_dir = crate::relays::get_config_dir_path();
    let file_path = config_dir.join("index.html");
    debug!("Attempting to serve index.html from: {}", file_path.display());

    if !file_path.exists() {
        if let Err(e) = crate::relays::write_index_html() {
            error!("Failed to create index.html: {}", e);
        }
    }

    match fs::read_to_string(&file_path).await {
        Ok(content) => Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, "text/html")
            .body(Body::from(content))
            .unwrap_or_else(|e| {
                error!("Failed to build HTML response: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Body::from("Internal Server Error")).into_response()
            }),
        Err(e) => {
            error!("Failed to read index.html: {}. Path: {}", e, file_path.display());
            (StatusCode::INTERNAL_SERVER_ERROR, Body::from(format!("Failed to read index.html: {}", e))).into_response()
        }
    }
}
