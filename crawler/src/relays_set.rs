use log::{debug, warn};
use nostr_sdk::prelude::Url;
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Write;

use crate::relay_io::normalize_relay_entry;
use crate::relays::get_config_dir_path;

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
        match normalize_relay_entry(s1) {
            Some(normalized) => match Url::parse(&normalized) {
                Ok(u) => self.r.insert(u),
                Err(e) => {
                    warn!("Skipping invalid relay URL {}: {}", normalized, e);
                    false
                }
            },
            None => false,
        }
    }

    pub fn count(&self) -> usize {
        self.r.len()
    }

    pub fn de_dup(&self, list: &[Url]) -> Vec<Url> {
        let list: Vec<Url> = list.to_vec();
        for url in &list {
            debug!("de_dup:: url={}", url);
        }
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
                debug!(
                    "Relays.yaml written to: {}",
                    file_path.canonicalize().unwrap_or_default().display()
                );
            }
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

#[cfg(test)]
mod tests {
    use super::Relays;

    #[test]
    fn add_rejects_private_relays_and_allows_loopback() {
        let mut relays = Relays::new();

        assert!(!relays.add("wss://10.0.0.21:4848/"));
        assert!(!relays.add("ws://100.71.217.147:4848/"));
        assert!(relays.add("wss://localhost:4848/"));
        assert!(relays.add("ws://127.0.0.1:4848/"));
        assert!(relays.add("wss://relay.example/"));
    }
}
