use crate::relay_metadata::Relay;
use futures::{stream, StreamExt};
use log::warn;
use reqwest::header::ACCEPT;

const CONCURRENT_REQUESTS: usize = 16;

pub fn websocket_http_url(url: &str) -> String {
    url.replace("wss://", "https://")
        .replace("ws://", "http://")
}

pub async fn fetch_relay_texts(
    relays: Vec<String>,
    client: &reqwest::Client,
    context: &str,
) -> Vec<Result<(String, String), reqwest::Error>> {
    stream::iter(relays)
        .map(|url| {
            let client = client.clone();
            let context = context.to_string();
            async move {
                let http_url = websocket_http_url(&url);
                let resp = client
                    .get(&http_url)
                    .header(ACCEPT, "application/nostr+json")
                    .send()
                    .await?;

                if !resp.status().is_success() {
                    warn!("{}: skipping {} due to HTTP {}", context, url, resp.status());
                    return Ok((url, String::new()));
                }

                let text = resp.text().await?;
                Ok((url, text))
            }
        })
        .buffer_unordered(CONCURRENT_REQUESTS)
        .collect::<Vec<Result<(String, String), reqwest::Error>>>()
        .await
}

pub fn parse_relay_metadata(json_string: &str) -> Result<Relay, serde_json::Error> {
    serde_json::from_str(json_string)
}
