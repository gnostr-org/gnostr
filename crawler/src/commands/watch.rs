use crate::{fetch_relay_texts, load_shitlist, parse_relay_metadata, Relay};
use log::{debug, info, trace};
use nostr_sdk::prelude::*;
use std::collections::HashSet;

pub async fn run_watch(
    shitlist_path: Option<String>,
    client: &reqwest::Client,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("lib::run_watch");
    let app_secret_key = SecretKey::from_bech32(crate::processor::APP_SECRET_KEY)?;
    let app_keys = Keys::new(app_secret_key);
    let processor = crate::processor::Processor::new();
    let mut relay_manager = crate::relay_manager::RelayManager::new(app_keys, processor).await;

    let bootstrap_relays: Vec<&str> = crate::processor::BOOTSTRAP_RELAYS
        .iter()
        .map(|s| s.as_str())
        .collect();
    relay_manager.run(bootstrap_relays).await?;
    let relays: Vec<String> = relay_manager.relays.get_all();

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

    let relays_iterator = relays.into_iter().filter(|url: &String| {
        if shitlist.is_empty() {
            true
        } else {
            !shitlist
                .iter()
                .any(|shitlisted_url| url.contains(shitlisted_url))
        }
    });

    let bodies = fetch_relay_texts(relays_iterator.collect(), client, "run_watch").await;
    let mut discovered_nips = HashSet::new();

    for b in bodies {
        if let Ok((url, json_string, ping_ms)) = b {
            trace!("{{\"relay\":\"{}\", \"data\":{}}}", url, json_string);
            let data: Result<Relay, serde_json::Error> = parse_relay_metadata(&json_string);
            if let Ok(relay_info) = data {
                let mut relay_info = relay_info;
                relay_info.ping_ms = Some(ping_ms);
                let supported_nips = relay_info.supported_nips.clone().unwrap_or_default();
                for n in &supported_nips {
                    trace!("run_watch: discovered NIP {:0>2} on {}", n, url);
                    discovered_nips.insert(*n);
                    if let Err(e) =
                        crate::commands::sniper::write_nip_relay_metadata(*n, &url, &relay_info)
                    {
                        debug!(
                            "run_watch: failed to persist NIP-{} metadata for {}: {}",
                            n, url, e
                        );
                    }
                }
            }
        }
    }

    for nip in discovered_nips {
        info!("run_watch: refreshing NIP {} bucket", nip);
        crate::commands::sniper::refresh_nip_bucket(nip);
    }

    Ok(())
}
