pub mod processor;
pub mod pubkeys;
pub mod relay_manager;
pub mod relays;
pub mod stats;

use clap::{Parser, Subcommand};
use futures::{stream, StreamExt};
use git2::Error;
use git2::{Commit, DiffOptions, Repository, Signature, Time};
use reqwest::header::ACCEPT;
use std::collections::HashSet;
use std::fs as sync_fs;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use std::str;
#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

use serde::{Deserialize, Serialize};

use ::time::at;
use ::time::Timespec;
use nostr_sdk::prelude::*;
use url::Url;

use crate::processor::Processor;
use crate::processor::APP_SECRET_KEY;
use crate::relay_manager::RelayManager;

#[allow(unused_imports)]
use crate::processor::LOCALHOST_8080;
use crate::processor::BOOTSTRAP_RELAYS;

use axum::{
    routing::get,
    response::{IntoResponse, Response},
    Router,
    body::Body, // Added for explicit body type
    http::{StatusCode, header::CONTENT_TYPE}, // Changed to axum::http
};
use std::net::SocketAddr;
use tokio::fs; // For async file operations
use tower_http::trace::{self, TraceLayer}; // For logging requests

const CONCURRENT_REQUESTS: usize = 16;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
    //nsec: Option<String>,
}

#[derive(Subcommand, Debug)]
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
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Relay {
    pub contact: Option<String>,
    pub description: Option<String>,
    pub name: Option<String>,
    pub software: Option<String>,
    pub supported_nips: Option<Vec<i32>>,
    pub version: Option<String>,
}

