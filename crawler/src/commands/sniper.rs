use crate::{load_relays_or_bootstrap, load_shitlist, Relay, CONCURRENT_REQUESTS};
use futures::{stream, StreamExt};
use log::{error, info, trace, warn};
use nostr_sdk::prelude::Url;
use reqwest::header::ACCEPT;
use std::fs as sync_fs;
use std::io::Write;

pub async fn run_sniper(
    nip_lower: i32,
    shitlist_path: Option<String>,
    client: &reqwest::Client,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("lib::run_sniper");

    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    info!("run_sniper: Finished initial sleep.");

    let relays = load_relays_or_bootstrap();
    info!(
        "run_sniper: Loaded {} relays from relays.yaml.",
        relays.len()
    );

    let shitlist = if let Some(path) = shitlist_path {
        match load_shitlist(&path) {
            Ok(sl) => sl,
            Err(e) => {
                eprintln!("Failed to load shitlist from {}: {}", path, e);
                return Err(e.into());
            }
        }
    } else {
        std::collections::HashSet::new()
    };
    info!(
        "run_sniper: Shitlist loaded. Contains {} entries.",
        shitlist.len()
    );

    let initial_relay_count = relays.len();
    let filtered_relays: Vec<String> = relays
        .into_iter()
        .filter(|url| {
            if shitlist.is_empty() {
                true
            } else {
                let is_shitlisted = shitlist
                    .iter()
                    .any(|shitlisted_url| url.contains(shitlisted_url));
                if is_shitlisted {
                    info!("run_sniper: Filtering out shitlisted relay: {}", url);
                }
                !is_shitlisted
            }
        })
        .collect();
    info!(
        "run_sniper: Filtered from {} to {} relays.",
        initial_relay_count,
        filtered_relays.len()
    );

    let bodies = stream::iter(filtered_relays)
        .map(|url| {
            info!("run_sniper: Processing URL: {}", url);
            let client = client.clone();
            async move {
                let http_url = url
                    .replace("wss://", "https://")
                    .replace("ws://", "http://");
                info!("run_sniper: Sending request to: {}", http_url);
                let resp = client
                    .get(&http_url)
                    .header(ACCEPT, "application/nostr+json")
                    .send()
                    .await?;

                if !resp.status().is_success() {
                    warn!(
                        "run_sniper: Failed to fetch NIP-11 document for {}: HTTP Status {}",
                        url,
                        resp.status()
                    );
                    return Ok((url, String::new()));
                }

                info!("run_sniper: Received response status: {:?}", resp.status());
                let text = resp.text().await?;
                info!("run_sniper: Raw response text from {}: {}", http_url, text);

                let r: Result<(String, String), reqwest::Error> = Ok((url.clone(), text.clone()));
                r
            }
        })
        .buffer_unordered(CONCURRENT_REQUESTS);

    bodies
        .for_each(|b: Result<(String, String), reqwest::Error>| async {
            if let Ok((url, json_string)) = b {
                let data: Result<Relay, _> = serde_json::from_str(&json_string);
                match data {
                    Ok(relay_info) => {
                        info!("run_sniper: Successfully parsed relay info for {}", url);
                        for n in &relay_info.supported_nips.unwrap_or_default() {
                            if n == &nip_lower {
                                info!(
                                    "run_sniper: Found NIP-{} support on relay: {}",
                                    nip_lower, url
                                );
                                info!("contact:{:?}", &relay_info.contact);
                                info!("description:{:?}", &relay_info.description);
                                info!("name:{:?}", &relay_info.name);
                                info!("software:{:?}", &relay_info.software);
                                info!("version:{:?}", &relay_info.version);

                                let parsed_url = match Url::parse(&url) {
                                    Ok(u) => u,
                                    Err(e) => {
                                        error!("Failed to parse URL {}: {}", url, e);
                                        return;
                                    }
                                };
                                let host = parsed_url.host_str().unwrap_or("unknown");
                                info!("run_sniper: Host for {} is {}", url, host);

                                let dir_path = crate::relays::get_config_dir_path()
                                    .join(format!("{}", nip_lower));
                                if let Err(e) = sync_fs::create_dir_all(&dir_path) {
                                    error!(
                                        "Failed to create directory {}: {}",
                                        dir_path.display(),
                                        e
                                    );
                                    return;
                                }
                                info!(
                                    "run_sniper: Ensured directory exists: {}",
                                    dir_path.display()
                                );

                                let file_name = format!("{}.json", host);
                                let file_path = dir_path.join(&file_name);
                                let file_path_str = file_path.display().to_string();
                                info!(
                                    "run_sniper: Attempting to write to file: {}\n\n{}",
                                    file_path_str, file_path_str
                                );

                                match sync_fs::File::create(&file_path) {
                                    Ok(mut file) => {
                                        info!("run_sniper: File created: {}", &file_path_str);
                                        match file.write_all(json_string.as_bytes()) {
                                            Ok(_) => info!(
                                                "run_sniper: Wrote relay metadata to: {}",
                                                &file_path_str
                                            ),
                                            Err(e) => {
                                                error!(
                                                    "Failed to write to {}: {}",
                                                    &file_path_str, e
                                                )
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        error!("Failed to create file {}: {}", &file_path_str, e)
                                    }
                                }

                                info!(
                                    "run_sniper: Processed NIP {} for relay: {}/{}",
                                    nip_lower,
                                    nip_lower,
                                    url.replace("https://", "")
                                        .replace("wss://", "")
                                        .replace("ws://", "")
                                );
                            } else {
                                trace!(
                                    "run_sniper: Relay {} does not support NIP-{}",
                                    url,
                                    nip_lower
                                );
                            }
                        }
                    }
                    Err(e) => {
                        error!(
                            "run_sniper: Failed to parse JSON for {}: {}. JSON: {}",
                            url, e, json_string
                        );
                    }
                }
            } else if let Err(e) = b {
                error!("run_sniper: Error fetching relay data: {}", e);
            }
        })
        .await;

    Ok(())
}
