use std::env;

use log::debug;

use crate::{ureq_async, ureq_sync};

const BLOCKHASH_URLS: [&str; 4] = [
    "https://bitcoin.gob.sv/api/blocks/tip/hash",
    "https://mempool.space/api/blocks/tip/hash",
    "https://blockstream.info/api/blocks/tip/hash",
    "https://blockchain.info/q/latesthash",
];

fn fetch_blockhash_sync() -> Option<String> {
    for url in BLOCKHASH_URLS.iter() {
        match ureq_sync((*url).to_string()) {
            Ok(val) => return Some(val),
            Err(err) => debug!("blockhash_sync: failed to fetch from {}: {:?}", url, err),
        }
    }
    None
}

async fn fetch_blockhash_async() -> Option<String> {
    for url in BLOCKHASH_URLS.iter() {
        match ureq_async((*url).to_string()).await {
            Ok(val) => return Some(val.to_string()),
            Err(err) => debug!("blockhash_async: failed to fetch from {}: {:?}", url, err),
        }
    }
    None
}

pub fn blockhash() -> Result<String, ascii::AsciiChar> {
    let blockhash = fetch_blockhash_sync().unwrap_or_else(|| "0".to_string());
    debug!("blockhash: {}", blockhash);
    unsafe { env::set_var("BLOCKHASH", blockhash.clone()) };
    Ok(blockhash)
}

pub async fn blockhash_async() -> String {
    let blockhash = fetch_blockhash_async().await.unwrap_or_else(|| "0".to_string());
    debug!("blockhash_async: {}", blockhash);
    unsafe { env::set_var("BLOCKHASH", blockhash.clone()) };
    blockhash
}
pub fn blockhash_sync() -> String {
    let blockhash = fetch_blockhash_sync().unwrap_or_default();
    debug!("blockhash_sync: {}", blockhash);
    blockhash
}
