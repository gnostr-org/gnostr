use crate::processor::BOOTSTRAP_RELAYS;
use log::{debug, trace, warn};
use nostr_sdk::prelude::Url;
use std::collections::HashSet;
use std::fs as sync_fs;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

pub fn preprocess_line(line: &str) -> String {
    let mut trimmed_line = line.trim().to_string();
    if let Some(stripped) = trimmed_line.strip_prefix("- ") {
        trimmed_line = stripped.trim().to_string();
    } else if let Some(stripped) = trimmed_line.strip_prefix('-') {
        trimmed_line = stripped.trim().to_string();
    }
    if let Some(comma_idx) = trimmed_line.find(',') {
        trimmed_line.truncate(comma_idx);
        trimmed_line = trimmed_line.trim().to_string();
    }
    trimmed_line
}

fn relay_host(relay: &str) -> Option<&str> {
    let relay = relay.trim();
    let relay = relay
        .strip_prefix("wss://")
        .or_else(|| relay.strip_prefix("ws://"))?;
    let authority = relay.split('/').next().unwrap_or("");
    let authority = authority.rsplit('@').next().unwrap_or(authority);
    if authority.starts_with('[') {
        Some(authority)
    } else {
        Some(authority.split(':').next().unwrap_or(authority))
    }
}

fn is_private_ipv4_relay(relay: &str) -> bool {
    let host = match relay_host(relay) {
        Some(host) => host,
        None => return false,
    };

    let mut octets = host.split('.');
    let first = match octets.next().and_then(|part| part.parse::<u8>().ok()) {
        Some(first) => first,
        None => return false,
    };
    let second = match octets.next().and_then(|part| part.parse::<u8>().ok()) {
        Some(second) => second,
        None => return false,
    };

    (first == 10)
        || (first == 172 && (16..=31).contains(&second))
        || (first == 192 && second == 168)
        || (first == 100 && (64..=127).contains(&second))
}

fn is_loopback_relay(relay: &str) -> bool {
    relay_host(relay)
        .map(|host| host == "localhost" || host == "127.0.0.1")
        .unwrap_or(false)
}

pub(crate) fn normalize_relay_entry(line: &str) -> Option<String> {
    let mut final_line = preprocess_line(line);
    if final_line.is_empty() {
        return None;
    }

    if !final_line.contains("://") {
        let potential_url = format!("wss://{}", final_line);
        if let Ok(url) = Url::parse(&potential_url) {
            debug!("Prepended 'wss://' to form valid URL: {}", url);
            final_line = url.to_string();
        }
    }

    if !final_line.starts_with("wss://") && !final_line.starts_with("ws://") {
        return None;
    }

    match Url::parse(&final_line) {
        Ok(url) => {
            let normalized = url.to_string();
            if is_loopback_relay(&normalized) {
                return Some(normalized);
            }

            if let Some(host) = relay_host(&normalized) {
                if host.starts_with('-') {
                    warn!("Skipping invalid relay host: {}", normalized);
                    return None;
                }
            }

            if is_private_ipv4_relay(&normalized) {
                warn!("Skipping private relay URL: {}", normalized);
                return None;
            }

            Some(normalized)
        }
        Err(_) => {
            trace!("Skipping invalid WEBSOCKET URL: {}", final_line);
            None
        }
    }
}

pub(crate) fn parse_relay_entries(content: &str) -> Vec<String> {
    if let Ok(values) = serde_yaml::from_str::<Vec<String>>(content) {
        return values
            .into_iter()
            .filter_map(|line| normalize_relay_entry(&line))
            .collect();
    }

    content
        .lines()
        .filter_map(normalize_relay_entry)
        .collect()
}

pub fn load_file(filename: impl AsRef<Path>) -> io::Result<Vec<String>> {
    let base_dir = crate::relays::get_config_dir_path();
    let file_path = base_dir.join(
        filename
            .as_ref()
            .file_name()
            .unwrap_or(filename.as_ref().as_os_str()),
    );

    if let Some(parent) = file_path.parent() {
        sync_fs::create_dir_all(parent)?;
    }

    debug!("load_file: start path={}", file_path.display());

    let file_content = sync_fs::read_to_string(&file_path)?;
    let relays = parse_relay_entries(&file_content);
    debug!(
        "load_file: parsed {} relay entries from {}",
        relays.len(),
        file_path.display()
    );
    Ok(relays)
}

pub fn load_shitlist(filename: impl AsRef<Path>) -> io::Result<HashSet<String>> {
    let path = filename.as_ref().to_path_buf();
    debug!("load_shitlist: start path={}", path.display());
    let entries = BufReader::new(sync_fs::File::open(&path)?)
        .lines()
        .collect::<io::Result<HashSet<String>>>()?;
    debug!(
        "load_shitlist: loaded {} entries from {}",
        entries.len(),
        path.display()
    );
    Ok(entries)
}

pub fn load_relays_or_bootstrap() -> Vec<String> {
    debug!("load_relays_or_bootstrap: start");
    match load_file("relays.yaml") {
        Ok(relays) => relays,
        Err(e) => {
            warn!(
                "Failed to load relays.yaml ({}); falling back to bootstrap relays",
                e
            );
            let relays: Vec<String> = BOOTSTRAP_RELAYS.iter().cloned().collect();
            debug!(
                "load_relays_or_bootstrap: using {} bootstrap relays",
                relays.len()
            );
            relays
        }
    }
}

#[cfg(test)]
mod tests {
    use super::parse_relay_entries;

    #[test]
    fn parses_yaml_relays_without_list_markers() {
        let entries = parse_relay_entries(
            "- wss://relay.example/\n- relay.example\n- wss://relay2.example/\n",
        );

        assert_eq!(
            entries,
            vec![
                "wss://relay.example/".to_string(),
                "wss://relay.example/".to_string(),
                "wss://relay2.example/".to_string(),
            ]
        );
    }

    #[test]
    fn parses_plain_text_relays() {
        let entries = parse_relay_entries("relay.example\nwss://relay2.example/\n");

        assert_eq!(
            entries,
            vec![
                "wss://relay.example/".to_string(),
                "wss://relay2.example/".to_string(),
            ]
        );
    }

    #[test]
    fn rejects_private_relay_entries_but_allows_loopback() {
        let entries = parse_relay_entries(
            "wss://10.0.0.21:4848/\nws://100.71.217.147:4848/\nwss://-auth.nostr1.com/\nwss://localhost:4848/\nws://127.0.0.1:4848/\nwss://relay.example/\n",
        );

        assert_eq!(
            entries,
            vec![
                "wss://localhost:4848/".to_string(),
                "ws://127.0.0.1:4848/".to_string(),
                "wss://relay.example/".to_string(),
            ]
        );
    }
}
