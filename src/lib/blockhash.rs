use crate::utils::{ureq_async, ureq_sync};
use std::io::Read;
use std::time::SystemTime;

use reqwest::Url;

pub fn check_curl() {

    //println!("check_curl");
}

pub fn blockhash() -> Result<String, ascii::AsciiChar> {
    let url = Url::parse("https://mempool.space/api/blocks/tip/hash").unwrap();
    let mut res = reqwest::blocking::get(url).unwrap();

    let mut blockhash = String::new();
    res.read_to_string(&mut blockhash).unwrap();
    Ok(blockhash)
}

pub async fn blockhash_async() -> String {
    ureq_async("https://mempool.space/api/blocks/tip/hash".to_string())
        .await
        .unwrap()
        .to_string()
}
pub fn blockhash_sync() -> String {
    ureq_sync("https://mempool.space/api/blocks/tip/hash".to_string())
        .unwrap()
        .to_string()
}
