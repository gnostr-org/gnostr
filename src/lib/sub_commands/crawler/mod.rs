//! Crawler subcommand for gnostr.
//!
//! This module contains the implementation of the `crawler` subcommand for the `gnostr` CLI.
//! It includes logic for scraping Nostr relay information, monitoring relays, and serving
//! relay data via a web API.

pub mod processor;
pub mod pubkeys;
pub mod relay_manager;
pub mod relays;
pub mod stats;

pub fn init_tracing() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env()
        .add_directive("nostr_sdk_0_19_1::relay=off".parse()?)
        //.add_directive("hyper=off".parse()?)

        /**/)/**/
        .init();
    Ok(())
}

use clap::{Parser, Subcommand};
use futures::{stream, StreamExt};
use reqwest::header::ACCEPT;
use std::collections::HashSet;
use std::fs as sync_fs;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use std::str;
#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

use serde::{Deserialize, Serialize};



use nostr_sdk_0_19_1::prelude::*;
use ::url::Url;
use relays::get_config_dir_path;

use processor::Processor;
use processor::APP_SECRET_KEY;
use relay_manager::RelayManager;
use processor::BOOTSTRAP_RELAYS;

use axum::{
    routing::get,
    response::{IntoResponse, Response},
    Router,
    body::Body, // Added for explicit body type
    http::{StatusCode, header::CONTENT_TYPE}, // Changed to axum::http
};
use std::net::SocketAddr;
use tokio::fs; // For async file operations
#[allow(unused_imports)] // Suppress false positive for tokio::task::spawn
use tokio::task::spawn; // Added for spawning async tasks
use tower_http::trace::{self, TraceLayer}; // For logging requests

const CONCURRENT_REQUESTS: usize = 16;

