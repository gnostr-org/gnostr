use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use log::debug;

use crate::{ureq_async, ureq_sync};

fn synthetic_blockheight() -> u64 {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    std::cmp::max(1, seconds / 600)
}

const BLOCKHEIGHT_URLS: [&str; 4] = [
    "https://bitcoin.gob.sv/api/blocks/tip/height",
    "https://mempool.space/api/blocks/tip/height",
    "https://blockstream.info/api/blocks/tip/height",
    "https://blockchain.info/q/getblockcount",
];

fn fetch_blockheight_sync() -> (Option<String>, u8) {
    for (index, url) in BLOCKHEIGHT_URLS.iter().enumerate() {
        match ureq_sync((*url).to_string()) {
            Ok(val) => return (Some(val), index as u8),
            Err(err) => {
                debug!("blockheight_sync: failed to fetch from {}: {:?}", url, err);
            }
        }
    }
    (None, BLOCKHEIGHT_URLS.len() as u8)
}

async fn fetch_blockheight_async() -> (Option<String>, u8) {
    for (index, url) in BLOCKHEIGHT_URLS.iter().enumerate() {
        match ureq_async((*url).to_string()).await {
            Ok(val) => return (Some(val), index as u8),
            Err(err) => {
                debug!("blockheight_async: failed to fetch from {}: {:?}", url, err);
            }
        }
    }
    (None, BLOCKHEIGHT_URLS.len() as u8)
}

pub fn check_curl() {

    //println!("check_curl");
}

pub fn blockheight() -> Result<f64, ascii::AsciiChar> {
    let (raw_blockheight, status) = fetch_blockheight_sync();
    unsafe { env::set_var("BLOCKHEIGHT_STATUS", status.to_string()) };
    let blockheight = raw_blockheight
        .and_then(|val| val.parse::<u64>().ok())
        .unwrap_or_else(synthetic_blockheight) as f64;

    debug!("blockheight: {}", blockheight);
    unsafe { env::set_var("BLOCKHEIGHT", blockheight.to_string()) };
    Ok(blockheight)
}

pub async fn blockheight_async() -> String {
    let (raw_blockheight, status) = fetch_blockheight_async().await;
    unsafe { env::set_var("BLOCKHEIGHT_STATUS", status.to_string()) };
    let blockheight = raw_blockheight
        .and_then(|val| val.parse::<u64>().ok())
        .unwrap_or_else(synthetic_blockheight)
        .to_string();
    debug!("blockheight_async: {}", blockheight);
    unsafe { env::set_var("BLOCKHEIGHT", blockheight.clone()) };
    blockheight
}
pub fn blockheight_sync() -> String {
    let (raw_blockheight, status) = fetch_blockheight_sync();
    unsafe { env::set_var("BLOCKHEIGHT_STATUS", status.to_string()) };
    let blockheight = raw_blockheight
        .and_then(|val| val.parse::<u64>().ok())
        .unwrap_or_else(synthetic_blockheight)
        .to_string();
    debug!("blockheight_sync: {}", blockheight);
    unsafe { env::set_var("BLOCKHEIGHT", blockheight.clone()) };
    blockheight
}

pub fn blockheight_status() -> u8 {
    env::var("BLOCKHEIGHT_STATUS")
        .ok()
        .and_then(|status| status.parse::<u8>().ok())
        .unwrap_or(0)
}
