use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use anyhow::Result;
use reqwest::Client;
use tokio;
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// build.rs - This file will generate src/relays.yaml

#[derive(Serialize, Deserialize, Debug, Default)]
struct CachedHashes {
    hashes: HashMap<String, String>,
}

#[tokio::main]
async fn main() -> Result<()> {

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let cache_path = out_dir.join("relay_hashes.json");
    let mut cached_hashes: CachedHashes = match fs::read_to_string(&cache_path) {
        Ok(content) => serde_json::from_str(&content)?,
        Err(_) => CachedHashes::default(),
    };
    let mut new_hashes = HashMap::new();

    let mut all_relays = HashSet::new();

    // Define relay sources
    let sources = vec![
        ("bitchat", "https://raw.githubusercontent.com/RandyMcMillan/bitchat/refs/heads/main/relays/online_relays_gps.csv"),
        ("sesseor", "https://raw.githubusercontent.com/sesseor/nostr-relays-list/main/relays.txt"),
    ];

    for (name, url) in sources {
        let should_fetch = match cached_hashes.hashes.get(url) {
            Some(cached_hash) => {
                // For now, always re-fetch. In a more optimized scenario, we'd fetch headers or a partial file to get a remote hash.
                // For simplicity, we'll assume a change requires a full re-fetch if the hash doesn't match a *previously stored* hash.
                // A more robust solution would involve fetching remote hash without downloading full content.
                eprintln!("Checking hash for {} (cached: {})...", name, cached_hash);
                // For simplicity in this `build.rs`, we'll always fetch for now.
                // A real optimization would involve checking remote hash from headers if available.
                true 
            },
            None => {
                eprintln!("No cached hash for {}. Fetching...", name);
                true
            }
        };

        if should_fetch {
            match fetch_online_relays_build(url).await {
                Ok((urls, current_hash)) => {
                    for url_str in urls {
                        all_relays.insert(url_str);
                    }
                    new_hashes.insert(url.to_string(), current_hash);
                },
                Err(e) => eprintln!("Could not fetch online relays from {}: {}", name, e),
            }
        }
    }

    // Write combined and deduplicated relays to relays.yaml in OUT_DIR
    let generated_relays_path_in_out_dir = out_dir.join("relays.yaml");
    let mut file = File::create(&generated_relays_path_in_out_dir)?;

    for relay_url in &all_relays {
        writeln!(file, "{}", relay_url)?;
    }

    // Tell Cargo the path to the generated file
    println!("cargo:rustc-env=RELAYS_YAML_PATH={}", generated_relays_path_in_out_dir.display());
    // Tell Cargo to rerun if build.rs itself changes
    println!("cargo:rerun-if-changed=build.rs");

    // Write updated hashes to cache file
    cached_hashes.hashes = new_hashes;
    let serialized = serde_json::to_string_pretty(&cached_hashes)?;
    fs::write(&cache_path, serialized)?;

    Ok(())
}

async fn fetch_online_relays_build(url: &str) -> Result<(Vec<String>, String)> {
    eprintln!("Fetching online relays from: {}", url);
    let client = Client::new();
    let response = client.get(url).send().await?.error_for_status()?;
    let text = response.text().await?;

    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    let current_hash = format!("{:x}", hasher.finalize());

    let relays: Vec<String> = text.lines()
        .filter(|line| !line.trim().is_empty())
        .map(String::from)
        .collect();

    eprintln!("Fetched {} online relays from {} (hash: {})", relays.len(), url, current_hash);
    Ok((relays, current_hash))
}
