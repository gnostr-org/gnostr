use crate::utils::{ureq_async, ureq_sync};
use std::env;

pub fn blockhash() -> Result<String, ascii::AsciiChar> {
    let blockhash = match ureq_sync("https://mempool.space/api/blocks/tip/hash".to_string()) {
        Ok(val) => val,
        Err(_) => "0".to_string(),
    };
    env::set_var("BLOCKHASH", blockhash.clone());
    Ok(blockhash)
}

pub async fn blockhash_async() -> String {
    let blockhash = match ureq_async("https://mempool.space/api/blocks/tip/hash".to_string()).await {
        Ok(val) => val.to_string(),
        Err(_) => "0".to_string(),
    };
    env::set_var("BLOCKHASH", blockhash.clone());
    blockhash
}
pub fn blockhash_sync() -> String {
    match ureq_sync("https://mempool.space/api/blocks/tip/hash".to_string()) {
        Ok(val) => val.to_string(),
        Err(_) => String::new(),
    }
}
