use gnostr_bins::get_relays;
use reqwest::Url;
use std::io::Read;

use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    let since_the_epoch = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("get millis error");
    let seconds = since_the_epoch.as_secs();
    let subsec_millis = since_the_epoch.subsec_millis() as u64;
    let now_millis = seconds * 1000 + subsec_millis;
    //println!("now millis: {}", seconds * 1000 + subsec_millis);

    let _ = get_relays();
    //https://api.nostr.watch/v1/online
    let url = Url::parse("https://api.nostr.watch/v1/online").unwrap();
    let mut res = reqwest::blocking::get(url).unwrap();

    let mut tmp_string = String::new();
    res.read_to_string(&mut tmp_string).unwrap().to_string();
    //let tmp_u64 = tmp_string.parse::<u64>().unwrap_or(0);

    //TODO:impl gnostr-weeble_millis
    //let weeble = now_millis as f64 / tmp_u64 as f64;
    //let weeble = seconds as f64 / tmp_u64 as f64;
    //println!("{}", format!("{}", weeble.floor()));
    println!("{}", format!("{  }", tmp_string));
}
