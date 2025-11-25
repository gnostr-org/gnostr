use crate::utils::{ureq_async, ureq_sync};
use reqwest::Url;
use std::io::Read;

pub fn blockhash() -> Result<String, ascii::AsciiChar> {
    let blockhash = match reqwest::blocking::get("https://mempool.space/api/blocks/tip/hash") {
        Ok(mut res) => {
            let mut tmp_string = String::new();
            match res.read_to_string(&mut tmp_string) {
                Ok(_) => tmp_string,
                Err(_) => String::new(),
            }
        }
        Err(_) => String::new(),
    };
    Ok(blockhash)
}

pub async fn blockhash_async() -> String {
    match ureq_async("https://mempool.space/api/blocks/tip/hash".to_string()).await {
        Ok(val) => val.to_string(),
        Err(_) => String::new(),
    }
}
pub fn blockhash_sync() -> String {
    match ureq_sync("https://mempool.space/api/blocks/tip/hash".to_string()) {
        Ok(val) => val.to_string(),
        Err(_) => String::new(),
    }
}
