use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use anyhow::Result;
use reqwest::Client;
use tokio;

// build.rs - This file will generate src/relays.yaml

#[tokio::main]
async fn main() -> Result<()> {
    // Tell Cargo that if the output of this build script changes, then the project must be recompiled.
    // This ensures that changes to fetched relay lists trigger a rebuild.
    println!("cargo:rerun-if-changed=src/relays.yaml");

    let mut all_relays = HashSet::new();

    // Fetch online relays (RandyMcMillan/bitchat)
    let bitchat_online_relays_url = "https://raw.githubusercontent.com/RandyMcMillan/bitchat/refs/heads/main/relays/online_relays_gps.csv";
    match fetch_online_relays_build(bitchat_online_relays_url).await {
        Ok(urls) => {
            for url_str in urls {
                all_relays.insert(url_str);
            }
        },
        Err(e) => eprintln!("Could not fetch online relays from bitchat: {}", e),
    }

    // Fetch online relays (sesseor/nostr-relays-list)
    let sesseor_online_relays_url = "https://raw.githubusercontent.com/sesseor/nostr-relays-list/main/relays.txt";
    match fetch_online_relays_build(sesseor_online_relays_url).await {
        Ok(urls) => {
            for url_str in urls {
                all_relays.insert(url_str);
            }
        },
        Err(e) => eprintln!("Could not fetch online relays from sesseor: {}", e),
    }

    // Write combined and deduplicated relays to src/relays.yaml
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = std::path::Path::new(&out_dir).join("relays.yaml");
    let mut file = File::create(&dest_path)?;

    for relay_url in all_relays {
        writeln!(file, "{}", relay_url)?;
    }

    // Tell Cargo to link the generated file
    println!("cargo:rerun-if-changed={}", dest_path.display());

    Ok(())
}

async fn fetch_online_relays_build(url: &str) -> Result<Vec<String>> {
    eprintln!("Fetching online relays from: {}", url);
    let client = Client::new();
    let response = client.get(url).send().await?.error_for_status()?;
    let text = response.text().await?;

    let relays: Vec<String> = text.lines()
        .filter(|line| !line.trim().is_empty())
        .map(String::from)
        .collect();

    eprintln!("Fetched {} online relays from {}", relays.len(), url);
    Ok(relays)
}