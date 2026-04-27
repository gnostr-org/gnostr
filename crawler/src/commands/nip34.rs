use crate::{load_relays_or_bootstrap, load_shitlist, Relay};
use futures::{stream, StreamExt};
use reqwest::header::ACCEPT;

pub async fn run_nip34(
    shitlist_path: Option<String>,
    client: &reqwest::Client,
) -> Result<(), Box<dyn std::error::Error>> {
    let relays = load_relays_or_bootstrap();

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

    let filtered_relays: Vec<String> = relays
        .into_iter()
        .filter(|url| {
            if shitlist.is_empty() {
                true
            } else {
                !shitlist
                    .iter()
                    .any(|shitlisted_url| url.contains(shitlisted_url))
            }
        })
        .collect();

    let bodies = stream::iter(filtered_relays)
        .map(|url| {
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

                let r: Result<(String, String), reqwest::Error> = Ok((url.clone(), text.clone()));
                r
            }
        })
        .buffer_unordered(crate::CONCURRENT_REQUESTS);

    bodies
        .for_each(|b: Result<(String, String), reqwest::Error>| async {
            if let Ok((url, json_string)) = b {
                let data: Result<Relay, _> = serde_json::from_str(&json_string);
                if let Ok(relay_info) = data {
                    let supported_nips = relay_info.supported_nips.unwrap_or_default();
                    let _supports_nip01 = supported_nips.contains(&1);
                    let _supports_nip11 = supported_nips.contains(&11);
                    let supports_nip34 = supported_nips.contains(&34);

                    if supports_nip34 {
                        println!("{}", url);
                    }
                }
            }
        })
        .await;

    Ok(())
}
