use std::env;

use reqwest;

/// Gets the current Bitcoin blockheight from mempool.space synchronously.
pub fn blockheight() -> Result<f64, ascii::AsciiChar> {
    let client = reqwest::blocking::Client::new();
    let blockheight = match client
        .get("https://mempool.space/api/blocks/tip/height")
        .send()
    {
        Ok(response) => match response.text() {
            Ok(text) => text.parse::<u64>().unwrap_or(0) as f64,
            Err(_) => 0.0,
        },
        Err(_) => 0.0,
    };

    env::set_var("BLOCKHEIGHT", blockheight.to_string());
    Ok(blockheight)
}

/// Gets the current Bitcoin blockheight from mempool.space asynchronously.
pub async fn blockheight_async() -> String {
    let client = reqwest::Client::new();
    let blockheight = match client
        .get("https://mempool.space/api/blocks/tip/height")
        .send()
        .await
    {
        Ok(response) => match response.text().await {
            Ok(text) => text.to_string(),
            Err(_) => "0".to_string(),
        },
        Err(_) => "0".to_string(),
    };
    env::set_var("BLOCKHEIGHT", blockheight.clone());
    blockheight
}
/// Gets the current Bitcoin blockheight from mempool.space synchronously (wrapper).
pub fn blockheight_sync() -> String {
    let client = reqwest::blocking::Client::new();
    let blockheight = match client
        .get("https://mempool.space/api/blocks/tip/height")
        .send()
    {
        Ok(response) => match response.text() {
            Ok(text) => text.to_string(),
            Err(_) => "0".to_string(),
        },
        Err(_) => "0".to_string(),
    };
    env::set_var("BLOCKHEIGHT", blockheight.clone());
    blockheight
}
