use futures::{SinkExt, StreamExt};
use log::{info, warn};
use serde_json::{json, Map};
use std::io::{Error as IoError, ErrorKind};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;

pub mod cli;
pub mod forms;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Config {
    host: String,
    port: u16,
    use_tls: bool,
    retries: u8,
    authors: String,
    ids: String,
    limit: i32,
    generic: (String, String),
    hashtag: String,
    mentions: String,
    references: String,
    kinds: String,
    search: (String, String),
}

#[derive(Debug, Default)]
pub struct ConfigBuilder {
    host: Option<String>,
    port: Option<u16>,
    use_tls: bool,
    retries: u8,
    authors: Option<String>,
    ids: Option<String>,
    limit: Option<i32>,
    generic: Option<(String, String)>,
    hashtag: Option<String>,
    mentions: Option<String>,
    references: Option<String>,
    kinds: Option<String>,
    search: Option<(String, String)>,
}
impl ConfigBuilder {
    pub fn new() -> Self {
        ConfigBuilder {
            host: None,
            port: None,
            use_tls: false,
            retries: 0,
            authors: None,
            ids: None,
            limit: None,
            generic: None,
            hashtag: None,
            mentions: None,
            references: None,
            kinds: None,
            search: None,
        }
    }
    pub fn host(mut self, host: &str) -> Self {
        self.host = Some(host.to_string());
        self
    }
    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }
    pub fn use_tls(mut self, use_tls: bool) -> Self {
        self.use_tls = use_tls;
        self
    }
    pub fn retries(mut self, retries: u8) -> Self {
        self.retries = retries;
        self
    }
    pub fn authors(mut self, authors: &str) -> Self {
        self.authors = Some(authors.to_string());
        self
    }
    pub fn ids(mut self, ids: &str) -> Self {
        self.ids = Some(ids.to_string());
        self
    }
    pub fn limit(mut self, limit: i32) -> Self {
        self.limit = Some(limit);
        self
    }
    //pub fn generic(mut self, generic: &(&str, &str), tag: &str, val: &str) -> Self {
    pub fn generic(mut self, tag: &str, val: &str) -> Self {
        //self.generic = Some(("".to_string(), "".to_string()));
        self.generic = Some((tag.to_string(), val.to_string()));
        self
    }
    pub fn hashtag(mut self, hashtag: &str) -> Self {
        self.hashtag = Some(hashtag.to_string());
        self
    }
    pub fn mentions(mut self, mentions: &str) -> Self {
        self.mentions = Some(mentions.to_string());
        self
    }
    pub fn references(mut self, references: &str) -> Self {
        self.references = Some(references.to_string());
        self
    }
    pub fn kinds(mut self, kinds: &str) -> Self {
        self.kinds = Some(kinds.to_string());
        self
    }
    pub fn search(mut self, element: &str, search: &str) -> Self {
        self.search = Some((element.to_string(), search.to_string()));
        self
    }
    pub fn build(self) -> Result<Config, String> {
        Ok(Config {
            host: self.host.ok_or("Missing host")?,
            port: self.port.ok_or("Missing port")?,
            use_tls: self.use_tls,
            retries: self.retries,
            authors: self.authors.ok_or("")?,
            ids: self.ids.ok_or("")?,
            limit: self.limit.ok_or("")?,
            generic: self.generic.ok_or("")?,
            hashtag: self.hashtag.ok_or("")?,
            mentions: self.mentions.ok_or("")?,
            references: self.references.ok_or("")?,
            kinds: self.kinds.ok_or("")?,
            search: self.search.ok_or("")?,
        })
    }
}
pub async fn send(
    query_string: String,
    relay_url: Vec<Url>,
    limit: Option<i32>,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let requested_limit = limit.unwrap_or(1);
    if requested_limit < 0 {
        return Err(Box::new(IoError::new(
            ErrorKind::InvalidInput,
            format!("Query limit must be >= 0, got {requested_limit}"),
        )));
    }
    if requested_limit == 0 {
        return Ok(vec![]);
    }

    let relays = websocket_relays(relay_url);
    let relay_count = relays.len();
    if relays.is_empty() {
        return Err(Box::new(IoError::new(
            ErrorKind::InvalidInput,
            "No websocket relays provided",
        )));
    }

    let mut last_error: Option<Box<dyn std::error::Error>> = None;

    for relay in relays {
        match send_to_relay(query_string.clone(), &relay, requested_limit).await {
            Ok(result) => return Ok(result),
            Err(err) => {
                warn!("relay query failed for {}: {}", relay, err);
                last_error = Some(err);
            }
        }
    }

    if let Some(err) = last_error {
        return Err(Box::new(IoError::other(format!(
            "Failed to query {relay_count} websocket relays. Last error: {err}"
        ))));
    }

    Err(Box::new(IoError::other(format!(
        "Failed to query {relay_count} websocket relays"
    ))))
}

fn websocket_relays(relays: Vec<Url>) -> Vec<Url> {
    relays
        .into_iter()
        .filter(|relay| matches!(relay.scheme(), "ws" | "wss"))
        .collect()
}

