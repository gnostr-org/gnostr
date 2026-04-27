use crate::preprocess_line;
use crate::processor::BOOTSTRAP_RELAYS;
use anyhow::Result;
use directories::ProjectDirs;
use nostr_sdk::prelude::Url;
use reqwest::header::ACCEPT;
use reqwest::Client;
use std::collections::HashSet;
use std::fs::{self};
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};
use tracing::{debug, info, warn};

#[path = "relays_set.rs"]
mod relays_set;
#[path = "relays_html.rs"]
mod relays_html;
pub use relays_set::Relays;
pub use relays_html::{render_page_shell, render_page_shell_with_header_right, write_index_html};

pub fn get_config_dir_path() -> PathBuf {
    ProjectDirs::from("org", "gnostr", "gnostr/crawler")
        .map(|proj_dirs| proj_dirs.config_dir().to_path_buf())
        .unwrap_or_else(|| Path::new(".").to_path_buf())
}

pub fn bootstrap_relays() -> Vec<String> {
    BOOTSTRAP_RELAYS.clone()
}

static LIVE_NIPS: LazyLock<Mutex<HashSet<i32>>> = LazyLock::new(|| Mutex::new(HashSet::new()));
static LIVE_KINDS: LazyLock<Mutex<HashSet<String>>> = LazyLock::new(|| Mutex::new(HashSet::new()));

pub fn record_live_nips(nips: impl IntoIterator<Item = i32>) {
    let mut live = LIVE_NIPS.lock().unwrap();
    let mut changed = false;
    for nip in nips {
        changed |= live.insert(nip);
    }
    drop(live);
    if changed {
        let _ = write_index_html();
    }
}

pub fn record_live_kind(kind: impl Into<String>) {
    let changed = LIVE_KINDS.lock().unwrap().insert(kind.into());
    if changed {
        let _ = write_kinds_serve_files();
        let _ = write_index_html();
    }
}

pub fn live_nips() -> Vec<i32> {
    let mut nips: Vec<i32> = LIVE_NIPS.lock().unwrap().iter().copied().collect();
    nips.sort_unstable();
    nips
}

pub fn live_kinds() -> Vec<String> {
    let mut kinds: Vec<String> = LIVE_KINDS.lock().unwrap().iter().cloned().collect();
    kinds.sort();
    kinds
}

fn kinds_from_disk() -> Vec<String> {
    let config_dir = get_config_dir_path();
    let kinds_path = config_dir.join("kinds.txt");

    match fs::read_to_string(&kinds_path) {
        Ok(content) => content
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(String::from)
            .collect(),
        Err(_) => Vec::new(),
    }
}

pub fn prime_live_kinds_from_disk() {
    let kinds = kinds_from_disk();
    if kinds.is_empty() {
        return;
    }

    let mut live = LIVE_KINDS.lock().unwrap();
    for kind in kinds {
        live.insert(kind);
    }
}

pub fn write_kinds_serve_files() -> std::io::Result<PathBuf> {
    let config_dir = get_config_dir_path();
    fs::create_dir_all(&config_dir)?;

    let mut kinds = live_kinds();
    for kind in kinds_from_disk() {
        if !kinds.contains(&kind) {
            kinds.push(kind);
        }
    }
    kinds.sort();
    kinds.dedup();

    let txt_path = config_dir.join("kinds.txt");
    let json_path = config_dir.join("kinds.json");

    fs::write(&txt_path, kinds.join("\n"))?;
    fs::write(
        &json_path,
        serde_json::to_string_pretty(&kinds).map_err(std::io::Error::other)?,
    )?;

    Ok(txt_path)
}

fn sanitize_relay_entry(line: &str) -> Option<String> {
    let mut final_line = crate::preprocess_line(line);
    if final_line.is_empty() {
        return None;
    }

    if !final_line.contains("://") {
        let potential_url = format!("wss://{}", final_line);
        if let Ok(url) = Url::parse(&potential_url) {
            final_line = url.to_string();
        }
    }

    if final_line.starts_with("wss://") || final_line.starts_with("ws://") {
        Url::parse(&final_line).ok().map(|url| url.to_string())
    } else {
        None
    }
}

