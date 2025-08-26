use futures::{stream, StreamExt};
use reqwest::header::ACCEPT;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use tracing::debug;

use gnostr_crawler::processor::Processor;
use gnostr_crawler::processor::APP_SECRET_KEY;
use gnostr_crawler::processor::BOOTSTRAP_RELAY1;
use gnostr_crawler::processor::BOOTSTRAP_RELAY2;
use gnostr_crawler::processor::BOOTSTRAP_RELAY3;
use gnostr_crawler::processor::BOOTSTRAP_RELAYS;
use gnostr_crawler::relay_manager::RelayManager;
use nostr_sdk::prelude::{FromBech32, Keys, SecretKey};

const CONCURRENT_REQUESTS: usize = 16;

#[derive(Serialize, Deserialize, Debug)]
struct Relay {
    contact: String,
    description: String,
    name: String,
    software: String,
    supported_nips: Vec<i32>,
    version: String,
}

fn load_shitlist(filename: impl AsRef<Path>) -> io::Result<HashSet<String>> {
    BufReader::new(fs::File::open(filename)?).lines().collect()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get relays from the crawler
    let app_secret_key = SecretKey::from_bech32(APP_SECRET_KEY)?;
    let app_keys = Keys::new(app_secret_key);
    let processor = Processor::new();
    let mut relay_manager = RelayManager::new(app_keys, processor);

    let bootstrap_relays = vec![
        BOOTSTRAP_RELAY1,
        BOOTSTRAP_RELAY2,
        BOOTSTRAP_RELAY3,
        BOOTSTRAP_RELAYS
            .get(3)
            .expect("BOOTSTRAP_RELAYS should have at least 4 elements")
            .as_str(),
    ];
    relay_manager.run(bootstrap_relays).await?;
    let relays: Vec<String> = relay_manager.relays.get_all();

    let shitlist = match load_shitlist("shitlist.txt") {
        Ok(shitlist) => shitlist,
        Err(e) => {
            eprintln!("Failed to load shitlist.txt: {}", e);
            return Err(e.into());
        }
    };

    let relays_iterator = relays.into_iter().filter(|url| {
        !shitlist
            .iter()
            .any(|shitlisted_url| url.contains(shitlisted_url))
    });

    let client = reqwest::Client::new();
    let bodies = stream::iter(relays_iterator)
        .map(|url| {
            let client = &client;
            async move {
                let resp = client
                    .get(
                        &url.replace("wss://", "https://")
                            .replace("ws://", "http://"),
                    )
                    .header(ACCEPT, "application/nostr+json")
                    .send()
                    .await?;
                let text = resp.text().await?;
                Ok((url, text))
            }
        })
        .buffer_unordered(CONCURRENT_REQUESTS);

    bodies
        .for_each(|b: Result<(String, String), reqwest::Error>| async {
            if let Ok((url, json)) = b {
                println!("{{\"relay\":\"{}\"}}", url);
                println!("{}", json);
                let data: Result<Relay, serde_json::Error> = serde_json::from_str(&json);
                if let Ok(relay_info) = data {
                    print!("{{\"nips\":\"");
                    debug!("len:{} ", relay_info.supported_nips.len());
                    let mut nip_count = relay_info.supported_nips.len();
                    for n in &relay_info.supported_nips {
                        debug!("nip_count:{}", nip_count);
                        if nip_count > 1 {
                            print!("{:0>2} ", n);
                        } else {
                            print!("{:0>2}", n);
                        }
                        nip_count -= 1;
                    }
                    print!("{}", "\"}}");
                    println!();
                }
            }
        })
        .await;

    Ok(())
}
