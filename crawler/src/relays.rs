use crate::relay_io::normalize_relay_entry;
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

fn collect_relays_from_content(path: &Path, content: &str, relays: &mut Vec<String>) {
    let mut record_relay = |relay: String| {
        info!(
            "write_relays_serve_files: including relay={} from {}",
            relay,
            path.display()
        );
        relays.push(relay);
    };

    match path.extension().and_then(|ext| ext.to_str()) {
        Some("json") => {
            if let Ok(values) = serde_json::from_str::<Vec<String>>(content) {
                for value in values {
                    if let Some(relay) = normalize_relay_entry(&value) {
                        record_relay(relay);
                    }
                }
                return;
            }
        }
        Some("yaml") | Some("yml") => {
            if let Ok(values) = serde_yaml::from_str::<Vec<String>>(content) {
                for value in values {
                    if let Some(relay) = normalize_relay_entry(&value) {
                        record_relay(relay);
                    }
                }
                return;
            }
        }
        Some("txt") => {
            for value in content.split_whitespace() {
                if let Some(relay) = normalize_relay_entry(value) {
                    record_relay(relay);
                }
            }
            return;
        }
        _ => {}
    }

    for line in content.lines() {
        if let Some(relay) = normalize_relay_entry(line) {
            record_relay(relay);
        }
    }
}

fn collect_relays_from_bucket_tree(root: &Path, relays: &mut Vec<String>) -> std::io::Result<()> {
    if !root.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if path.is_dir() {
            info!(
                "write_relays_serve_files: scanning bucket directory {}",
                path.display()
            );
            collect_relays_from_bucket_tree(&path, relays)?;
            continue;
        }

        if path.parent() == Some(root) && matches!(name.as_str(), "relays.json" | "relays.txt") {
            debug!(
                "write_relays_serve_files: skipping root aggregate file {}",
                path.display()
            );
            continue;
        }

        if matches!(name.as_str(), "relays.yaml" | "relays.json" | "relays.txt") {
            match fs::read_to_string(&path) {
                Ok(content) => {
                    info!(
                        "write_relays_serve_files: reading relay bucket file {}",
                        path.display()
                    );
                    collect_relays_from_content(&path, &content, relays);
                }
                Err(e) => {
                    warn!(
                        "write_relays_serve_files: failed to read {}: {}",
                        path.display(),
                        e
                    );
                }
            }
        }
    }

    Ok(())
}

fn write_bucket_serve_files(bucket_name: &str, relays: &[String]) -> std::io::Result<PathBuf> {
    let config_dir = get_config_dir_path().join(bucket_name);
    fs::create_dir_all(&config_dir)?;

    let yaml_path = config_dir.join("relays.yaml");
    let json_path = config_dir.join("relays.json");
    let txt_path = config_dir.join("relays.txt");

    debug!(
        "write_bucket_serve_files: writing {}",
        yaml_path.display()
    );
    let yaml_content = serde_yaml::to_string(relays).map_err(std::io::Error::other)?;
    fs::write(&yaml_path, yaml_content)?;
    debug!(
        "write_bucket_serve_files: writing {}",
        json_path.display()
    );
    fs::write(
        &json_path,
        serde_json::to_string_pretty(relays).map_err(std::io::Error::other)?,
    )?;
    debug!("write_bucket_serve_files: writing {}", txt_path.display());
    fs::write(&txt_path, relays.join(" "))?;

    Ok(config_dir)
}

pub fn append_recent_relay(relay: &str) -> std::io::Result<PathBuf> {
    let config_dir = get_config_dir_path().join("recent");
    fs::create_dir_all(&config_dir)?;
    let txt_path = config_dir.join("relays.txt");

    let mut relays: Vec<String> = match fs::read_to_string(&txt_path) {
        Ok(content) => content
            .split_whitespace()
            .filter_map(|relay| Url::parse(relay).ok().map(|url| url.to_string()))
            .collect(),
        Err(_) => Vec::new(),
    };

    let relay = match Url::parse(relay) {
        Ok(url) => url.to_string(),
        Err(_) => {
            debug!("append_recent_relay: skipping invalid relay={relay}");
            return Ok(config_dir);
        }
    };

    if !relays.iter().any(|existing| existing == &relay) {
        debug!("append_recent_relay: appending relay={} bucket=recent", relay);
        relays.push(relay);
        relays.sort();
        relays.dedup();
        write_bucket_serve_files("recent", &relays)?;
        let _ = write_relays_serve_files();
    } else {
        debug!("append_recent_relay: relay already present bucket=recent relay={relay}");
    }

    Ok(config_dir)
}

pub fn write_relays_json_from_yaml() -> std::io::Result<PathBuf> {
    write_relays_serve_files()?;
    Ok(get_config_dir_path().join("relays.json"))
}

pub fn write_relays_serve_files() -> std::io::Result<()> {
    let config_dir = get_config_dir_path();
    fs::create_dir_all(&config_dir)?;

    let mut relays: Vec<String> = Vec::new();
    collect_relays_from_bucket_tree(&config_dir, &mut relays)?;
    if relays.is_empty() {
        info!(
            "write_relays_serve_files: no bucket relays found, falling back to bootstrap relays"
        );
        relays.extend(BOOTSTRAP_RELAYS.clone());
    }
    relays.sort();
    relays.dedup();
    info!(
        "write_relays_serve_files: built {} aggregated relay entries",
        relays.len()
    );
    for relay in &relays {
        info!("write_relays_serve_files: root relay={relay}");
    }

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
    let _ = write_relays_serve_files();

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
    let _ = write_relays_serve_files();

    Ok(config_dir)
}

pub async fn fetch_online_relays(url: &str) -> Result<Vec<String>> {
    debug!("Fetching online relays from: {}", url);
    let client = Client::new();
    let response = client.get(url).send().await?.error_for_status()?;
    let text = response.text().await?;

    let relays: Vec<String> = text
        .lines()
        .filter_map(normalize_relay_entry)
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
