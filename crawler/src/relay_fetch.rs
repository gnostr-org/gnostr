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
    debug!(
        "{}: fetching relay metadata from {} relays",
        context,
        relays.len()
    );
    stream::iter(relays)
        .map(|url| {
            let client = client.clone();
            let context = context.to_string();
            async move {
                let started = Instant::now();
                let http_url = websocket_http_url(&url);
                debug!("{}: GET {}", context, http_url);
                let resp = client
                    .get(&http_url)
                    .header(ACCEPT, "application/nostr+json")
                    .send()
                    .await?;
                let ping_ms = started.elapsed().as_millis() as u64;
                debug!("{}: {} responded in {}ms", context, url, ping_ms);

                if !resp.status().is_success() {
                    warn!("{}: skipping {} due to HTTP {}", context, url, resp.status());
                    return Ok((url, String::new(), ping_ms));
                }

                let text = resp.text().await?;
                let ping_ms = started.elapsed().as_millis() as u64;
                debug!(
                    "{}: {} body received ({} bytes)",
                    context,
                    url,
                    text.len()
                );
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

#[cfg(test)]
mod tests {
    use super::{parse_relay_metadata, websocket_http_url};

    #[test]
    fn websocket_http_url_converts_ws_schemes() {
        assert_eq!(
            websocket_http_url("wss://relay.example.com"),
            "https://relay.example.com"
        );
        assert_eq!(
            websocket_http_url("ws://relay.example.com"),
            "http://relay.example.com"
        );
    }

    #[test]
    fn parse_relay_metadata_preserves_supported_nips_and_ping() {
        let relay = parse_relay_metadata(
            r#"{"name":"Relay","supported_nips":[1,34],"ping_ms":42,"version":"1.0"}"#,
        )
        .unwrap();

        assert_eq!(relay.name.as_deref(), Some("Relay"));
        assert_eq!(relay.supported_nips, Some(vec![1, 34]));
        assert_eq!(relay.ping_ms, Some(42));
        assert_eq!(relay.version.as_deref(), Some("1.0"));
    }
}
