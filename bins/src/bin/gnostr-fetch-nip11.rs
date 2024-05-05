// TEMPORARILY
#![allow(clippy::uninlined_format_args)]

use std::env;
use std::time::Duration;

use gnostr_types::RelayInformationDocument;
use reqwest::blocking::Client;
use reqwest::redirect::Policy;

fn main() {
    let mut args = env::args();
    let _ = args.next(); // program name
    let url = match args.next() {
        Some(u) => u,
        None => panic!("Usage: fetch_nip11 <RelayURL>"),
    };

    let (host, uri) = gnostr_bins::url_to_host_and_uri(&url);

    let scheme = match uri.scheme() {
        Some(refscheme) => match refscheme.as_str() {
            "wss" => "https",
            "ws" => "http",
            u => panic!("Unknown scheme {}", u),
        },
        None => panic!("Relay URL has no scheme."),
    };

    let client = Client::builder()
        .redirect(Policy::none())
        .connect_timeout(Some(Duration::from_secs(60)))
        .timeout(Some(Duration::from_secs(60)))
        .connection_verbose(true)
        .build()
        .unwrap();
    let response = client
        .get(format!("{}://{}", scheme, host))
        .header("Host", host)
        .header("Accept", "application/nostr+json")
        .send()
        .unwrap();
    let json = response.text().unwrap();
    if let Ok(rid) = serde_json::from_str::<RelayInformationDocument>(&json) {
        println!("{}", rid);
    } else {
        println!("INVALID DECODE");
        println!("{}", json);
    }
}
