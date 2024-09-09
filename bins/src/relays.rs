use std::io::Read;

use reqwest::Url;

/// pub fn nip(nip: &str) -> Result<String, ascii::AsciiChar>
pub fn relays_by_nip(nip: &str) -> Result<String, ascii::AsciiChar> {
    let nip_url = &format!("https://api.nostr.watch/v1/nip/{}", nip);
    let url = Url::parse(nip_url).unwrap();
    let mut res = reqwest::blocking::get(url).unwrap();

    let mut tmp_string = String::new();
    res.read_to_string(&mut tmp_string).unwrap().to_string();
    //println!("{}", format!("{:?}", tmp_string));
    Ok(tmp_string)
}
/// pub fn relays() -> Result<String, ascii::AsciiChar>
pub fn relays() -> Result<String, ascii::AsciiChar> {
    let url = Url::parse("https://api.nostr.watch/v1/online").unwrap();
    let mut res = reqwest::blocking::get(url).unwrap();

    let mut tmp_string = String::new();
    res.read_to_string(&mut tmp_string).unwrap().to_string();
    //println!("{}", format!("{:?}", tmp_string));
    Ok(tmp_string)
}
/// pub fn relays_public() -> Result<String, ascii::AsciiChar>
pub fn relays_public() -> Result<String, ascii::AsciiChar> {
    let url = Url::parse("https://api.nostr.watch/v1/online").unwrap();
    let mut res = reqwest::blocking::get(url).unwrap();

    let mut tmp_string = String::new();
    res.read_to_string(&mut tmp_string).unwrap().to_string();
    //println!("{}", format!("{:?}", tmp_string));
    Ok(tmp_string)
}
/// pub fn relays_online() -> Result<String, ascii::AsciiChar>
pub fn relays_online() -> Result<String, ascii::AsciiChar> {
    let url = Url::parse("https://api.nostr.watch/v1/online").unwrap();
    let mut res = reqwest::blocking::get(url).unwrap();

    let mut tmp_string = String::new();
    res.read_to_string(&mut tmp_string).unwrap().to_string();
    //println!("{}", format!("{:?}", tmp_string));
    Ok(tmp_string)
}
/// pub fn relays_paid() -> Result<String, ascii::AsciiChar>
pub fn relays_paid() -> Result<String, ascii::AsciiChar> {
    let url = Url::parse("https://api.nostr.watch/v1/paid").unwrap();
    let mut res = reqwest::blocking::get(url).unwrap();

    let mut tmp_string = String::new();
    res.read_to_string(&mut tmp_string).unwrap().to_string();
    //println!("{}", format!("{:?}", tmp_string));
    Ok(tmp_string)
}
/// pub fn relays_offline() -> Result<String, ascii::AsciiChar>
pub fn relays_offline() -> Result<String, ascii::AsciiChar> {
    let url = Url::parse("https://api.nostr.watch/v1/offline").unwrap();
    let mut res = reqwest::blocking::get(url).unwrap();

    let mut tmp_string = String::new();
    res.read_to_string(&mut tmp_string).unwrap().to_string();
    //println!("{}", format!("{:?}", tmp_string));
    Ok(tmp_string)
}