#[derive(Subcommand, Debug, Clone)]
pub enum InnerCrawlerCommand {
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

#[derive(clap::Args, Debug, Clone)]
#[command(author, version, about = "Gnostr Crawler Subcommand", long_about = None)]
pub struct CrawlerSubCommand {
    #[clap(subcommand)]
    pub command: InnerCrawlerCommand,
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

pub fn preprocess_line(line: &str) -> String {
    let mut trimmed_line = line.trim().to_string();
    // Truncate at the first comma, if any
    if let Some(comma_idx) = trimmed_line.find(',') {
        trimmed_line.truncate(comma_idx);
        trimmed_line = trimmed_line.trim().to_string(); // Re-trim after truncation
    }
    trimmed_line
}

pub fn load_file(filename: impl AsRef<Path>) -> io::Result<Vec<String>> {
    let base_dir = get_config_dir_path();
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

    let preprocessed_content_for_yaml = preprocessed_lines.join("
");

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


pub fn load_shitlist(filename: impl AsRef<Path>) -> io::Result<HashSet<String>> {
    BufReader::new(sync_fs::File::open(filename)?).lines().collect()
}

#[allow(clippy::manual_strip)]
#[derive(clap::Args, Debug, Clone)]
pub struct CliArgs {
    #[clap(name = "dir", long = "git-dir")]
    /// alternative git directory to use
    flag_git_dir: Option<String>,
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

// These functions will need `crate::relays::get_config_dir_path()` and module imports
// once the modules are moved. I will add placeholders for now.

pub async fn run(args: &CliArgs) -> Result<(), Box<dyn std::error::Error>> {

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

pub async fn run_sniper(
    nip_lower: i32,
    shitlist_path: Option<String>,
    client: &reqwest::Client,
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

                                let dir_path = get_config_dir_path().join(format!("{}", nip_lower));
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
    let relays = load_file("relays.yaml").unwrap();

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

    // Start the watch process in a separate asynchronous task
    let client_for_watch = client.clone();
    tokio::task::spawn(async move {
        if let Err(e) = run_watch(None, &client_for_watch).await {
            error!("Watch process failed: {}", e);
        }
    });

    let app = Router::new()
        .route("/", get(get_index_html))
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

// Handlers for the API server - placeholders
async fn get_relays_yaml() -> Response {
    let config_dir = get_config_dir_path();
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
    let config_dir = get_config_dir_path();
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
    let config_dir = get_config_dir_path();
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

async fn get_index_html() -> Response {
    let html_content = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Gnostr Crawler Endpoints</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
            margin: 0;
            line-height: 1.6;
            background-color: #1a1a1a;
            color: #e0e0e0;
            display: flex;
            justify-content: center;
            align-items: center;
            min-height: 100vh;
        }
        .container {
            max-width: 800px;
            margin: 1em;
            background: #2c2c2c;
            padding: 2em;
            border-radius: 8px;
            box-shadow: 0 4px 8px rgba(0,0,0,0.3);
            text-align: center;
        }
        .header {
            display: flex;
            align-items: center;
            justify-content: center;
            margin-bottom: 1.5em;
        }
        .header svg {
            width: 50px;
            height: 50px;
            margin-right: 15px;
            fill: #9a4fe6; /* Adjusted to a purple color */
        }
        h1 {
            color: #e0e0e0;
            font-weight: 300;
        }
        p {
            font-size: 1.1em;
        }
        ul {
            list-style-type: none;
            padding: 0;
            margin-top: 1.5em;
        }
        li {
            margin-bottom: 1em;
            background: #3c3c3c;
            padding: 0.8em;
            border-radius: 5px;
            transition: background-color 0.3s ease;
        }
        li:hover {
            background-color: #4c4c4c;
        }
        a {
            color: #a56de2; /* Lighter purple for links */
            text-decoration: none;
            font-weight: bold;
        }
        a:hover {
            text-decoration: underline;
        }
        footer {
            margin-top: 2em;
            font-size: 0.9em;
            color: #888;
        }
        @media (max-width: 600px) {
            body {
                margin: 0;
            }
            .container {
                margin: 0;
                border-radius: 0;
                min-height: 100vh;
                padding: 1em;
            }
            h1 {
                font-size: 1.5em;
            }
            .header svg {
                width: 40px;
                height: 40px;
            }
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1024 1024">
                <g>
                    <path style="fill-rule:nonzero;" d="M 978.710938 468.800781 L 555.1875 45.296875 C 530.800781 20.902344 491.246094 20.902344 466.839844 45.296875 L 378.902344 133.257812 L 490.453125 244.808594 C 516.382812 236.046875 546.109375 241.929688 566.769531 262.597656 C 587.554688 283.394531 593.382812 313.375 584.398438 339.398438 L 691.929688 446.917969 C 717.949219 437.945312 747.953125 443.75 768.726562 464.558594 C 797.757812 493.589844 797.757812 540.628906 768.726562 569.683594 C 739.675781 598.722656 692.636719 598.722656 663.574219 569.683594 C 641.742188 547.828125 636.335938 515.738281 647.410156 488.839844 L 547.121094 388.558594 L 547.121094 652.445312 C 554.195312 655.953125 560.878906 660.617188 566.769531 666.507812 C 595.800781 695.539062 595.800781 742.578125 566.769531 771.640625 C 537.738281 800.664062 490.664062 800.664062 461.65625 771.640625 C 432.625 742.570312 432.625 695.527344 461.65625 666.507812 C 468.832031 659.339844 477.140625 653.914062 486.007812 650.28125 L 486.007812 383.9375 C 477.140625 380.304688 468.855469 374.933594 461.65625 367.710938 C 439.664062 345.730469 434.375 313.449219 445.652344 286.445312 L 335.683594 176.457031 L 45.265625 466.839844 C 20.871094 491.257812 20.871094 530.8125 45.265625 555.207031 L 468.8125 978.722656 C 493.199219 1003.117188 532.742188 1003.117188 557.160156 978.722656 L 978.710938 557.171875 C 1003.109375 532.777344 1003.109375 493.199219 978.710938 468.800781 Z M 978.710938 468.800781 "/>
                </g>
            </svg>
            <h1>Gnostr Crawler Endpoints</h1>
        </div>
        <p>Explore the available data endpoints for the Gnostr Crawler:</p>
        <ul>
            <li><a href="/relays.txt" target="_blank">/relays.txt</a></li>
            <li><a href="/relays.yaml" target="_blank">/relays.yaml</a></li>
            <li><a href="/relays.json" target="_blank">/relays.json</a></li>
        </ul>
        <footer>
            <p>These endpoints are served by the <code>gnostr-crawler</code> API server.</p>
        </footer>
    </div>
</body>
</html>
"#;

    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "text/html")
        .body(Body::from(html_content.as_ref() as &[u8]))
        .unwrap_or_else(|e| {
            error!("Failed to build HTML response: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Body::from("Internal Server Error")).into_response()
        })
}

pub async fn dispatch_crawler_command(
    command: InnerCrawlerCommand,
    client: &reqwest::Client,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        InnerCrawlerCommand::Sniper { nip, shitlist } => {
            run_sniper(nip, shitlist, client).await?;
        }
        InnerCrawlerCommand::Watch { shitlist } => {
            run_watch(shitlist, client).await?;
        }
        InnerCrawlerCommand::Nip34 { shitlist } => {
            run_nip34(shitlist, client).await?;
        }
        InnerCrawlerCommand::Crawl(args) => {
            run(&args).await?;
        }
        InnerCrawlerCommand::Serve { port } => {
            run_api_server(port).await?;
        }
    }
    Ok(())
}
