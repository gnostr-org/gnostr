use futures::{stream, StreamExt};
use reqwest::header::ACCEPT;
use serde::{Deserialize, Serialize};
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
    let bodies = stream::iter(relays)
        .map(|url| {
            let client = &client;
            let nip_lower = args.nip_lower; // Capture nip_lower from args
            async move {
                let resp = client
                    .get(url.replace("wss://", "https://"))
                    .header(ACCEPT, "application/nostr+json")
                    .send()
                    .await?;
                let text = resp.text().await?;

                let r: Result<(String, String), reqwest::Error> = Ok((url.clone(), text.clone()));

                //shitlist - This filtering logic should ideally be configurable or more robust.
                if !url.contains("monad.jb55.com")
                    && !url.contains("onlynotes")
                    && !url.contains("archives")
                    && !url.contains("relay.siamstr.com")
                    && !url.contains("no.str")
                    && !url.contains("multiplexer.huszonegy.world")
                    && !url.contains("relay.0xchat.com")
                    && !url.contains("snort.social")
                    && !url.contains("mguy")
                    && !url.contains("stoner.com")
                    && !url.contains("nostr.nodeofsven.com")
                    && !url.contains("nvote.co")
                    && !url.contains("utxo")
                    && !url.contains("relay.lexingtonbitcoin.org")
                    && !url.contains("nostr.info")
                    && !url.contains("nostr.band")
                    && !url.contains("bitcoin.ninja")
                    && !url.contains("brb.io")
                    && !url.contains("nbo.angani.co")
                    && !url.contains("nostr.relayer.se")
                    && !url.contains("relay.nostr.nu")
                    && !url.contains("knostr.neutrine.com")
                    && !url.contains("nostr.easydns.ca")
                    && !url.contains("relay.nostrgraph.net")
                    && !url.contains("gruntwerk.org")
                    && !url.contains("nostr.noones.com")
                    && !url.contains("relay.nonce.academy")
                    && !url.contains("relay.r3d.red")
                    && !url.contains("nostr.bitcoiner.social")
                    && !url.contains("btc.klendazu.com")
                    && !url.contains("vulpem.com")
                    && !url.contains("bch.ninja")
                    && !url.contains("sg.qemura.xyz")
                    && !url.contains("relay.schnitzel.world")
                    && !url.contains("nostr.datamagik.com")
                    && !url.contains("nostrid")
                    && !url.contains("damus.io")
                    && !url.contains(".local")
                {
                    //we want a view of the network
                }
                r
            }
        })
        .buffer_unordered(CONCURRENT_REQUESTS);

    bodies
        .for_each(|b| async {
            if let Ok((url, json)) = b {
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

                            let file_name = url
                                .replace("wss://", "")
                                .replace("https://", "")
                                .replace("ws://", "")
                                + ".json";
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
                                url.replace("https://", "")
                                    .replace("wss://", "")
                                    .replace("ws://", "")
                            );
                        }
                    }
                }
            }
        })
        .await;

    Ok(())
}
