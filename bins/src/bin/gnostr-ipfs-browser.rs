//! Search for a running IPFS (Kubo) WebUI service and view it with the
//! `./tui` terminal browser.
//!
//! Candidate endpoints are taken from `IPFS_WEBUI` / `IPFS_API` and by probing
//! the default Kubo API port range on localhost. The first endpoint that answers
//! successfully is opened in `gnostr_tui::browser`.

use std::time::Duration;

use anyhow::{anyhow, Result};
use reqwest::Client;

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .context("failed to build HTTP client")?;

    let mut candidates: Vec<String> = Vec::new();

    if let Ok(value) = std::env::var("IPFS_WEBUI") {
        let value = value.trim().to_string();
        if !value.is_empty() {
            candidates.push(value);
        }
    }

    if let Ok(value) = std::env::var("IPFS_API") {
        let base = value.trim().trim_end_matches('/').to_string();
        if !base.is_empty() {
            candidates.push(format!("{}/webui", base));
        }
    }

    // Probe the default Kubo API port range.
    for port in 5001u16..=5010 {
        for host in ["127.0.0.1", "localhost"] {
            candidates.push(format!("http://{}:{}/webui", host, port));
        }
    }

    // De-duplicate while preserving order.
    let mut seen = std::collections::HashSet::new();
    candidates.retain(|url| seen.insert(url.clone()));

    for url in candidates {
        match client.get(&url).send().await {
            Ok(response) if response.status().is_success() => {
                println!("Found IPFS WebUI: {}", url);
                return gnostr_tui::browser::run_url(Some(url)).await;
            }
            _ => {}
        }
    }

    Err(anyhow!(
        "No IPFS WebUI service found. Start a Kubo node (default API port 5001) \
         or set IPFS_WEBUI / IPFS_API."
    ))
}

// Keep `anyhow::Context` in scope for the client builder.
use anyhow::Context;
