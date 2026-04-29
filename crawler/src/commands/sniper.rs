use crate::{fetch_relay_texts, load_relays_or_bootstrap, load_shitlist, parse_relay_metadata};
use log::{error, info, trace};
use nostr_sdk::prelude::Url;
use std::fs as sync_fs;
use std::io::Write;

pub(crate) fn write_nip_relay_metadata(
    nip_lower: i32,
    url: &str,
    json_string: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let parsed_url = Url::parse(url)?;
    let host = parsed_url.host_str().unwrap_or("unknown");
    let dir_path = crate::relays::get_config_dir_path().join(nip_lower.to_string());
    sync_fs::create_dir_all(&dir_path)?;

    let file_path = dir_path.join(format!("{}.json", host));
    let file_path_str = file_path.display().to_string();

    info!("write_nip_relay_metadata: writing {}", file_path_str);
    let mut file = sync_fs::File::create(&file_path)?;
    file.write_all(json_string.as_bytes())?;
    Ok(())
}

pub(crate) fn refresh_nip_bucket(nip_lower: i32) {
    crate::relays::record_live_nips(std::iter::once(nip_lower));
    if let Err(e) = crate::relays::write_nip_relays_serve_files_from_dir(nip_lower) {
        error!("Failed to refresh nip {} relay files: {}", nip_lower, e);
    }
}

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

    let bodies = fetch_relay_texts(filtered_relays, client, "run_sniper").await;

    for b in bodies {
        if let Ok((url, json_string)) = b {
            let data = parse_relay_metadata(&json_string);
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

                            if let Err(e) = write_nip_relay_metadata(nip_lower, &url, &json_string)
                            {
                                error!(
                                    "Failed to write NIP-{} metadata for {}: {}",
                                    nip_lower, url, e
                                );
                                continue;
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
                            trace!("run_sniper: Relay {} does not support NIP-{}", url, nip_lower);
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
    }

    refresh_nip_bucket(nip_lower);

    Ok(())
}
