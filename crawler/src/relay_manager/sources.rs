use crate::load_file;
use crate::processor::SHITLIST_RELAYS;
use crate::relays::{fetch_online_relays, Relays};
use log::debug;

pub async fn load_relay_sources() -> Relays {
    let mut relays = Relays::new();

    match load_file("relays.yaml") {
        Ok(urls) => {
            for url_str in urls {
                relays.add(&url_str);
            }
            debug!("Loaded {} relays from relays.yaml", relays.count());
        }
        Err(e) => debug!("Could not load relays.yaml: {}", e),
    }

    let bitchat_online_relays_url = "https://raw.githubusercontent.com/permissionlesstech/bitchat/refs/heads/main/relays/online_relays_gps.csv";
    match fetch_online_relays(bitchat_online_relays_url).await {
        Ok(urls) => {
            for url_str in urls {
                relays.add(&url_str);
            }
            debug!(
                "Loaded {} relays from online CSV (bitchat)",
                relays.count()
            );
        }
        Err(e) => debug!("Could not fetch online relays from bitchat: {}", e),
    }

    let sesseor_online_relays_url =
        "https://raw.githubusercontent.com/sesseor/nostr-relays-list/main/relays.txt";
    match fetch_online_relays(sesseor_online_relays_url).await {
        Ok(urls) => {
            for url_str in urls {
                relays.add(&url_str);
            }
            debug!(
                "Loaded {} relays from online TXT (sesseor)",
                relays.count()
            );
        }
        Err(e) => debug!("Could not fetch online relays from sesseor: {}", e),
    }

    relays
}

pub fn seed_bootstrap_relays(relays: &mut Relays, bootstrap_relays: &[&str]) {
    for relay in bootstrap_relays {
        if is_shitlisted(relay) {
            debug!("Skipping shitlisted bootstrap relay: {}", relay);
            continue;
        }

        relays.add(relay);
    }
}

pub fn is_shitlisted(url: &str) -> bool {
    SHITLIST_RELAYS.iter().any(|relay| url.contains(relay))
}
