use crate::{load_shitlist, Relay};
use futures::{stream, StreamExt};
use log::{debug, trace};
use nostr_sdk::prelude::*;
use reqwest::header::ACCEPT;
use crate::commands::run_sniper;

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

    let bodies = stream::iter(relays_iterator)
        .map(|url: String| {
            let client = client.clone();
            async move {
                let resp = client
                    .get(
                        url.replace("wss://", "https://")
                            .replace("ws://", "http://"),
                    )
                    .header(ACCEPT, "application/nostr+json")
                    .send()
                    .await?;
                let text = resp.text().await?;

                Ok((url, text))
            }
        })
        .buffer_unordered(crate::CONCURRENT_REQUESTS);

    bodies
        .for_each(|b: Result<(String, String), reqwest::Error>| async {
            if let Ok((url, json_string)) = b {
                trace!("{{\"relay\":\"{}\", \"data\":{}}}", url, json_string);
                let data: Result<Relay, serde_json::Error> = serde_json::from_str(&json_string);
                if let Ok(relay_info) = data {
                    let supported_nips = relay_info.supported_nips.unwrap_or_default();
                    let mut nip_count = supported_nips.len();
                    for n in &supported_nips {
                        trace!("nip_count:{}", nip_count);
                        if nip_count > 1 {
                            debug!("run_watch::bodies::nip-count > 1 -- {:0>2} ", n);
                            trace!("LINE::581 lib::run_watch");
                            let _ = run_sniper(*n, None, client).await;
                        } else {
                            trace!("{:0>2}", n);
                        }
                        nip_count -= 1;
                    }
                }
            }
        })
        .await;

    Ok(())
}
