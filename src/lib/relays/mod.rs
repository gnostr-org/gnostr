use std::io::Read;

use anyhow::Result;
use reqwest::Url;

// https://api.nostr.watch/v2/#get/openapijson
// let client = reqwest::Client::new();
// 
// let request = client.get("https://api.nostr.watch/v2/openapi.json");
// 
// let response = request.send().await?;

fn _get_relays_from_url_path(path: &str) -> Result<String> {
    let url = Url::parse(&format!("https://api.nostr.watch/v2/{}", path))?;
    let mut res = reqwest::blocking::get(url)?;
    let mut tmp_string = String::new();
    res.read_to_string(&mut tmp_string)?;
    Ok(tmp_string)
}

pub fn relays_by_nip(nip: &str) -> Result<String> {
    _get_relays_from_url_path(&format!("relays/by/nip/{}", nip))
}

pub fn relays_all() -> Result<String> {
    _get_relays_from_url_path("relays")
}

pub fn relays_online() -> Result<String> {
    _get_relays_from_url_path("relays/online")
}

pub fn relays_paid() -> Result<String> {
    _get_relays_from_url_path("relays/paid")
}

pub fn relays_offline() -> Result<String> {
    _get_relays_from_url_path("relays/offline")
}
