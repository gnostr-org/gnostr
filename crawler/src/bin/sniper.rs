use futures::{stream, StreamExt};
use reqwest::header::ACCEPT;
use serde::{Deserialize, Serialize};
use std::fs;
use std::{
    env,
    io::{self, BufRead, BufReader, Write},
    path::Path,
};
use tracing::{debug, error};

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

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let args: Vec<String> = env::args().collect();
    let mut nip_lower: i32 = 1;
    let mut nip_upper: i32 = 1;
    if args.len() > 1 {
        let first_argument = &args[1];

        match first_argument.parse::<i32>() {
            Ok(number) => {
                nip_lower = number;
            }
            Err(e) => {
                eprintln!("Error converting first argument to i32: {}", e);
            }
        }
    }
    if args.len() > 2 {
        let second_argument = &args[2];

        match second_argument.parse::<i32>() {
            Ok(number) => {
                nip_upper = number;
            }
            Err(e) => {
                eprintln!("Error converting first argument to i32: {}", e);
            }
        }
    }

    let file = "relays.yaml";
    let relays = load_file(file).unwrap();
    let client = reqwest::Client::new();
    let bodies = stream::iter(relays)
        .map(|url| {
            let client = &client;
            async move {
                let resp = client
                    .get(&url)
                    .header(ACCEPT, "application/nostr+json")
                    .send()
                    .await?;
                let text = resp.text().await?;

                let r: Result<(String, String), reqwest::Error> = Ok((url.clone(), text.clone()));

                //shitlist
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
                let data: Result<Relay, serde_json::Error> = serde_json::from_str(&json);
                if let Ok(relay_info) = data {
                    for n in &relay_info.supported_nips {
                        if n == &nip_lower.clone() {
                            debug!("contact:{:?}", &relay_info.contact);
                            debug!("description:{:?}", &relay_info.description);
                            debug!("name:{:?}", &relay_info.name);
                            debug!("software:{:?}", &relay_info.software);
                            debug!("version:{:?}", &relay_info.version);

                            let dir_name = format!("{}", nip_lower);
                            let path = Path::new(&dir_name);

                            if !path.exists() {
                                match fs::create_dir(path) {
                                    Ok(_) => debug!("created ./{}", nip_lower),
                                    Err(e) => eprintln!("Error creating directory: {}", e),
                                }
                            } else {
                                debug!("{dir_name} already exists...");
                            }

                            let file_name = url.replace("https://", "").replace("/", "_") + ".json";
                            let file_path = path.join(&file_name);
                            let file_path_str = file_path.display().to_string();
                            debug!("{}", file_path_str);

                            match fs::File::create(&file_path) {
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

                            println!("{}/{}", nip_lower, url.replace("https://", ""));
                        }
                    }
                }
            }
        })
        .await;

    Ok(())
}
