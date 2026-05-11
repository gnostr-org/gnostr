use futures::{SinkExt, StreamExt};
use log::{debug, info};
use std::io;
use std::time::Duration;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tokio::time::timeout;
use url::Url;

use crate::message::{
    ClientMessage, EventKind, Filter, IdHex, PublicKeyHex, RelayMessage, SubscriptionId,
};

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
    info!("send: start relays={} limit={limit:?}", relay_url.len());
    debug!("send: query={query_string}");
    if relay_url.is_empty() {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::InvalidInput,
            "no valid relay URLs available",
        )));
    }

    let mut last_error: Option<String> = None;
    let mut last_success: Option<Vec<String>> = None;

    for relay in relay_url {
        debug!("send: trying relay {}", relay);
        match send_to_relay(&relay, &query_string, limit).await {
            Ok(result) => {
                if result.is_empty() {
                    debug!("relay {} returned no results; trying next relay", relay);
                    last_success = Some(result);
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

    if let Some(result) = last_success {
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
    let relay_timeout = Duration::from_secs(1);
    debug!("query relay: {relay}");
    debug!("send_to_relay: connecting {}", relay);
    let (ws_stream, _) = match timeout(relay_timeout, connect_async(relay.as_str())).await {
        Ok(Ok(result)) => result,
        Ok(Err(err)) => return Err(Box::new(err)),
        Err(_) => {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::TimedOut,
                format!("timed out connecting to relay {}", relay),
            )));
        }
    };
    let (mut write, mut read) = ws_stream.split();
    debug!("send_to_relay: connected {}", relay);
    println!("query relay connected: {relay}");
    write.send(Message::Text(query_string.into())).await?;
    debug!("send_to_relay: sent request {}", relay);
    println!("query relay sent request: {relay}");
    let mut vec_result: Vec<String> = vec![];
    let limit = limit.unwrap_or(i32::MAX);

    loop {
        let message = match timeout(relay_timeout, read.next()).await {
            Ok(message) => message,
            Err(_) => {
                debug!("Timed out waiting for relay {} response; moving on", relay);
                break;
            }
        };

        let Some(message) = message else {
            break;
        };

        let data = message?;
        if let Message::Text(text) = data {
            println!("query relay frame from {relay}: {text}");
            match serde_json::from_str::<RelayMessage>(&text) {
                Ok(RelayMessage::Event(_, _)) => {
                    vec_result.push(text.to_string());
                    debug!(
                        "send_to_relay: {} event received count={}",
                        relay,
                        vec_result.len()
                    );
                    if vec_result.len() as i32 >= limit {
                        return Ok(vec_result);
                    }
                    continue;
                }
                Ok(RelayMessage::Eose(_)) => {
                    debug!("send_to_relay: {} eose received; continuing", relay);
                    return Ok(vec_result);
                }
                Ok(_) => continue,
                Err(err) => {
                    debug!("send_to_relay: {} ignoring non-relay frame: {}", relay, err);
                }
            }
        }
    }

    debug!(
        "send_to_relay: done {} results={}",
        relay,
        vec_result.len()
    );
    Ok(vec_result)
}

fn build_filter(
    authors: Option<&str>,
    ids: Option<&str>,
    limit: Option<i32>,
    generic: Option<(&str, &str)>,
    hashtag: Option<&str>,
    mentions: Option<&str>,
    references: Option<&str>,
    kinds: Option<&str>,
) -> Result<Filter, Box<dyn std::error::Error>> {
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

    let mut filter = Filter::default();

    if let Some(authors) = authors {
        let _ = authors.len();
        filter.authors = split_csv(authors)
            .into_iter()
            .map(|author| PublicKeyHex::try_from_str(&author))
            .collect::<Result<Vec<_>, _>>()?;
    }

    if let Some(ids) = ids {
        filter.ids = split_csv(ids)
            .into_iter()
            .map(|id| IdHex::try_from_str(&id))
            .collect::<Result<Vec<_>, _>>()?;
    }

    if let Some(limit) = limit {
        filter.limit = Some(
            usize::try_from(limit).map_err(|_| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "limit must be greater than or equal to zero",
                )
            })?,
        );
    }

    if let Some((tag, val)) = generic {
        if let Some(tag) = tag.trim().chars().next() {
            filter.tags.insert(tag, split_csv(val.trim()));
        }
    }

    if let Some(hashtag) = hashtag {
        filter.tags.insert('t', split_csv(hashtag));
    }

    if let Some(mentions) = mentions {
        filter.tags.insert('p', split_csv(mentions));
    }

    if let Some(references) = references {
        filter.tags.insert('e', split_csv(references));
    }

    if let Some(kinds) = kinds {
        let kind_ints: Result<Vec<u32>, _> = kinds
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| strip_kind_prefix(s).parse::<u32>())
            .collect();
        filter.kinds = kind_ints?
            .into_iter()
            .map(EventKind::from)
            .collect();
    }

    Ok(filter)
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
    let filter = build_filter(authors, ids, limit, generic, hashtag, mentions, references, kinds)?;
    let query = ClientMessage::Req(SubscriptionId("gnostr-query".to_string()), vec![filter]);
    let wire = serde_json::to_string(&query)?;
    debug!("build_gnostr_query wire={wire}");
    info!("{wire}");
    Ok(wire)
}

