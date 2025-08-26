use clap::Parser;
use futures::{stream, StreamExt};
use reqwest::header::ACCEPT;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use tracing::{debug, error, info};

const CONCURRENT_REQUESTS: usize = 16;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// NIP to check for
    #[arg(short, long, default_value_t = 1)]
    nip: i32,
}

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

fn load_shitlist(filename: impl AsRef<Path>) -> io::Result<HashSet<String>> {
    BufReader::new(fs::File::open(filename)?).lines().collect()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let nip_lower = args.nip;

    let relays = match load_file("relays.yaml") {
        Ok(relays) => relays,
        Err(e) => {
            error!("Failed to load relays.yaml: {}", e);
            return Err(e.into());
        }
    };

    let shitlist = match load_shitlist("shitlist.txt") {
        Ok(shitlist) => shitlist,
        Err(e) => {
            error!("Failed to load shitlist.txt: {}", e);
            return Err(e.into());
        }
    };

    let dir_name = format!("{}", nip_lower);
    let path = Path::new(&dir_name);
    if !path.exists() {
        if let Err(e) = fs::create_dir(path) {
            error!("Error creating directory: {}", e);
        } else {
            info!("created ./{}", nip_lower);
        }
    }

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
                    .get(url.replace("wss://", "https://"))
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
                let data: Result<Relay, serde_json::Error> = serde_json::from_str(&json);
                if let Ok(relay_info) = data {
                    if relay_info.supported_nips.contains(&nip_lower) {
                        debug!("contact:{:?}", &relay_info.contact);
                        debug!("description:{:?}", &relay_info.description);
                        debug!("name:{:?}", &relay_info.name);
                        debug!("software:{:?}", &relay_info.software);
                        debug!("version:{:?}", &relay_info.version);

                        let file_name = url
                            .replace("https://", "")
                            .replace("http://", "")
                            .replace("ws://", "")
                            .replace("wss://", "")
                            + ".json";
                        let file_path = path.join(&file_name);
                        let file_path_str = file_path.display().to_string();
                        debug!(
                            "

{}

",
                            file_path_str
                        );

                        match fs::File::create(&file_path) {
                            Ok(mut file) => {
                                debug!("{}", &file_path_str);
                                if let Err(e) = file.write_all(json.as_bytes()) {
                                    error!("Failed to write to {}: {}", &file_path_str, e);
                                } else {
                                    debug!("wrote relay metadata:{}", &file_path_str);
                                }
                            }
                            Err(e) => error!("Failed to create file {}: {}", &file_path_str, e),
                        }

                        println!(
                            "{}/{}",
                            nip_lower,
                            url.replace("https://", "")
                                .replace("wss://", "")
                                .replace("ws://", "")
                        );
                    }
                }
            }
        })
        .await;

    Ok(())
}
