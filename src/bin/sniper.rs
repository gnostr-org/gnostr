use futures::{stream, StreamExt};
use reqwest::header::ACCEPT;
use serde::{Deserialize, Serialize};
use std::{
    fs::{File, OpenOptions},
    io::{self, BufRead, BufReader, Write},
    path::Path,
};

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
    BufReader::new(File::open(filename)?).lines().collect()
}

fn append_to_file(filename: &str, data_to_append: &str) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true) // Creates the file if it doesn't exist
        .open(filename)?;

    file.write_all(data_to_append.as_bytes())?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let mut file_path = "./relays.yaml".to_string();

    let file = File::open(file_path.clone()).expect("");

   // let mut file = OpenOptions::new()
   //     .write(true)
   //     //.append(true)
   //     //.create(true) // Creates the file if it doesn't exist
   //     .open(file_path.clone()).expect("");



let filename = file_path.clone()+".txt";

    let reader = io::BufReader::new(&file);
    // Iterate over each line in the file
    for line_result in reader.lines() {
		//append_to_file(&filename, line_result.expect(""));



        //// Replace "wss://" with "https://" in the current line
        let modified_line = line_result.expect("REASON").replace("wss://", "https://").replace("ws://","http://");

        //// Print the modified line
        println!("{}", modified_line);
        //println!("{:?}", line_result);

        let mut file = File::create(file_path.clone() + ".txt").expect("");



		if !modified_line.contains("monad.jb55.com") &&
			!modified_line.contains("onlynotes") &&
			!modified_line.contains("archives") &&
			!modified_line.contains("relay.siamstr.com") &&
			!modified_line.contains("mguy"){
        file.write(modified_line.as_bytes()).expect("");
        //file.write(b"\n").expect("");
		}
        ////writeln!(file, "{}", line).expect("");

        let mut file_path = "./relays.yaml.txt".to_string();

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

                    let r: Result<(String, String), reqwest::Error> = Ok((url, text));
                    println!("{:?}", r);
                    r
                }
            })
            .buffer_unordered(CONCURRENT_REQUESTS);

        bodies
            .for_each(|b| async {
                if let Ok((url, json)) = b {
                    let data: Result<Relay, serde_json::Error> = serde_json::from_str(&json);
                    if let Ok(json) = data {
                        for n in &json.supported_nips {
                            println!("{:?}", n);
                            if n == &nip {
                                println!("{} Supports nip{nip}", url);
                            }
                        }
                    }
                }
            })
            .await;
    }
    Ok(())
}
