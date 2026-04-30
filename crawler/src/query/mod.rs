use futures::{SinkExt, StreamExt};
use log::{debug, info};
use serde_json::{json, Map};
use std::io;
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
    if relay_url.is_empty() {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::InvalidInput,
            "no valid relay URLs available",
        )));
    }

    let mut last_error: Option<String> = None;
    let mut empty_result: Option<Vec<String>> = None;

    for relay in relay_url {
        match send_to_relay(&relay, &query_string, limit).await {
            Ok(result) => {
                if result.is_empty() {
                    debug!("relay {} returned no results; trying next relay", relay);
                    empty_result = Some(result);
                    continue;
                }
                return Ok(result);
            }
            Err(err) => {
                debug!("relay {} failed: {}", relay, err);
                last_error = Some(format!("{}: {}", relay, err));
            }
        }
    }

    if let Some(result) = empty_result {
        return Ok(result);
    }

    Err(Box::new(io::Error::new(
        io::ErrorKind::Other,
        format!(
            "failed to connect to any relay{}",
            last_error
                .as_deref()
                .map(|err| format!(" ({})", err))
                .unwrap_or_default()
        ),
    )))
}

async fn send_to_relay(
    relay: &Url,
    query_string: &str,
    limit: Option<i32>,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let (ws_stream, _) = connect_async(relay.as_str()).await?;
    let (mut write, mut read) = ws_stream.split();
    write.send(Message::Text(query_string.into())).await?;
    let mut count: i32 = 0;
    let mut vec_result: Vec<String> = vec![];
    let limit = limit.unwrap_or(i32::MAX);

    while let Some(message) = read.next().await {
        let data = message?;
        if count >= limit {
            return Ok(vec_result);
        }
        if let Message::Text(text) = data {
            vec_result.push(text.to_string());
            count += 1;
            if count >= limit {
                return Ok(vec_result);
            }
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
    _search: Option<(&str, &str)>,
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
            "nip:", "nip=", "nip/", "nip ", "nips:", "nips=", "nips/", "nips ", "kind:", "kind=",
            "kind/", "kind ", "kinds:", "kinds=", "kinds/", "kinds ",
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
    debug!("build_gnostr_query filter={:?}", filt);
    let q = json!(["REQ", "gnostr-query", filt]);
    info!("q={}", q);
    info!("{}", serde_json::to_string(&q)?);
    Ok(serde_json::to_string(&q)?)
}

#[cfg(test)]
mod tests {
    use super::build_gnostr_query;
    use super::send;
    use futures::{SinkExt, StreamExt};
    use tokio::net::TcpListener;
    use tokio_tungstenite::{accept_async, tungstenite::Message};
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
    fn build_gnostr_query_ignores_search_for_wire_filtering() {
        let query = build_gnostr_query(
            None,
            None,
            Some(10),
            None,
            None,
            None,
            None,
            Some("1"),
            Some(("search", "hello")),
        )
        .unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&query).unwrap();
        assert!(parsed[2].get("search").is_none());
    }

    #[tokio::test]
    async fn send_falls_back_to_later_relay() -> anyhow::Result<()> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;

        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut websocket = accept_async(stream).await.unwrap();
            if let Some(message) = websocket.next().await {
                let message = message.unwrap();
                assert!(matches!(message, Message::Text(_)));
                websocket
                    .send(Message::Text("{\"ok\":true}".to_string().into()))
                    .await
                    .unwrap();
            }
        });

        let bad_relay = Url::parse("ws://127.0.0.1:1/")?;
        let good_relay = Url::parse(&format!("ws://{}", addr))?;
        let result = send("REQ".to_string(), vec![bad_relay, good_relay], Some(1))
            .await
            .expect("query should fall back to the healthy relay");

        assert_eq!(result, vec!["{\"ok\":true}".to_string()]);
        server.await.unwrap();
        Ok(())
    }

    #[tokio::test]
    async fn send_returns_empty_result_when_all_relays_are_empty() -> anyhow::Result<()> {
        let listener_one = TcpListener::bind("127.0.0.1:0").await?;
        let addr_one = listener_one.local_addr()?;
        let server_one = tokio::spawn(async move {
            let (stream, _) = listener_one.accept().await.unwrap();
            let mut websocket = accept_async(stream).await.unwrap();
            if let Some(message) = websocket.next().await {
                assert!(matches!(message.unwrap(), Message::Text(_)));
            }
            websocket.close(None).await.unwrap();
        });

        let listener_two = TcpListener::bind("127.0.0.1:0").await?;
        let addr_two = listener_two.local_addr()?;
        let server_two = tokio::spawn(async move {
            let (stream, _) = listener_two.accept().await.unwrap();
            let mut websocket = accept_async(stream).await.unwrap();
            if let Some(message) = websocket.next().await {
                assert!(matches!(message.unwrap(), Message::Text(_)));
            }
            websocket.close(None).await.unwrap();
        });

        let relay_one = Url::parse(&format!("ws://{}", addr_one))?;
        let relay_two = Url::parse(&format!("ws://{}", addr_two))?;
        let result = send("REQ".to_string(), vec![relay_one, relay_two], Some(1))
            .await
            .expect("empty relays should still return Ok");

        assert!(result.is_empty());
        server_one.await.unwrap();
        server_two.await.unwrap();
        Ok(())
    }
}
