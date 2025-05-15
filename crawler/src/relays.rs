use log::info;
use log::{debug, trace};
use nostr_sdk::prelude::Url;
use std::collections::HashSet;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Relay {
    contact: String,
    description: String,
    name: String,
    software: String,
    supported_nips: Vec<i32>,
    version: String,
}

/// Maintain a list of all encountered relays
pub struct Relays {
    r: HashSet<Url>,
}

impl Default for Relays {
    fn default() -> Self {
        Self::new()
    }
}

impl Relays {
    pub fn new() -> Self {
        Self {
            r: HashSet::default(),
        }
    }

    pub async fn add(&mut self, s1: &str) -> bool {
        let mut res = false;
        if let Ok(u) = Url::parse(s1) {
            res = self.r.insert(u);
            if res {
                self.print().await;
            }
        }
        res
    }

    pub fn count(&self) -> usize {
        self.r.len()
    }

    pub fn de_dup(&self, list: &Vec<Url>) -> Vec<Url> {
        let list: Vec<Url> = list.clone().into_iter().collect();
        list
    }

    pub fn get_some(&self, max_count: usize) -> Vec<Url> {
        let mut res = Vec::new();
        for u in &self.r {
            res.push(u.clone());
            if res.len() >= max_count {
                return res;
            }
        }
        res = self.de_dup(&res);
        res
    }

    pub async fn print(&self) {
		println!("relays.rs:print");
        for u in &self.r {
            let mut relays = vec![];
            let mut relay = format!("{}", u.to_string());
            if relay.ends_with('/') {
                relay.pop();
                println!("{}", relay);
                relays.push(relay);
            } else {
                println!("{}", relay);
                relays.push(relay);
            }

            use futures::{stream, StreamExt};
            use reqwest::header::ACCEPT;
            const CONCURRENT_REQUESTS: usize = 16;

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
    }

    pub fn dump_json_object(&self) {
        let mut count = 0;
        print!("[\"RELAYS\",");
        for u in &self.r {
            print!("{{\"{}\":\"{}\"}},", count, u);
            count += 1;
        }
        print!("{{\"{}\":\"wss://relay.gnostr.org\"}}", count);
        print!("]");
    }

    pub fn dump_list(&self) {
        let mut count = 0;
        for u in &self.r {
            print!("78:{{\"{}\":\"{}\"}}", count, u);
            count += 1;
        }
        print!("{{\"{}\":\"wss://relay.gnostr.org\"}}", count);
    }
}
