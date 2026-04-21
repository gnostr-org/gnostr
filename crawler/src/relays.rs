use nostr_sdk::prelude::Url;
use directories::ProjectDirs;
use crate::preprocess_line;
use crate::processor::BOOTSTRAP_RELAYS;
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};
use reqwest::Client;
use anyhow::Result;
use reqwest::header::ACCEPT;
use std::sync::{LazyLock, Mutex};

pub fn get_config_dir_path() -> PathBuf {
    ProjectDirs::from("org", "gnostr", "gnostr/crawler")
        .map(|proj_dirs| proj_dirs.config_dir().to_path_buf())
        .unwrap_or_else(|| Path::new(".").to_path_buf())
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

    debug!("write_relays_json_from_yaml: reading {}", yaml_path.display());
    let relays: Vec<String> = match fs::read_to_string(&yaml_path) {
        Ok(content) => content
            .lines()
            .filter_map(sanitize_relay_entry)
            .collect(),
        Err(_) => BOOTSTRAP_RELAYS.clone(),
    };

    let json_content = serde_json::to_string_pretty(&relays)
        .map_err(std::io::Error::other)?;
    debug!("write_relays_json_from_yaml: writing {}", json_path.display());
    fs::write(&json_path, json_content)?;
    Ok(json_path)
}

pub fn write_relays_serve_files() -> std::io::Result<()> {
    let config_dir = get_config_dir_path();
    fs::create_dir_all(&config_dir)?;

    let yaml_source = config_dir.join("relays.yaml");
    debug!("write_relays_serve_files: reading {}", yaml_source.display());
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
    fs::write(&json_path, serde_json::to_string_pretty(&relays).map_err(std::io::Error::other)?)?;
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

    debug!("write_nip_relays_serve_files: writing {}", yaml_path.display());
    let yaml_content = serde_yaml::to_string(relays).map_err(std::io::Error::other)?;
    fs::write(&yaml_path, yaml_content)?;
    debug!("write_nip_relays_serve_files: writing {}", json_path.display());
    fs::write(&json_path, serde_json::to_string_pretty(relays).map_err(std::io::Error::other)?)?;
    debug!("write_nip_relays_serve_files: writing {}", txt_path.display());
    fs::write(&txt_path, relays.join(" "))?;

    Ok(config_dir)
}

pub fn write_nip_relays_serve_files_from_dir(nip: i32) -> std::io::Result<PathBuf> {
    let config_dir = get_config_dir_path().join(nip.to_string());
    fs::create_dir_all(&config_dir)?;

    let mut relays: Vec<String> = Vec::new();
    debug!("write_nip_relays_serve_files_from_dir: reading {}", config_dir.display());
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

    debug!("write_nip_relays_serve_files_from_dir: writing {}", yaml_path.display());
    let yaml_content = serde_yaml::to_string(&relays).map_err(std::io::Error::other)?;
    fs::write(&yaml_path, yaml_content)?;
    debug!("write_nip_relays_serve_files_from_dir: writing {}", json_path.display());
    fs::write(&json_path, serde_json::to_string_pretty(&relays).map_err(std::io::Error::other)?)?;
    debug!("write_nip_relays_serve_files_from_dir: writing {}", txt_path.display());
    fs::write(&txt_path, relays.join(" "))?;

    Ok(config_dir)
}

pub fn write_index_html() -> std::io::Result<PathBuf> {
    let config_dir = get_config_dir_path();
    fs::create_dir_all(&config_dir)?;

    let mut nips = live_nips();
    if let Ok(entries) = fs::read_dir(&config_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                if let Some(name) = entry.file_name().to_str() {
                    if let Ok(nip) = name.parse::<i32>() {
                        nips.push(nip);
                    }
                }
            }
        }
    }
    nips.sort_unstable();
    nips.dedup();
    let mut kinds = live_kinds();
    for kind in kinds_from_disk() {
        if !kinds.contains(&kind) {
            kinds.push(kind);
        }
    }
    kinds.sort();
    kinds.dedup();

    let nip_links = if nips.is_empty() {
        "<li>No NIP buckets yet. Start serve and wait for the sniper service.</li>".to_string()
    } else {
        nips.iter()
            .map(|nip| {
                format!(
                    "<li><a href=\"/{0}\">NIP {0}</a> - <a href=\"/{0}/relays.json\">json</a> <a href=\"/{0}/relays.yaml\">yaml</a> <a href=\"/{0}/relays.txt\">txt</a></li>",
                    nip
                )
            })
            .collect::<Vec<_>>()
            .join("")
    };

    let kind_links = if kinds.is_empty() {
        "<li>No kinds seen yet.</li>".to_string()
    } else {
        kinds
            .iter()
            .map(|kind| format!("<li>{}</li>", kind))
            .collect::<Vec<_>>()
            .join("")
    };

    let nav = [
        ("/", "gnostr crawler"),
        ("/relays.json", "relays.json"),
        ("/relays.yaml", "relays.yaml"),
        ("/relays.txt", "relays.txt"),
    ];
    let body = format!(
        "<section><h2>NIPs</h2><ul>{}</ul></section><section><h2>Kinds</h2><ul>{}</ul></section>",
        nip_links, kind_links
    );
    let html = render_page_shell("gnostr crawler", &nav, &body);

    let path = config_dir.join("index.html");
    fs::write(&path, html)?;
    Ok(path)
}

