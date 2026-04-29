use crate::{fetch_relay_texts, load_relays_or_bootstrap, load_shitlist, parse_relay_metadata};

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

    let bodies = fetch_relay_texts(filtered_relays, client, "run_nip34").await;

    for b in bodies {
        if let Ok((url, json_string, _ping_ms)) = b {
            let data = parse_relay_metadata(&json_string);
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
    }

    Ok(())
}
