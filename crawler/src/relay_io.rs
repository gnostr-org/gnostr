use crate::processor::BOOTSTRAP_RELAYS;
use log::{debug, warn};
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
    let preprocessed_lines: Vec<String> = file_content
        .lines()
        .map(preprocess_line)
        .filter(|line| !line.is_empty())
        .collect();

    let preprocessed_content_for_yaml = preprocessed_lines.join("\n");
    let relays: Vec<String> =
        match serde_yaml::from_str::<Vec<String>>(&preprocessed_content_for_yaml) {
            Ok(yaml_relays) => yaml_relays,
            Err(e) => {
                warn!(
                    "Failed to parse {} as YAML: {}. Falling back to preprocessed lines.",
                    file_path.display(),
                    e
                );
                preprocessed_lines
            }
        };
    debug!(
        "load_file: parsed {} relay entries from {}",
        relays.len(),
        file_path.display()
    );

    let filtered_relays: Vec<String> = relays
        .into_iter()
        .filter_map(|line| {
            if line.is_empty() {
                return None;
            }

            let mut final_line = line.clone();

            if !final_line.contains("://") {
                let potential_url = format!("wss://{}", final_line);
                match Url::parse(&potential_url) {
                    Ok(url) => {
                        debug!("Prepended 'wss://' to form valid URL: {}", url);
                        final_line = url.to_string();
                    }
                    Err(_) => {
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
                        warn!(
                            "Skipping invalid WEBSOCKET URL in {}: {}",
                            filename.as_ref().display(),
                            final_line
                        );
                        None
                    }
                }
            } else if final_line.contains("://") {
                warn!(
                    "Skipping non-websocket URL scheme in {}: {}",
                    filename.as_ref().display(),
                    final_line
                );
                None
            } else {
                debug!(
                    "Silently skipping non-URL line in {}: {}",
                    filename.as_ref().display(),
                    final_line
                );
                None
            }
        })
        .collect();

    debug!(
        "load_file: returning {} filtered relay entries from {}",
        filtered_relays.len(),
        file_path.display()
    );
    Ok(filtered_relays)
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