async fn send_to_relay(
    query_string: String,
    relay: &Url,
    limit: i32,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let (ws_stream, _) = connect_async(relay.as_str()).await?;
    let (mut write, mut read) = ws_stream.split();
    write.send(Message::Text(query_string.into())).await?;
    let mut count: i32 = 0;
    let mut vec_result: Vec<String> = vec![];
    while let Some(message) = read.next().await {
        let data = message?;
        if count >= limit {
            //std::process::exit(0);
            return Ok(vec_result);
        }
        if let Message::Text(text) = data {
            //print!("{text}");
            vec_result.push(text.to_string());
            count += 1;
        }
    }
    Ok(vec_result)
}

#[allow(clippy::too_many_arguments)]
pub fn build_gnostr_query(
    authors: Option<&str>,
    ids: Option<&str>,
    limit: Option<i32>,
    generic: Option<(&str, &str)>,
    hashtag: Option<&str>,
    mentions: Option<&str>,
    references: Option<&str>,
    kinds: Option<&str>,
    search: Option<(&str, &str)>,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut filt = Map::new();
    let split_csv = |value: &str| {
        value
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect::<Vec<String>>()
    };
    let strip_kind_prefix = |value: &str| {
        let trimmed = value.trim();
        let lowered = trimmed.to_ascii_lowercase();
        for prefix in [
            "nip:", "nip=", "nip/", "nip ", "nips:", "nips=", "nips/", "nips ",
            "kind:", "kind=", "kind/", "kind ", "kinds:", "kinds=", "kinds/", "kinds ",
        ] {
            if lowered.starts_with(prefix) {
                return trimmed[prefix.len()..].trim().to_string();
            }
        }
        trimmed.to_string()
    };

    if let Some(authors) = authors {
        let _ = authors.len(); // Use the field to avoid dead_code warning
        filt.insert("authors".to_string(), json!(split_csv(authors)));
    }

    if let Some(ids) = ids {
        filt.insert("ids".to_string(), json!(split_csv(ids)));
    }

    if let Some(limit) = limit {
        filt.insert("limit".to_string(), json!(limit));
    }

    if let Some((tag, val)) = generic {
        let tag_with_hash = format!("#{}", tag.trim());
        filt.insert(tag_with_hash, json!(split_csv(val.trim())));
    }

    if let Some(hashtag) = hashtag {
        filt.insert("#t".to_string(), json!(split_csv(hashtag)));
    }

    if let Some(mentions) = mentions {
        filt.insert("#p".to_string(), json!(split_csv(mentions)));
    }

    if let Some(references) = references {
        filt.insert("#e".to_string(), json!(split_csv(references)));
    }

    if let Some(kinds) = kinds {
        let kind_ints: Result<Vec<i64>, _> = kinds
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| strip_kind_prefix(s).parse::<i64>())
            .collect();
        match kind_ints {
            Ok(kind_ints) => {
                filt.insert("kinds".to_string(), json!(kind_ints));
            }
            Err(_) => {
                return Err("Error parsing kinds. Ensure they are integers.".into());
            }
        }
    }
    if search.is_some() {
        let (field, value) = search.expect("REASON");
        filt.insert(field.trim().to_string(), json!(value.trim()));
    }

    println!("{:?}", filt);
    let q = json!(["REQ", "gnostr-query", filt]);
    info!("q={}", q);
    info!("{}", serde_json::to_string(&q)?);
    Ok(serde_json::to_string(&q)?)
}

#[cfg(test)]
mod tests {
    use super::build_gnostr_query;
    use super::send;
    use super::websocket_relays;
    use url::Url;

    #[test]
    fn build_gnostr_query_trims_kind_prefixes_and_whitespace() {
        let query = build_gnostr_query(
            None,
            None,
            Some(10),
            None,
            Some("gnostr "),
            None,
            None,
            Some("kind:1,nip=2, 3"),
            None,
        )
        .unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&query).unwrap();
        assert_eq!(parsed[2]["#t"], serde_json::json!(["gnostr"]));
        assert_eq!(parsed[2]["kinds"], serde_json::json!([1, 2, 3]));
    }

    #[test]
    fn websocket_relays_only_keeps_ws_and_wss_schemes() {
        let relays = vec![
            Url::parse("https://example.com").unwrap(),
            Url::parse("wss://relay.damus.io").unwrap(),
            Url::parse("ws://127.0.0.1:8080").unwrap(),
            Url::parse("ftp://example.com").unwrap(),
        ];

        let filtered = websocket_relays(relays);

        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].as_str(), "wss://relay.damus.io/");
        assert_eq!(filtered[1].as_str(), "ws://127.0.0.1:8080/");
    }

    #[tokio::test]
    async fn send_rejects_negative_limit() {
        let relays = vec![Url::parse("wss://relay.damus.io").unwrap()];
        let result = send("[]".to_string(), relays, Some(-1)).await;

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Query limit must be >= 0"),
        );
    }

    #[tokio::test]
    async fn send_errors_when_no_websocket_relays_exist() {
        let relays = vec![Url::parse("https://example.com").unwrap()];
        let result = send("[]".to_string(), relays, Some(1)).await;

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "No websocket relays provided",
        );
    }
}