#[cfg(test)]
mod tests {
    use super::build_filter;
    use super::build_gnostr_query;
    use super::send;
    use crate::message::{EventBuilder, EventKind, PrivateKey, RelayMessage, SubscriptionId};
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
        assert_eq!(parsed[0], serde_json::json!("REQ"));
        assert_eq!(parsed[1], serde_json::json!("gnostr-query"));
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

    #[test]
    fn build_filter_uses_shared_types() {
        let filter = build_filter(
            Some("ee11a5dff40c19a555f41fe42b48f00e618c91225622ae37b6c2bb67b76c4e49"),
            Some("3ab7b776cb547707a7497f209be799710ce7eb0801e13fd3c4e7b9261ac29084"),
            Some(25),
            None,
            Some("gnostr"),
            Some("ee11a5dff40c19a555f41fe42b48f00e618c91225622ae37b6c2bb67b76c4e49"),
            Some("5df64b33303d62afc799bdc36d178c07b2e1f0d824f31b7dc812219440affab6"),
            Some("kind:1,nip=1617"),
        )
        .unwrap();

        assert_eq!(filter.ids.len(), 1);
        assert_eq!(filter.authors.len(), 1);
        assert_eq!(filter.limit, Some(25));
        assert_eq!(filter.tags.get(&'t').unwrap(), &vec!["gnostr".to_string()]);
        assert_eq!(filter.kinds, vec![EventKind::TextNote, EventKind::Patches]);
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
                    .send(Message::Text(
                        serde_json::to_string(&RelayMessage::Event(
                            SubscriptionId("sub-1".to_string()),
                            Box::new(
                                EventBuilder::text_note("hello relay".to_string())
                                    .to_event(&PrivateKey::generate())
                                    .unwrap(),
                            ),
                        ))
                        .unwrap()
                        .into(),
                    ))
                    .await
                    .unwrap();
                websocket
                    .send(Message::Text(
                        serde_json::to_string(&RelayMessage::Eose(SubscriptionId(
                            "sub-1".to_string(),
                        )))
                        .unwrap()
                        .into(),
                    ))
                    .await
                    .unwrap();
            }
        });

        let bad_relay = Url::parse("ws://127.0.0.1:1/")?;
        let good_relay = Url::parse(&format!("ws://{}", addr))?;
        let result = send("REQ".to_string(), vec![bad_relay, good_relay], Some(1))
            .await
            .expect("query should fall back to the healthy relay");

        assert_eq!(result.len(), 1);
        assert!(result[0].starts_with("[\"EVENT\""));
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
            websocket
                .send(Message::Text(
                    serde_json::to_string(&RelayMessage::Eose(SubscriptionId(
                        "sub-1".to_string(),
                    )))
                    .unwrap()
                    .into(),
                ))
                .await
                .unwrap();
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
            websocket
                .send(Message::Text(
                    serde_json::to_string(&RelayMessage::Eose(SubscriptionId(
                        "sub-1".to_string(),
                    )))
                    .unwrap()
                    .into(),
                ))
                .await
                .unwrap();
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
