use futures::{SinkExt, StreamExt};
use log::{debug, info};
use serde_json::{json, Map};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;

pub mod cli;

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
    relay_url: Url,
    limit: Option<i32>,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    //println!("\n{}\n", query_string);
    //println!("\n{}\n", relay_url);
    //println!("\n{}\n", limit.unwrap());
    info!("\n{query_string}\n");
    info!("\n{relay_url}\n");
    info!("\n{}\n", limit.unwrap());
    let (ws_stream, _) = connect_async(relay_url).await?;
    let (mut write, mut read) = ws_stream.split();
    write.send(Message::Text(query_string)).await?;
    let mut count: i32 = 0;
    let mut vec_result: Vec<String> = vec![];
    while let Some(message) = read.next().await {
        let data = message?;
        if count >= limit.unwrap() {
            //std::process::exit(0);
            return Ok(vec_result);
        }
        if let Message::Text(text) = data {
            //print!("{text}");
            vec_result.push(text);
            count += 1;
        }
    }
    Ok(vec_result)
}

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

    if let Some(authors) = authors {
        filt.insert(
            "authors".to_string(),
            json!(authors.split(',').collect::<Vec<&str>>()),
        );
    }

    if let Some(ids) = ids {
        filt.insert(
            "ids".to_string(),
            json!(ids.split(',').collect::<Vec<&str>>()),
        );
    }

    if let Some(limit) = limit {
        filt.insert("limit".to_string(), json!(limit));
    }

    if let Some((tag, val)) = generic {
        let tag_with_hash = format!("#{tag}");
        filt.insert(tag_with_hash, json!(val.split(',').collect::<Vec<&str>>()));
    }

    if let Some(hashtag) = hashtag {
        filt.insert(
            "#t".to_string(),
            json!(hashtag.split(',').collect::<Vec<&str>>()),
        );
    }

    if let Some(mentions) = mentions {
        filt.insert(
            "#p".to_string(),
            json!(mentions.split(',').collect::<Vec<&str>>()),
        );
    }

    if let Some(references) = references {
        filt.insert(
            "#e".to_string(),
            json!(references.split(',').collect::<Vec<&str>>()),
        );
    }

    if let Some(kinds) = kinds {
        let kind_ints: Result<Vec<i64>, _> = kinds.split(',').map(|s| s.parse::<i64>()).collect();
        match kind_ints {
            Ok(kind_ints) => {
                filt.insert("kinds".to_string(), json!(kind_ints));
            }
            Err(_) => {
                return Err("Error parsing kinds. Ensure they are integers.".into());
            }
        }
    }
    if let Some(search) = search {
        filt.insert(
            "search".to_string(),
            json!(hashtag.expect("REASON")),//.split(',').collect::<Vec<&str>>()),
            //json!(hashtag.expect("REASON").split(',').collect::<Vec<&str>>()),
        );
    }
    //if let Some(search) = search {
    //    let search_terms: Result<Vec<String>, _> =
    //        search.split(',').map(|s| s.parse::<String>()).collect();
    //    match search_terms {
    //        Ok(search_terms) => {
    //            filt.insert("search".to_string(), json!(search_terms[0]));
    //        }
    //        Err(_) => {
    //            return Err("Unknonw error parsing search terns.".into());
    //        }
    //    }
    //}
    //TODO ["REQ", "", { "search": "orange" }, { "kinds": [1, 2], "search": "purple" }]
    //TODO ["REQ", "1", {"search": "nostr"}]

    println!("{:?}", filt);
    let q = json!(["REQ", "gnostr-query", filt]);
    info!("q={}", q);
    info!("{}", serde_json::to_string(&q)?);
    Ok(serde_json::to_string(&q)?)
}