pub fn write_relays_json_from_yaml() -> std::io::Result<PathBuf> {
    let config_dir = get_config_dir_path();
    let yaml_path = config_dir.join("relays.yaml");
    let json_path = config_dir.join("relays.json");

    if let Some(parent) = json_path.parent() {
        fs::create_dir_all(parent)?;
    }

    debug!(
        "write_relays_json_from_yaml: reading {}",
        yaml_path.display()
    );
    let relays: Vec<String> = match fs::read_to_string(&yaml_path) {
        Ok(content) => content.lines().filter_map(sanitize_relay_entry).collect(),
        Err(_) => BOOTSTRAP_RELAYS.clone(),
    };

    let json_content = serde_json::to_string_pretty(&relays).map_err(std::io::Error::other)?;
    debug!(
        "write_relays_json_from_yaml: writing {}",
        json_path.display()
    );
    fs::write(&json_path, json_content)?;
    Ok(json_path)
}

pub fn write_relays_serve_files() -> std::io::Result<()> {
    let config_dir = get_config_dir_path();
    fs::create_dir_all(&config_dir)?;

    let yaml_source = config_dir.join("relays.yaml");
    debug!(
        "write_relays_serve_files: reading {}",
        yaml_source.display()
    );
    let relays: Vec<String> = match fs::read_to_string(&yaml_source) {
        Ok(content) => content.lines().filter_map(sanitize_relay_entry).collect(),
        Err(_) => BOOTSTRAP_RELAYS.clone(),
    };

    let yaml_path = config_dir.join("relays.yaml");
    let json_path = config_dir.join("relays.json");
    let txt_path = config_dir.join("relays.txt");

    let yaml_content = serde_yaml::to_string(&relays).map_err(std::io::Error::other)?;
    debug!("write_relays_serve_files: writing {}", yaml_path.display());
    fs::write(&yaml_path, yaml_content)?;
    debug!("write_relays_serve_files: writing {}", json_path.display());
    fs::write(
        &json_path,
        serde_json::to_string_pretty(&relays).map_err(std::io::Error::other)?,
    )?;
    debug!("write_relays_serve_files: writing {}", txt_path.display());
    fs::write(&txt_path, relays.join(" "))?;
    Ok(())
}

pub fn write_nip_relays_serve_files(nip: i32, relays: &[String]) -> std::io::Result<PathBuf> {
    let config_dir = get_config_dir_path().join(nip.to_string());
    fs::create_dir_all(&config_dir)?;

    let yaml_path = config_dir.join("relays.yaml");
    let json_path = config_dir.join("relays.json");
    let txt_path = config_dir.join("relays.txt");

    debug!(
        "write_nip_relays_serve_files: writing {}",
        yaml_path.display()
    );
    let yaml_content = serde_yaml::to_string(relays).map_err(std::io::Error::other)?;
    fs::write(&yaml_path, yaml_content)?;
    debug!(
        "write_nip_relays_serve_files: writing {}",
        json_path.display()
    );
    fs::write(
        &json_path,
        serde_json::to_string_pretty(relays).map_err(std::io::Error::other)?,
    )?;
    debug!(
        "write_nip_relays_serve_files: writing {}",
        txt_path.display()
    );
    fs::write(&txt_path, relays.join(" "))?;

    Ok(config_dir)
}

