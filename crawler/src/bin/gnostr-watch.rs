use futures::{stream, StreamExt};
use reqwest::header::ACCEPT;
use serde::{Deserialize, Serialize};
use std::{
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

async fn gnostr_crawler() -> Result<(), Box<dyn std::error::Error>> {
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
    let file_path = "./relays.yaml".to_string();

    //TODO:gnostr-sniper --refresh
    let _gnostr_crawler = gnostr_crawler().await;
    let file = File::open(file_path.clone()).expect("");

    let reader = io::BufReader::new(&file);
    for line_result in reader.lines() {
        //append_to_file(&filename, line_result.expect(""));
        let modified_line = line_result
            .expect("use http/https:// when querying supported nips")
            .replace("wss://", "https://")
            .replace("ws://", "http://");

        debug!("{}", modified_line);
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
        let nip = 0;

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
                    println!("{{\"relay\":\"{}\"}}", url);
                    println!("{}", text);
                    r
                }
            })
            .buffer_unordered(CONCURRENT_REQUESTS);

        bodies
            .for_each(|b| async {
                if let Ok((url, json)) = b {
                    let data: Result<Relay, serde_json::Error> = serde_json::from_str(&json);
                    if let Ok(json) = data {
                        print!("{{\"nips\":\"");
                        debug!("len:{} ", json.supported_nips.len());
                        let mut nip_count = json.supported_nips.len();
                        for n in &json.supported_nips {
                            debug!("nip_count:{}", nip_count);
                            if nip_count > 1 {
                                print!("{:<3}", format!("{:0>2}", n));
                            } else {
                                print!("{:<2}", format!("{:0>2}", n));
                            }
                            if n == &nip {
                                println!("{} Supports nip{nip}", url);
                            }
                            nip_count = nip_count - 1;
                        }
                        print!("\"}}");
                        println!();
                    }
                }
            })
            .await;
    }
    Ok(())
}
