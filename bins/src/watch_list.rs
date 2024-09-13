//use futures::executor::block_on;
//use url::Url;

use serde::{Deserialize, Serialize};
//use serde_json::{Result, Value};
use serde_json::Result;

use crate::get_relays_public;

#[derive(Serialize, Deserialize)]
struct Relay {
    url: String,
}

/// pub async fn parse_json(urls_str: &str) -> Result<Vec<String>>
///
pub async fn parse_json(urls_str: &str) -> Result<Vec<String>> {
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
                        collected.push(part.clone());
                        let relay = Relay {
                            url: part.to_owned(),
                        };
                        let j = serde_json::to_string(&relay)?;
                        print!("{},", format!("{}", j.clone().replace("\\\"", "")));
                        collected.push(part.clone());
                        part = String::new();
                    } //end if !part.is_empty()
                }
                x => part.push(x.expect("REASON")),
            } //end match
        } //end loop
    }
    Ok(collected)
}
/// pub async fn parse_urls(urls_str: &str) -> Result<Vec<String>>
///
pub async fn parse_urls(urls_str: &str) -> Result<Vec<String>> {
    let mut part = String::new();
    let mut collected = Vec::new();
    let mut char_iter = urls_str.chars();
    for _ in urls_str.chars() {
        if char_iter.next() == Some('[') {}
        loop {
            match char_iter.next() {
                Some(']') => {
                    print!("\"wss://relay.gnostr.org\", ");
                    print!("\"wss://proxy.gnostr.org\"");
                    return std::result::Result::Ok(collected);
                }
                Some(',') | Some(' ') => {
                    if !part.is_empty() {
                        collected.push(part.clone());
                        print!("{}, ", format!("{}", part.clone().replace("\"", "")));
                        part = String::new();
                    }
                }
                //None => todo!(),
                x => part.push(x.expect("REASON")),
            }
        } //end loop
    }
    Ok(collected)
}
/// pub async fn stripped_urls(urls_str: &str) -> Result<Vec<String>>
///
pub async fn stripped_urls(urls_str: &str) -> Result<Vec<String>> {
    let mut part = String::new();
    let mut collected = Vec::new();
    let mut char_iter = urls_str.chars();
    for _ in urls_str.chars() {
        if char_iter.next() == Some('[') {}
        if char_iter.next() == Some(',') {}
        loop {
            match char_iter.next() {
                Some(']') => {
                    return std::result::Result::Ok(collected);
                }
                Some(' ') | Some(',') => {
                    if !part.is_empty() {
                        collected.push(part.clone());
                        //print!("{}:{}",collected.len(),collected[collected.len()-1]);
                        print!(
                            "{} ",
                            format!(
                                "{}",
                                part.clone()
                                    .replace("}},", "")
                                    .replace(",", "\u{a0}")
                                    .replace("\"", "")
                            )
                        );
                        part = String::new();
                    }
                }
                //None => todo!(),
                x => part.push(x.expect("REASON")),
            }
        } //end loop
    }
    Ok(collected)
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
pub async fn get_stripped_urls() -> Result<Vec<String>> {
    let vec_relay_list = stripped_urls(&get_relays_public().unwrap().as_str()).await;
    vec_relay_list //.expect("REASON")
}
