use std::io::Read;
use std::time::SystemTime;

use reqwest::Url;

pub fn check_curl() {

    //println!("check_curl");
}

pub fn blockhash() -> Result<String, ascii::AsciiChar> {
    let since_the_epoch = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("get millis error");
    let seconds = since_the_epoch.as_secs();
    let subsec_millis = since_the_epoch.subsec_millis() as u64;
    let _now_millis = seconds * 1000 + subsec_millis;
    //println!("now millis: {}", seconds * 1000 + subsec_millis);

    //let bh = get_blockheight();
    //println!("{}",bh.unwrap());
    let url = Url::parse("https://mempool.space/api/blocks/tip/hash").unwrap();
    let mut res = reqwest::blocking::get(url).unwrap();

    let mut blockhash = String::new();
    res.read_to_string(&mut blockhash).unwrap();
    Ok(blockhash)
}
