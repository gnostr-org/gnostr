use anyhow::Result;
use chrono::TimeZone;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use tokio;
use url::Url;

// build.rs - This file will generate src/relays.yaml

const CACHE_TTL_SECS: u64 = 24 * 60 * 60;

#[derive(Serialize, Deserialize, Debug, Default)]
struct SourceCache {
    hash: String,
    fetched_at: u64,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct CachedHashes {
    sources: HashMap<String, SourceCache>,
}

fn sanitize_relay_entry(line: &str) -> Option<String> {
    let mut trimmed = line.trim().to_string();
    if let Some(stripped) = trimmed.strip_prefix("- ") {
        trimmed = stripped.trim().to_string();
    } else if let Some(stripped) = trimmed.strip_prefix('-') {
        trimmed = stripped.trim().to_string();
    }
    if let Some(comma_idx) = trimmed.find(',') {
        trimmed.truncate(comma_idx);
        trimmed = trimmed.trim().to_string();
    }
    if trimmed.starts_with("wss://") || trimmed.starts_with("ws://") {
        Url::parse(&trimmed).ok().map(|url| url.to_string())
    } else if trimmed.contains("://") {
        None
    } else if trimmed.is_empty() {
        None
    } else {
        let potential = format!("wss://{}", trimmed);
        Url::parse(&potential).ok().map(|url| url.to_string())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    report_build_name();
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let cache_path = out_dir.join("relay_hashes.json");
    let mut cached_hashes: CachedHashes = match fs::read_to_string(&cache_path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => CachedHashes::default(),
    };

    let mut all_relays = HashSet::new();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Define relay sources
    let sources = vec![
        ("bitchat", "https://raw.githubusercontent.com/RandyMcMillan/bitchat/refs/heads/main/relays/online_relays_gps.csv"),
        ("bitchat", "https://raw.githubusercontent.com/permissionlesstech/bitchat/refs/heads/main/relays/online_relays_gps.csv"),
        ("sesseor", "https://raw.githubusercontent.com/sesseor/nostr-relays-list/main/relays.txt"),
    ];

    for (name, url) in sources {
        let cache_fresh = cached_hashes
            .sources
            .get(url)
            .map(|entry| now.saturating_sub(entry.fetched_at) < CACHE_TTL_SECS)
            .unwrap_or(false);

        if cache_fresh {
            eprintln!(
                "Using cached relay list for {} (hash: {}, age: {}s)",
                name,
                cached_hashes.sources[url].hash,
                now.saturating_sub(cached_hashes.sources[url].fetched_at)
            );
            continue;
        }

        eprintln!("Fetching online relays from: {} ({})", name, url);
        match fetch_online_relays_build(url).await {
            Ok((urls, current_hash)) => {
                for url_str in urls {
                    all_relays.insert(url_str);
                }
                cached_hashes.sources.insert(
                    url.to_string(),
                    SourceCache {
                        hash: current_hash,
                        fetched_at: now,
                    },
                );
            }
            Err(e) => {
                eprintln!("Could not fetch online relays from {}: {}", name, e);
                // Keep any existing cached entry so stale-but-present data is not lost.
            }
        }
    }

    let generated_relays_path = out_dir.join("relays.yaml");
    let mut file = File::create(&generated_relays_path)?;

    for relay_url in &all_relays {
        writeln!(file, "{}", relay_url)?;
    }

    // Tell Cargo to rerun only if build.rs itself changes.
    println!("cargo:rerun-if-changed=build.rs");
    println!(
        "cargo:rustc-env=GNOSTR_CRAWLER_RELAYS_YAML={}",
        generated_relays_path.display()
    );

    let serialized = serde_json::to_string_pretty(&cached_hashes)?;
    fs::write(&cache_path, serialized)?;

    Ok(())
}

fn report_build_name() {
    let now = match std::env::var("SOURCE_DATE_EPOCH") {
        Ok(val) => chrono::Local
            .timestamp_opt(val.parse::<i64>().unwrap(), 0)
            .unwrap(),
        Err(_) => chrono::Local::now(),
    };
    let build_date = now.date_naive();
    let build_name = if std::env::var("GITUI_RELEASE").is_ok() {
        format!("{}@{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
    } else {
        format!(
            "{}@{} {} ({})",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            build_date,
            get_git_hash()
        )
    };

    println!("cargo:warning=buildname '{build_name}'");
    println!("cargo:rustc-env=GITUI_BUILD_NAME={build_name}");
}

fn get_git_hash() -> String {
    if let Ok(commit) = std::env::var("BUILD_GIT_COMMIT_ID") {
        return commit[..7].to_string();
    }

    let commit = Command::new("git")
        .arg("rev-parse")
        .arg("--short=7")
        .arg("--verify")
        .arg("HEAD")
        .output();

    if let Ok(commit_output) = commit {
        let commit_string = String::from_utf8_lossy(&commit_output.stdout);
        return commit_string.lines().next().unwrap_or("").into();
    }

    panic!("Can not get git commit: {}", commit.unwrap_err());
}

async fn fetch_online_relays_build(url: &str) -> Result<(Vec<String>, String)> {
    eprintln!("Fetching online relays from: {}", url);
    let client = Client::new();
    let response = client.get(url).send().await?.error_for_status()?;
    let text = response.text().await?;

    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    let current_hash = format!("{:x}", hasher.finalize());

    let relays: Vec<String> = text.lines().filter_map(sanitize_relay_entry).collect();

    eprintln!(
        "Fetched {} online relays from {} (hash: {})",
        relays.len(),
        url,
        current_hash
    );
    Ok((relays, current_hash))
}
