use futures::{stream, StreamExt};
use reqwest::header::ACCEPT;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::{self, File};
use std::{
    io::{self, BufRead, BufReader, Write},
    path::Path,
};
use tracing::{debug, error};
use clap::Parser;

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

fn load_file(filename: impl AsRef<Path>) -> io::Result<Vec<String>> {
    BufReader::new(fs::File::open(filename)?).lines().collect()
}

/// Create a HashSet of blocked relay domains for O(1) lookup performance
fn create_blocklist() -> HashSet<&'static str> {
    let mut blocklist = HashSet::with_capacity(40);
    blocklist.insert("monad.jb55.com");
    blocklist.insert("onlynotes");
    blocklist.insert("archives");
    blocklist.insert("relay.siamstr.com");
    blocklist.insert("no.str");
    blocklist.insert("multiplexer.huszonegy.world");
    blocklist.insert("relay.0xchat.com");
    blocklist.insert("snort.social");
    blocklist.insert("mguy");
    blocklist.insert("stoner.com");
    blocklist.insert("nostr.nodeofsven.com");
    blocklist.insert("nvote.co");
    blocklist.insert("utxo");
    blocklist.insert("relay.lexingtonbitcoin.org");
    blocklist.insert("nostr.info");
    blocklist.insert("nostr.band");
    blocklist.insert("bitcoin.ninja");
    blocklist.insert("brb.io");
    blocklist.insert("nbo.angani.co");
    blocklist.insert("nostr.relayer.se");
    blocklist.insert("relay.nostr.nu");
    blocklist.insert("knostr.neutrine.com");
    blocklist.insert("nostr.easydns.ca");
    blocklist.insert("relay.nostrgraph.net");
    blocklist.insert("gruntwerk.org");
    blocklist.insert("nostr.noones.com");
    blocklist.insert("relay.nonce.academy");
    blocklist.insert("relay.r3d.red");
    blocklist.insert("nostr.bitcoiner.social");
    blocklist.insert("btc.klendazu.com");
    blocklist.insert("vulpem.com");
    blocklist.insert("bch.ninja");
    blocklist.insert("sg.qemura.xyz");
    blocklist.insert("relay.schnitzel.world");
    blocklist.insert("nostr.datamagik.com");
    blocklist.insert("nostrid");
    blocklist.insert("damus.io");
    blocklist.insert(".local");
    blocklist
}

/// Check if a URL contains any blocked domain
fn is_url_blocked(url: &str, blocklist: &HashSet<&str>) -> bool {
    blocklist.iter().any(|blocked| url.contains(blocked))
}

/// Strip protocol prefixes from URL more efficiently than chained .replace() calls
/// Handles wss://, https://, and ws:// protocols
fn strip_protocol(url: &str) -> &str {
    if let Some(stripped) = url.strip_prefix("wss://") {
        stripped
    } else if let Some(stripped) = url.strip_prefix("https://") {
        stripped
    } else if let Some(stripped) = url.strip_prefix("ws://") {
        stripped
    } else {
        url
    }
}

#[derive(Parser, Debug, Clone)]
pub struct SniperArgs {
    /// The minimum NIP version to filter relays by.
    #[clap(short, long, default_value = "1")]
    nip_lower: i32,
    // The second argument from the original main function was commented out,
    // so we'll omit it here unless it's explicitly needed.
    // #[clap(short, long)]
    // nip_upper: i32,
}

pub async fn run_sniper(args: SniperArgs) -> Result<(), Box<dyn std::error::Error>> {
    let relays = load_file("relays.yaml").unwrap();
    let client = reqwest::Client::new();
    let blocklist = create_blocklist();
    
    let bodies = stream::iter(relays)
        .map(|url| {
            let client = &client;
            let blocklist = &blocklist;
            let _nip_lower = args.nip_lower; // Capture nip_lower from args
            async move {
                let resp = client
                    .get(url.replace("wss://", "https://"))
                    .header(ACCEPT, "application/nostr+json")
                    .send()
                    .await?;
                let text = resp.text().await?;

                // Avoid cloning - move url ownership into the result
                let r: Result<(String, String), reqwest::Error> = Ok((url, text));

                r
            }
        })
        .buffer_unordered(CONCURRENT_REQUESTS);

    bodies
        .for_each(|b| async {
            if let Ok((url, json)) = b {
                // Skip blocked relays using efficient HashSet lookup
                if is_url_blocked(&url, &blocklist) {
                    return;
                }
                
                let data: Result<Relay, _> = serde_json::from_str(&json);
                if let Ok(relay_info) = data {
                    for n in &relay_info.supported_nips {
                        if n == &args.nip_lower { // Use nip_lower from args
                            debug!("contact:{:?}", &relay_info.contact);
                            debug!("description:{:?}", &relay_info.description);
                            debug!("name:{:?}", &relay_info.name);
                            debug!("software:{:?}", &relay_info.software);
                            debug!("version:{:?}", &relay_info.version);

                            let dir_name = format!("{}", args.nip_lower); // Use nip_lower from args
                            let path = Path::new(&dir_name);

                            if !path.exists() {
                                match fs::create_dir(path) {
                                    Ok(_) => debug!("created ./{}", args.nip_lower),
                                    Err(e) => eprintln!("Error creating directory: {}", e),
                                }
                            } else {
                                debug!("{dir_name} already exists...");
                            }

                            let file_name = format!("{}.json", strip_protocol(&url));
                            let file_path = path.join(&file_name);
                            let file_path_str = file_path.display().to_string();
                            debug!("

{}

", file_path_str);

                            match File::create(&file_path) {
                                Ok(mut file) => {
                                    debug!("{}", &file_path_str);
                                    match file.write_all(json.as_bytes()) {
                                        Ok(_) => debug!("wrote relay metadata:{}", &file_path_str),
                                        Err(e) => {
                                            error!("Failed to write to {}: {}", &file_path_str, e)
                                        }
                                    }
                                }
                                Err(e) => error!("Failed to create file {}: {}", &file_path_str, e),
                            }

                            println!(
                                "{}/{}",
                                args.nip_lower, // Use nip_lower from args
                                strip_protocol(&url)
                            );
                        }
                    }
                }
            }
        })
        .await;

    Ok(())
}
