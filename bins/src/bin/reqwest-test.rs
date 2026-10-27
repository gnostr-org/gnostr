use std::time::Duration;
#[tokio::main]
async fn main() {
    let url = std::env::var("URL").unwrap_or_else(|_| "http://127.0.0.1:80/webui".to_string());
    println!("fetching {}", url);
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/145.0.0.0 Safari/537.36")
        .redirect(reqwest::redirect::Policy::limited(10))
        .default_headers({
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7".parse().unwrap());
            headers.insert("Accept-Language", "en-US,en;q=0.9".parse().unwrap());
            headers.insert("Accept-Encoding", "gzip, deflate, br".parse().unwrap());
            headers.insert("Upgrade-Insecure-Requests", "1".parse().unwrap());
            headers.insert("Sec-Fetch-Dest", "document".parse().unwrap());
            headers.insert("Sec-Fetch-Mode", "navigate".parse().unwrap());
            headers.insert("Sec-Fetch-Site", "none".parse().unwrap());
            headers.insert("Sec-Fetch-User", "?1".parse().unwrap());
            headers.insert("Connection", "keep-alive".parse().unwrap());
            headers
        })
        .build().unwrap();
    let start = std::time::Instant::now();
    match client.get(&url).send().await {
        Ok(resp) => println!("status={} final={} elapsed={:?}", resp.status(), resp.url(), start.elapsed()),
        Err(e) => println!("err={:#} elapsed={:?}", e, start.elapsed()),
    }
}
