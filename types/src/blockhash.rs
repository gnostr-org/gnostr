use std::env;

use reqwest;

/// Gets the current Bitcoin blockhash from mempool.space synchronously.
pub fn blockhash() -> Result<String, ascii::AsciiChar> {
    let client = reqwest::blocking::Client::new();
    let blockhash = match client
        .get("https://mempool.space/api/blocks/tip/hash")
        .send()
    {
        Ok(response) => match response.text() {
            Ok(text) => text,
            Err(_) => "0".to_string(),
        },
        Err(_) => "0".to_string(),
    };
    env::set_var("BLOCKHASH", blockhash.clone());
    Ok(blockhash)
}

/// Gets the current Bitcoin blockhash from mempool.space asynchronously.
pub async fn blockhash_async() -> String {
    let client = reqwest::Client::new();
    let blockhash = match client
        .get("https://mempool.space/api/blocks/tip/hash")
        .send()
        .await
    {
        Ok(response) => match response.text().await {
            Ok(text) => text.to_string(),
            Err(_) => "0".to_string(),
        },
        Err(_) => "0".to_string(),
    };
    env::set_var("BLOCKHASH", blockhash.clone());
    blockhash
}
/// Gets the current Bitcoin blockhash from mempool.space synchronously (wrapper).
pub fn blockhash_sync() -> String {
    let client = reqwest::blocking::Client::new();
    match client
        .get("https://mempool.space/api/blocks/tip/hash")
        .send()
    {
        Ok(response) => match response.text() {
            Ok(text) => text.to_string(),
            Err(_) => String::new(),
        },
        Err(_) => String::new(),
    }
}
