use clap::{Parser, Subcommand};
use futures::{stream, StreamExt};
use reqwest::header::ACCEPT;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use tracing::{debug, error};

use gnostr_crawler::load_file;
use gnostr_crawler::load_shitlist;
use gnostr_crawler::Relay;

use gnostr_crawler::processor::Processor;
use gnostr_crawler::processor::APP_SECRET_KEY;
use gnostr_crawler::processor::BOOTSTRAP_RELAY1;
use gnostr_crawler::processor::BOOTSTRAP_RELAY2;
use gnostr_crawler::processor::BOOTSTRAP_RELAY3;
use gnostr_crawler::processor::BOOTSTRAP_RELAYS;
use gnostr_crawler::relay_manager::RelayManager;
use nostr_sdk::prelude::{FromBech32, Keys, SecretKey};

const CONCURRENT_REQUESTS: usize = 16;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Runs the sniper mode to find relays supporting a specific NIP
    Sniper {
        /// The NIP number to search for (e.g., 1)
        nip: i32,
        /// Optional: Path to a shitlist file to exclude relays
        #[clap(long, short)]
        shitlist: Option<String>,
    },
    /// Runs the watch mode to monitor relays and print their metadata
    Watch {
        /// Optional: Path to a shitlist file to exclude relays
        #[clap(long, short)]
        shitlist: Option<String>,
    },
    /// Lists relays that are likely to support NIP-34 (Git collaboration)
    Nip34 {
        /// Optional: Path to a shitlist file to exclude relays
        #[clap(long, short)]
        shitlist: Option<String>,
    },
}

async fn run_sniper(
    nip_lower: i32,
    shitlist_path: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let relays = load_file("relays.yaml").unwrap();
    let client = reqwest::Client::new();

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
            let client = &client;
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
        .buffer_unordered(CONCURRENT_REQUESTS);

    bodies
        .for_each(|b: Result<(String, String), reqwest::Error>| async {
            if let Ok((url, json_string)) = b {
                let data: Result<Relay, _> = serde_json::from_str(&json_string);
                if let Ok(relay_info) = data {
                    for n in &relay_info.supported_nips {
                        if n == &nip_lower {
                            debug!("contact:{:?}", &relay_info.contact);
                            debug!("description:{:?}", &relay_info.description);
                            debug!("name:{:?}", &relay_info.name);
                            debug!("software:{:?}", &relay_info.software);
                            debug!("version:{:?}", &relay_info.version);

                            let dir_name = format!("{}", nip_lower);
                            let path = Path::new(&dir_name);

                            if !path.exists() {
                                match fs::create_dir(path) {
                                    Ok(_) => debug!("created {}", nip_lower),
                                    Err(e) => eprintln!("Error creating directory: {}", e),
                                }
                            } else {
                                debug!("{} already exists...", dir_name);
                            }

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

                            match File::create(&file_path) {
                                Ok(mut file) => {
                                    debug!("{}", &file_path_str);
                                    match file.write_all(json_string.as_bytes()) {
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
                                nip_lower,
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

async fn run_watch(shitlist_path: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
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

    let relays_iterator = relays.into_iter().filter(|url| {
        if shitlist.is_empty() {
            true
        } else {
            !shitlist
                .iter()
                .any(|shitlisted_url| url.contains(shitlisted_url))
        }
    });

    let client = reqwest::Client::new();
    let bodies = stream::iter(relays_iterator)
        .map(|url| {
            let client = &client;
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
        .buffer_unordered(CONCURRENT_REQUESTS);

    bodies
        .for_each(|b: Result<(String, String), reqwest::Error>| async {
            if let Ok((url, json_string)) = b {
                println!("{{\"relay\":\"{}\", \"data\":{}}}", url, json_string);
                let data: Result<Relay, serde_json::Error> = serde_json::from_str(&json_string);
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
                    print!("\"}}");
                    println!();
                }
            }
        })
        .await;

    Ok(())
}

async fn run_nip34(shitlist_path: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let relays = load_file("relays.yaml").unwrap();
    let client = reqwest::Client::new();

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
            let client = &client;
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
        .buffer_unordered(CONCURRENT_REQUESTS);

    bodies
        .for_each(|b: Result<(String, String), reqwest::Error>| async {
            if let Ok((url, json_string)) = b {
                let data: Result<Relay, _> = serde_json::from_str(&json_string);
                if let Ok(relay_info) = data {
                    let supports_nip01 = relay_info.supported_nips.contains(&1);
                    let supports_nip11 = relay_info.supported_nips.contains(&11);

                    if supports_nip01 && supports_nip11 {
                        println!("{}", url);
                    }
                }
            }
        })
        .await;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    //tracing_subscriber::fmt()
    //    .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
    //    .init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Sniper { nip, shitlist } => {
            run_sniper(*nip, shitlist.clone()).await?;
        }
        Commands::Watch { shitlist } => {
            run_watch(shitlist.clone()).await?;
        }
        Commands::Nip34 { shitlist } => {
            run_nip34(shitlist.clone()).await?;
        }
    }

    Ok(())
}
