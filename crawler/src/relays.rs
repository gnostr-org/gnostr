use nostr_sdk::prelude::Url;
use directories::ProjectDirs;
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use tracing::{debug, warn};
use reqwest::Client;
use anyhow::Result;
use reqwest::header::ACCEPT;

pub fn get_config_dir_path() -> PathBuf {
    ProjectDirs::from("org", "gnostr", "gnostr/crawler")
        .map(|proj_dirs| proj_dirs.config_dir().to_path_buf())
        .unwrap_or_else(|| Path::new(".").to_path_buf())
}

pub async fn fetch_online_relays(url: &str) -> Result<Vec<String>> {
    debug!("Fetching online relays from: {}", url);
    let client = Client::new();
    let response = client.get(url).send().await?.error_for_status()?;
    let text = response.text().await?;

    let relays: Vec<String> = text.lines()
        .filter(|line| !line.trim().is_empty())
        .map(String::from)
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
            if res {
                //self.print();
            }
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

        let mut file = File::create(&file_path).expect("Failed to create relays.yaml");
        for u in &self.r {
            writeln!(file, "{}", u).expect("Failed to write relay URL");
        }
        debug!("Relays dumped to {}", file_path.display());
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
