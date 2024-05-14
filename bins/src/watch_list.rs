//use futures::executor::block_on;
//use url::Url;

use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};

use crate::get_relays_public;

#[derive(Serialize, Deserialize)]
struct Relay {
    url: String,
}

pub async fn parse_json(urls_str: &str) -> Result<Vec<String>> {
    let mut urls: Vec<String> = Vec::new();
    let mut part = String::new();
    let mut collected = Vec::new();
    let mut char_iter = urls_str.chars();
    for _ in urls_str.chars() {
        if char_iter.next() == Some('[') {
        print!("[\"RELAYS\", ");
        }
        loop {
            match char_iter.next() {
                Some(']') => {
                    print!("{{\"url\":\"wss://relay.gnostr.org\"}},");
                    print!("{{\"url\":\"wss://proxy.gnostr.org\"}}]");
                    return std::result::Result::Ok(collected);
                }
                Some(',') | Some(' ') => {
                    if !part.is_empty() {
                    let relay = Relay {
                        url: part.to_owned(),
                    };
                    let j = serde_json::to_string(&relay)?;
                    print!("{},", format!("{}", j.clone().replace("\\\"", "")));
                        collected.push(part.clone());
                        part = String::new();
                    } //end if !part.is_empty()
                },
                x => part.push(x.expect("REASON")),
            } //end match
        } //end loop
    }
    Ok(urls)
}
pub async fn parse_urls(urls_str: &str) -> Result<Vec<String>> {
    let mut urls: Vec<String> = Vec::new();
    let mut part = String::new();
    let mut collected = Vec::new();
    let mut char_iter = urls_str.chars();
    for url_str in urls_str.chars() {
        if char_iter.next() == Some('[') {}
        loop {
            match char_iter.next() {
                Some(']') => {
                    return std::result::Result::Ok(collected);
                }
                Some(',') | Some(' ') => {
                    if !part.is_empty() {
                        collected.push(part.clone());
                        print!("char_iter.next()={}, ", format!("{}", part.clone().replace("\"", "")));
                        part = String::new();
                    }
                },
                //None => todo!(),
                x => part.push(x.expect("REASON")),
            }
        } //end loop
    for relay in collected {
    print!("{}, ", format!("relay.clone()={}", relay.clone()));
    
    }
    }
    Ok(urls)
}

pub async fn print_watch_list() -> Result<Vec<String>> {
    let vec_relay_list = parse_urls(&get_relays_public().unwrap().as_str()).await;
    vec_relay_list //.expect("REASON")
}
pub async fn get_watch_list() -> Result<Vec<String>> {
    let vec_relay_list = parse_urls(&get_relays_public().unwrap().as_str()).await;
    vec_relay_list //.expect("REASON")
}
pub async fn get_watch_list_json() -> Result<Vec<String>> {
    let vec_relay_list = parse_json(&get_relays_public().unwrap().as_str()).await;
    vec_relay_list //.expect("REASON")
}