pub fn load_file(filename: impl AsRef<Path>) -> io::Result<Vec<String>> {
     let base_dir = crate::relays::get_config_dir_path();

     let file_path = base_dir.join(filename.as_ref().file_name().unwrap_or(filename.as_ref().as_os_str()));

     if let Some(parent) = file_path.parent() {
         sync_fs::create_dir_all(parent)?;
     }

     debug!("Loading file: {}", file_path.display());

     let file_content = BufReader::new(sync_fs::OpenOptions::new().read(true).write(true).create(true).open(file_path)?).lines().collect::<io::Result<Vec<String>>>()?;

     let filtered_relays: Vec<String> = file_content.into_iter()
         .filter_map(|line| {
             let trimmed_line = line.trim();
             if trimmed_line.starts_with("wss://") || trimmed_line.starts_with("ws://") {
                 match Url::parse(trimmed_line) {
                     Ok(url) => Some(url.to_string()),
                     Err(_) => {
                         warn!("Skipping invalid URL in relays.yaml: {}", trimmed_line);
                         None
                     }
                 }
             } else {
                 warn!("Skipping non-websocket URL in relays.yaml: {}", trimmed_line);
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

#[allow(clippy::manual_strip)]
#[derive(Parser, Debug)]
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
        let opts = Options::new(); //.wait_for_send(true);
        let app_keys = Keys::from_sk_str(args.arg_nsec.clone().as_ref().expect("REASON")).unwrap();
        let relay_client = Client::new_with_opts(&app_keys, opts);
        let _ = relay_client.publish_text_note("#gnostr", &[]).await;
    };

    let app_keys = Keys::from_sk_str(args.arg_nsec.clone().as_ref().expect("REASON")).unwrap();
    let processor = Processor::new();
    let mut relay_manager = RelayManager::new(app_keys, processor).await;
    let bootstrap_relay_refs: Vec<&str> = BOOTSTRAP_RELAYS.iter().map(|s| s.as_str()).collect();
    let _run_async = relay_manager.run(bootstrap_relay_refs).await?;

     if args.arg_dump {
        relay_manager.processor.dump();
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
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("lib::run_sniper");

    //TODO run_watcher populates relays.yaml
    // add async background thread here
    // allow to run for a few seconds
    // giving the sniper a populated list


    // Allow some time for the watcher to populate relays.yaml
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    debug!("run_sniper: Finished initial sleep.");

    let relays = load_file("relays.yaml").unwrap();
    debug!("run_sniper: Loaded {} relays from relays.yaml.", relays.len());
    let client = reqwest::Client::new();

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
    debug!("run_sniper: Shitlist loaded. Contains {} entries.", shitlist.len());

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
                    debug!("run_sniper: Filtering out shitlisted relay: {}", url);
                }
                !is_shitlisted
            }
        })
        .collect();
    debug!("run_sniper: Filtered from {} to {} relays.", initial_relay_count, filtered_relays.len());

    let bodies = stream::iter(filtered_relays)
        .map(|url| {
            debug!("run_sniper: Processing URL: {}", url);
            let client = client.clone();
            async move {
                let http_url = url.replace("wss://", "https://").replace("ws://", "http://");
                debug!("run_sniper: Sending request to: {}", http_url);
                let resp = client
                    .get(&http_url)
                    .header(ACCEPT, "application/nostr+json")
                    .send()
                    .await?;

                if !resp.status().is_success() {
                    warn!("run_sniper: Failed to fetch NIP-11 document for {}: HTTP Status {}", url, resp.status());
                    return Ok((url, String::new())); // Return empty string to skip JSON parsing
                }

                debug!("run_sniper: Received response status: {:?}", resp.status());
                let text = resp.text().await?;
                debug!("run_sniper: Raw response text from {}: {}", http_url, text); // Added debug log

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
                        debug!("run_sniper: Successfully parsed relay info for {}", url);
                        for n in &relay_info.supported_nips.unwrap_or_default() {
                            if n == &nip_lower {
                                debug!("run_sniper: Found NIP-{} support on relay: {}", nip_lower, url);
                                debug!("contact:{:?}", &relay_info.contact);
                                debug!("description:{:?}", &relay_info.description);
                                debug!("name:{:?}", &relay_info.name);
                                debug!("software:{:?}", &relay_info.software);
                                debug!("version:{:?}", &relay_info.version);

                                let parsed_url = match Url::parse(&url) {
                                    Ok(u) => u,
                                    Err(e) => {
                                        error!("Failed to parse URL {}: {}", url, e);
                                        return;
                                    }
                                };
                                let host = parsed_url.host_str().unwrap_or("unknown");
                                debug!("run_sniper: Host for {} is {}", url, host);

                                let dir_path = crate::relays::get_config_dir_path().join(format!("{}", nip_lower));
                                if let Err(e) = sync_fs::create_dir_all(&dir_path) {
                                    error!("Failed to create directory {}: {}", dir_path.display(), e);
                                    return;
                                };
                                debug!("run_sniper: Ensured directory exists: {}", dir_path.display());

                                let file_name = format!("{}.json", host);
                            let file_path = dir_path.join(&file_name);
                            let file_path_str = file_path.display().to_string();
                            debug!("run_sniper: Attempting to write to file: {}\n\n{}", file_path_str, file_path_str);

                                match sync_fs::File::create(&file_path) {
                                    Ok(mut file) => {
                                        debug!("run_sniper: File created: {}", &file_path_str);
                                        match file.write_all(json_string.as_bytes()) {
                                            Ok(_) => debug!("run_sniper: Wrote relay metadata to: {}", &file_path_str),
                                            Err(e) => {
                                                error!("Failed to write to {}: {}", &file_path_str, e)
                                            }
                                        }
                                    }
                                    Err(e) => error!("Failed to create file {}: {}", &file_path_str, e),
                                }

                                debug!(
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

pub async fn run_watch(shitlist_path: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
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

    let client = reqwest::Client::new();
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
                              let _ = run_sniper(*n, None).await;
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

pub async fn run_nip34(shitlist_path: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let relays = load_file("relays.yaml").unwrap();
    let client = reqwest::Client::new();

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

    let app = Router::new()
        .route("/relays.yaml", get(get_relays_yaml))
        .route("/relays.json", get(get_relays_json))
        .route("/relays.txt", get(get_relays_txt))
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

async fn get_relays_yaml() -> Response {
    let config_dir = crate::relays::get_config_dir_path();
    let file_path = config_dir.join("relays.yaml");
    debug!("Attempting to serve relays.yaml from: {}", file_path.display());

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
    let file_path = config_dir.join("relays.yaml"); // Use relays.yaml as source
    debug!("Attempting to serve relays.txt (from relays.yaml) from: {}", file_path.display());

    match fs::read_to_string(&file_path).await {
        Ok(content) => {
            match serde_yaml::from_str::<Vec<String>>(&content) {
                Ok(relays) => {
                    let relays_output = relays.join(" ");
                    Response::builder()
                        .status(StatusCode::OK)
                        .header(CONTENT_TYPE, "text/plain")
                        .body(Body::from(relays_output))
                        .unwrap_or_else(|e| {
                            error!("Failed to build TXT response: {}", e);
                            (StatusCode::INTERNAL_SERVER_ERROR, Body::from("Internal Server Error")).into_response()
                        })
                },
                Err(e) => {
                    error!("Failed to parse relays.yaml for relays.txt: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, Body::from(format!("Failed to parse relays.yaml for relays.txt: {}", e))).into_response()
                }
            }
        },
        Err(e) => {
            error!("Failed to read relays.yaml for relays.txt: {}. Path: {}", e, file_path.display());
            (StatusCode::INTERNAL_SERVER_ERROR, Body::from(format!("Failed to read relays.yaml for relays.txt: {}", e))).into_response()
        }
    }
}
