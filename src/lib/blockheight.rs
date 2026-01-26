use std::env;

use crate::utils::{ureq_async, ureq_sync};

pub fn check_curl() {

    //println!("check_curl");
}

pub fn blockheight() -> Result<f64, ascii::AsciiChar> {
    let blockheight = match ureq_sync("https://mempool.space/api/blocks/tip/height".to_string()) {
        Ok(val) => val.parse::<u64>().unwrap_or(0) as f64,
        Err(_) => 0.0,
    };

    env::set_var("BLOCKHEIGHT", blockheight.to_string());
    Ok(blockheight)
}

pub async fn blockheight_async() -> String {
    let blockheight =
        match ureq_async("https://mempool.space/api/blocks/tip/height".to_string()).await {
            Ok(val) => val.to_string(),
            Err(_) => "0".to_string(),
        };
    env::set_var("BLOCKHEIGHT", blockheight.clone());
    blockheight
}
pub fn blockheight_sync() -> String {
    let blockheight = match ureq_sync("https://mempool.space/api/blocks/tip/height".to_string()) {
        Ok(val) => val.to_string(),
        Err(_) => "0".to_string(),
    };
    env::set_var("BLOCKHEIGHT", blockheight.clone());
    blockheight
}