pub fn render_page_shell(title: &str, nav: &[(&str, &str)], body: &str) -> String {
    let nav_html = nav
        .iter()
        .map(|(href, label)| format!("<a href=\"{}\">{}</a>", href, label))
        .collect::<Vec<_>>()
        .join("<span class=\"nav-sep\">/</span>");

    format!(
        "<!doctype html><html><head><meta charset=\"utf-8\">\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
         <title>{}</title>\
         <style>\
         :root{{color-scheme:dark light;}}\
         body{{font-family:system-ui,-apple-system,BlinkMacSystemFont,\"Segoe UI\",sans-serif;margin:0;line-height:1.5;}}\
         .site-header{{position:sticky;top:0;z-index:10;background:#111;border-bottom:1px solid #333;padding:0.9rem 1rem;}}\
         .site-title{{margin:0;font-size:1.1rem;}}\
         .site-nav{{margin-top:0.35rem;display:flex;flex-wrap:wrap;gap:0.5rem;align-items:center;}}\
         .site-nav a{{color:inherit;text-decoration:none;padding:0.2rem 0.4rem;border-radius:0.35rem;background:rgba(255,255,255,0.06);}}\
         .nav-sep{{opacity:0.4;}}\
         main{{padding:1rem;max-width:1100px;}}\
         section{{margin-bottom:1.5rem;}}\
         ul{{padding-left:1.2rem;}}\
         code{{background:rgba(255,255,255,0.08);padding:0.1rem 0.25rem;border-radius:0.25rem;}}\
         </style></head><body>\
          <header class=\"site-header\"><nav class=\"site-nav\">{}</nav></header>\
          <main>{}</main></body></html>",
         title, nav_html, body
    )
}

pub async fn fetch_online_relays(url: &str) -> Result<Vec<String>> {
    debug!("Fetching online relays from: {}", url);
    let client = Client::new();
    let response = client.get(url).send().await?.error_for_status()?;
    let text = response.text().await?;

    let relays: Vec<String> = text.lines()
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

                                    warn!("Skipping invalid WEBSOCKET URL format: {}", final_line);

                                    None

                                }

                            }

                        } else if final_line.contains(":://") { // It's a URL, but not a websocket URL

                            warn!("Skipping non-websocket URL scheme: {}", final_line);

                            None

                        } else { // It's not a URL at all (e.g., "Relay URL")

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
                warn!("Liveness check failed for {}: Status {}", url_str, response.status());
            }
            is_success
        }
        Err(e) => {
            warn!("Liveness check error for {}: {}", url_str, e);
            false
        }
    }
}

/// Maintain a list of all encountered relays
pub struct Relays {
    r: HashSet<Url>,
}

impl Default for Relays {
    fn default() -> Self {
        Self::new()
    }
}

impl Relays {
    pub fn new() -> Self {
        Self {
            r: HashSet::default(),
        }
    }

    pub fn add(&mut self, s1: &str) -> bool {
        let mut res = false;
        if let Ok(u) = Url::parse(s1) {
            res = self.r.insert(u);
        }
        res
    }

    pub fn count(&self) -> usize {
        self.r.len()
    }

    pub fn de_dup(&self, list: &[Url]) -> Vec<Url> {
        let list: Vec<Url> = list.to_vec();
        for url in &list { debug!("de_dup:: url={}", url); }
        list
    }
    pub fn de_dup_string(&self, list: &[String]) -> Vec<String> {
        let list: Vec<String> = list.to_vec();
        list
    }

    pub fn get_some(&self, max_count: usize) -> Vec<Url> {
        let mut res = Vec::new();
        for u in &self.r {
            res.push(u.clone());
            if res.len() >= max_count {
                return res;
            }
        }
        res = self.de_dup(&res);
        res
    }

    pub fn get_all(&self) -> Vec<String> {
        let list: Vec<String> = self.r.iter().map(|u| u.to_string()).collect();
        self.de_dup_string(&list)
    }

    pub fn print(&self) {
        for u in &self.r {
            let mut relay = format!("{}", u);
            if relay.ends_with('/') {
                relay.pop();
                debug!("relays::125:{}", relay);
            } else {
                debug!("relays::127:{}", relay);
            }
        }
    }

    pub fn dump_list(&self) {
        self.dump_to_file("relays.yaml");
        self.dump_to_json("relays.json");
    }

    pub fn dump_to_file(&self, filename: &str) {
        let config_dir = get_config_dir_path();
        let file_path = config_dir.join(filename);

        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create directory");
        }

        let relays: Vec<String> = self.r.iter().map(|u| u.to_string()).collect();
        match serde_yaml::to_string(&relays) {
            Ok(yaml_content) => {
                let mut file = File::create(&file_path).expect("Failed to create relays.yaml");
                write!(file, "{}", yaml_content).expect("Failed to write YAML content");
                debug!("Relays dumped to {}", file_path.display());
                debug!("Relays.yaml written to: {}", file_path.canonicalize().unwrap_or_default().display());
            },
            Err(e) => {
                warn!("Failed to serialize relays to YAML for {}: {}", filename, e);
            }
        }
    }

    pub fn dump_to_json(&self, filename: &str) {
        let config_dir = get_config_dir_path();
        let file_path = config_dir.join(filename);

        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create directory");
        }

        let mut file = File::create(&file_path).expect("Failed to create relays.yaml");
        debug!("file={:?}", file);

        let mut count = 0;
        let _ = writeln!(file, "[\"RELAYS\",");
        for u in &self.r {
            let _ = writeln!(file, "{{\"{}\":\"{}\"}},", count, u);
            count += 1;
        }
        let _ = writeln!(file, "{{\"{}\":\"wss://relay.gnostr.org\"}}", count);
        let _ = writeln!(file, "]");

        debug!("Relays dumped to {}", file_path.display());
    }
}