pub fn write_nip_relays_serve_files_from_dir(nip: i32) -> std::io::Result<PathBuf> {
    let config_dir = get_config_dir_path().join(nip.to_string());
    fs::create_dir_all(&config_dir)?;

    let mut relays: Vec<String> = Vec::new();
    debug!(
        "write_nip_relays_serve_files_from_dir: reading {}",
        config_dir.display()
    );
    for entry in fs::read_dir(&config_dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.ends_with(".json") || name == "relays.json" {
            continue;
        }
        if let Some(host) = name.strip_suffix(".json") {
            if let Ok(url) = Url::parse(&format!("wss://{}", host)) {
                info!(
                    "write_nip_relays_serve_files_from_dir: including {} from {}",
                    url,
                    entry.path().display()
                );
                relays.push(url.to_string());
            } else {
                info!(
                    "write_nip_relays_serve_files_from_dir: skipping invalid host file {}",
                    entry.path().display()
                );
            }
        }
    }
    relays.sort();
    relays.dedup();
    info!(
        "write_nip_relays_serve_files_from_dir: built {} relay entries for NIP {}",
        relays.len(),
        nip
    );

    let yaml_path = config_dir.join("relays.yaml");
    let json_path = config_dir.join("relays.json");
    let txt_path = config_dir.join("relays.txt");

    debug!(
        "write_nip_relays_serve_files_from_dir: writing {}",
        yaml_path.display()
    );
    let yaml_content = serde_yaml::to_string(&relays).map_err(std::io::Error::other)?;
    fs::write(&yaml_path, yaml_content)?;
    debug!(
        "write_nip_relays_serve_files_from_dir: writing {}",
        json_path.display()
    );
    fs::write(
        &json_path,
        serde_json::to_string_pretty(&relays).map_err(std::io::Error::other)?,
    )?;
    debug!(
        "write_nip_relays_serve_files_from_dir: writing {}",
        txt_path.display()
    );
    fs::write(&txt_path, relays.join(" "))?;

    Ok(config_dir)
}

pub async fn fetch_online_relays(url: &str) -> Result<Vec<String>> {
    debug!("Fetching online relays from: {}", url);
    let client = Client::new();
    let response = client.get(url).send().await?.error_for_status()?;
    let text = response.text().await?;

    let relays: Vec<String> = text
        .lines()
        .filter_map(|line| {
            let preprocessed_line = preprocess_line(line);

            if preprocessed_line.is_empty() {
                return None;
            }

            let mut final_line = preprocessed_line;

            // Attempt to prepend wss:// if it looks like a hostname without a scheme

            if !final_line.contains("://") {
                let potential_url = format!("wss://{}", final_line);

                match Url::parse(&potential_url) {
                    Ok(url) => {
                        debug!("Prepended 'wss://' to form valid URL: {}", url);

                        final_line = url.to_string();
                    }

                    Err(_) => {
                        // If prepending wss:// doesn't form a valid URL, keep the original line

                        // and let the next checks handle it as a non-URL line.

                        debug!(
                            "Attempted to prepend 'wss://' but it's still not a valid URL: {}",
                            potential_url
                        );
                    }
                }
            }

            if final_line.starts_with("wss://") || final_line.starts_with("ws://") {
                match Url::parse(&final_line) {
                    Ok(url) => Some(url.to_string()),

                    Err(_) => {
                        warn!("Skipping invalid WEBSOCKET URL format: {}", final_line);

                        None
                    }
                }
            } else if final_line.contains(":://") {
                // It's a URL, but not a websocket URL

                warn!("Skipping non-websocket URL scheme: {}", final_line);

                None
            } else {
                // It's not a URL at all (e.g., "Relay URL")

                debug!("Silently skipping non-URL line: {}", final_line);

                None
            }
        })
        .collect();

    debug!("Fetched {} online relays", relays.len());
    Ok(relays)
}

pub async fn check_relay_liveness(url_str: &str) -> bool {
    let client = Client::new();
    let http_url = url_str
        .replace("wss://", "https://")
        .replace("ws://", "http://");

    match client
        .head(&http_url)
        .header(ACCEPT, "application/nostr+json")
        .timeout(std::time::Duration::from_secs(5)) // 5 second timeout
        .send()
        .await
    {
        Ok(response) => {
            let is_success = response.status().is_success();
            if !is_success {
                warn!(
                    "Liveness check failed for {}: Status {}",
                    url_str,
                    response.status()
                );
            }
            is_success
        }
        Err(e) => {
            warn!("Liveness check error for {}: {}", url_str, e);
            false
        }
    }
}
