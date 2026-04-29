use crate::relay_metadata::Relay;
use futures::{stream, StreamExt};
use log::warn;
use reqwest::header::ACCEPT;
use std::time::Instant;

const CONCURRENT_REQUESTS: usize = 16;

pub fn websocket_http_url(url: &str) -> String {
    url.replace("wss://", "https://")
        .replace("ws://", "http://")
}

pub async fn fetch_relay_texts(
    relays: Vec<String>,
    client: &reqwest::Client,
    context: &str,
) -> Vec<Result<(String, String, u64), reqwest::Error>> {
    stream::iter(relays)
        .map(|url| {
            let client = client.clone();
            let context = context.to_string();
            async move {
                let started = Instant::now();
                let http_url = websocket_http_url(&url);
                let resp = client
                    .get(&http_url)
                    .header(ACCEPT, "application/nostr+json")
                    .send()
                    .await?;
                let ping_ms = started.elapsed().as_millis() as u64;

                if !resp.status().is_success() {
                    warn!("{}: skipping {} due to HTTP {}", context, url, resp.status());
                    return Ok((url, String::new(), ping_ms));
                }

                let text = resp.text().await?;
                let ping_ms = started.elapsed().as_millis() as u64;
                Ok((url, text, ping_ms))
            }
        })
        .buffer_unordered(CONCURRENT_REQUESTS)
        .collect::<Vec<Result<(String, String, u64), reqwest::Error>>>()
        .await
}

pub fn parse_relay_metadata(json_string: &str) -> Result<Relay, serde_json::Error> {
    serde_json::from_str(json_string)
}
