use futures::executor::block_on;
use url::Url;
use crate::get_relays_public;
pub async fn parse_urls(urls_str: &str) -> Result<Vec<String>, url::ParseError> {
    let mut urls: Vec<String> = Vec::new();
    let mut part = String::new();
    let mut collected = Vec::new();
    let mut char_iter = urls_str.chars();
    for url_str in urls_str.chars() {
        if char_iter.next() == Some('[') {}
        loop {
            match char_iter.next() {
                Some(']') => {
                    return std::result::Result::Ok(collected);
                }
                Some(',') | Some(' ') => {
                    if !part.is_empty() {
                        collected.push(part.clone());
                        print!("{}\n", format!("{}",part.clone().replace("\"", "")));
                        part = String::new();
                    }
                }
                x => part.push(x.expect("REASON")),
            }
        } //end loop
    }
    Ok(urls)
}

pub async fn print_watch_list() -> Result<Vec<String>, url::ParseError> {
    let vec_relay_list = parse_urls(&get_relays_public().unwrap().as_str()).await;
    vec_relay_list//.expect("REASON")
}
