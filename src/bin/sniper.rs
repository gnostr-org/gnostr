use futures::{stream, StreamExt};
use reqwest::header::ACCEPT;
use serde::{Deserialize, Serialize};
use std::{
    env,
    fs::File,
    io::{self, BufRead, BufReader, Write},
    path::Path,
    process::{Command, Stdio},
};

use tracing::debug;
//use tracing_subscriber::FmtSubscriber;
//use tracing_core::metadata::LevelFilter;

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

fn gnostr_crawler() -> Result<(), Box<dyn std::error::Error>> {
    let mut command = Command::new("gnostr-crawler");

    // Configure the command to capture standard output
    command.stdout(Stdio::piped());

    let mut child = command.spawn()?;

    // Open the file to write to
    let mut outfile = File::create("relays.yaml")?;

    if let Some(mut stdout) = child.stdout.take() {
        let mut buffer = Vec::new();
        std::io::copy(&mut stdout, &mut buffer)?;
        outfile.write_all(&buffer)?;
    }

    // Wait for the child process to finish
    child.wait()?;

    Ok(())
}

fn load_file(filename: impl AsRef<Path>) -> io::Result<Vec<String>> {
    BufReader::new(File::open(filename)?).lines().collect()
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let args: Vec<String> = env::args().collect();
    let mut nip_lower: i32 = 1;
    let mut nip_upper: i32 = 1;
    let mut first_argument = "";
    let mut second_argument = "";
    if args.len() > 1 {
        first_argument = &args[1];

        match first_argument.parse::<i32>() {
            Ok(number) => {
                nip_lower = number;

                use std::fs::DirBuilder;
                use std::io;

                #[cfg(unix)]
                use std::os::unix::fs::DirBuilderExt;

                let mut builder = DirBuilder::new();
                builder.recursive(true); // Create parent directories if they don't exist

                #[cfg(unix)]
                builder.mode(0o755); // Set permissions (read, write, execute for owner; read and execute for group and others)

                builder
                    .create(format!("{}", first_argument))
                    .expect("failed to create relays sub directory.");
            }
            Err(e) => {
                eprintln!("Error converting first argument to i32: {}", e);
            }
        }
    }
    if args.len() > 2 {
        second_argument = &args[2];

        match second_argument.parse::<i32>() {
            Ok(number) => {
                nip_upper = number;
                use std::fs::DirBuilder;
                use std::io;

                #[cfg(unix)]
                use std::os::unix::fs::DirBuilderExt;

                let mut builder = DirBuilder::new();
                builder.recursive(true); // Create parent directories if they don't exist

                #[cfg(unix)]
                builder.mode(0o755); // Set permissions (read, write, execute for owner; read and execute for group and others)

                builder
                    .create(format!("{}", second_argument))
                    .expect("failed to create relays sub directory.");
            }
            Err(e) => {
                eprintln!("Error converting first argument to i32: {}", e);
            }
        }
    }

    let file_path = "./relays.yaml".to_string();

    //TODO:gnostr-sniper --refresh
    let _gnostr_crawler = gnostr_crawler();
    let file = File::open(file_path.clone()).expect("");

    let reader = io::BufReader::new(&file);
    for line_result in reader.lines() {
        //append_to_file(&filename, line_result.expect(""));
        let modified_line = line_result
            .expect("use http/https:// when querying supported nips")
            .replace("wss://", "https://")
            .replace("ws://", "http://");

        debug!("132:{}", modified_line);
        let mut file = File::create(file_path.clone() + ".txt")
            .expect("create relays.yaml.txt and modify protocol (ws/wss to http/https)");
        if !modified_line.contains("monad.jb55.com")
            && !modified_line.contains("onlynotes")
            && !modified_line.contains("archives")
            && !modified_line.contains("relay.siamstr.com")
            && !modified_line.contains("no.str")
            && !modified_line.contains("multiplexer.huszonegy.world")
            && !modified_line.contains("relay.0xchat.com")
            && !modified_line.contains("snort.social")
            && !modified_line.contains("mguy")
            && !modified_line.contains("stoner.com")
            && !modified_line.contains("nostr.nodeofsven.com")
            && !modified_line.contains("nvote.co")
            && !modified_line.contains("utxo")
            && !modified_line.contains("relay.lexingtonbitcoin.org")
            && !modified_line.contains("nostr.info")
            && !modified_line.contains("nostr.band")
            && !modified_line.contains("bitcoin.ninja")
            && !modified_line.contains("brb.io")
            && !modified_line.contains("nbo.angani.co")
            && !modified_line.contains("nostr.relayer.se")
            && !modified_line.contains("relay.nostr.nu")
            && !modified_line.contains("knostr.neutrine.com")
            && !modified_line.contains("nostr.easydns.ca")
            && !modified_line.contains("relay.nostrgraph.net")
            && !modified_line.contains("gruntwerk.org")
            && !modified_line.contains("nostr.noones.com")
            && !modified_line.contains("relay.nonce.academy")
            && !modified_line.contains("relay.r3d.red")
            && !modified_line.contains("nostr.bitcoiner.social")
            && !modified_line.contains("btc.klendazu.com")
            && !modified_line.contains("vulpem.com")
            && !modified_line.contains("bch.ninja")
            && !modified_line.contains("sg.qemura.xyz")
            && !modified_line.contains("relay.schnitzel.world")
            && !modified_line.contains("nostr.datamagik.com")
            && !modified_line.contains("nostrid")
            && !modified_line.contains("damus.io")
            && !modified_line.contains(".local")
            && !modified_line.contains("onlynotes")
            && !modified_line.contains("archives")
            && !modified_line.contains("relay.siamstr.com")
            && !modified_line.contains("no.str")
            && !modified_line.contains("multiplexer.huszonegy.world")
            && !modified_line.contains("relay.0xchat.com")
            && !modified_line.contains("snort.social")
            && !modified_line.contains("mguy")
            && !modified_line.contains("stoner.com")
            && !modified_line.contains("nostr.info")
            && !modified_line.contains(".local")
        //we want a view of the network
        {
            file.write(modified_line.as_bytes()).expect("");
            //file.write(b"\n").expect("");
        }
        ////writeln!(file, "{}", line).expect("");

        let file_path = "./relays.yaml.txt".to_string();

        //}

        let relays = load_file(&file_path).unwrap();
        // Nip you are looking for on relays
        //println!("{:?}", relays);
        let nip = 11;

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

                    let r: Result<(String, String), reqwest::Error> =
                        Ok((url.clone(), text.clone()));
                    //tracing::info!("{:?}", r);
                    //print!("{{\"relay\":\"{}\",", url);
                    debug!("{}", text);
                    r
                }
            })
            .buffer_unordered(CONCURRENT_REQUESTS);

        bodies
            .for_each(|b| async {
                if let Ok((url, json)) = b {
                    let data: Result<Relay, serde_json::Error> = serde_json::from_str(&json);
                    if let Ok(json) = data {
                        print!("{{\"227:url\":\"{}\",", url.replace("https", "wss"));
                        print!("\"228:supported_nips\":[");
                        //debug!("len:{} ", json.supported_nips.len());
                        let mut nip_count = json.supported_nips.len();
                        for n in &json.supported_nips {
                            //debug!("nip_count:{}", nip_count);
                            if nip_count > 1 {
                                print!("{:<2}", format!("{},", n));
                            } else {
                                print!("{:<2}", format!("{}", n));
                            }
                            if n == &nip {
                                debug!("{} Supports nip{nip}", url);
                            }

                            //print!("242:{} ", file_path);
                            let file_name = url
                                .replace("https://", "")
                                .replace("http://", "")
                                .replace("wss://", "")
                                .replace("ws://", "")
                                + ".json";
                            let file_path = file_path.clone() + &file_name;
                            //print!("250:{} ", file_path);
                            let file_path_str = file_path.to_string();
                            //print!("252:{} ", file_path_str);

                            use tracing::error;
                            match File::create(&file_path) {
                                Ok(mut file) => {
                                    debug!("257:{}", &file_path_str);
                                    match file.write_all(json.contact.as_bytes()) {
                                        Ok(_) => debug!("wrote relay metadata:{}", &file_path_str),
                                        Err(e) => {
                                            error!("Failed to write to {}: {}", &file_path_str, e)
                                        }
                                    }
                                }
                                Err(e) => error!("Failed to create file {}: {}", &file_path_str, e),
                            }
                            //print!("]}}");
                            //println!();
                            nip_count = nip_count - 1;
                        }
                        print!("]}}");
                        println!();
                        //nip_count = nip_count - 1;
                    }
                }
            })
            .await;
        //}
    }
    Ok(())
}
