use std::io::Read;
use std::time::SystemTime;

use reqwest::Url;

use crate::get_blockheight;

pub fn check_curl() {

    //println!("check_curl");
}

pub fn wobble() -> Result<f64, ascii::AsciiChar> {
    let since_the_epoch = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("get millis error");
    let seconds = since_the_epoch.as_secs();
    let subsec_millis = since_the_epoch.subsec_millis() as u64;
    let _now_millis = seconds * 1000 + subsec_millis;
    //println!("now millis: {}", seconds * 1000 + subsec_millis);

    let _ = get_blockheight();
    let url = Url::parse("https://mempool.space/api/blocks/tip/height").unwrap();
    let mut res = reqwest::blocking::get(url).unwrap();

    let mut tmp_string = String::new();
    res.read_to_string(&mut tmp_string).unwrap();
    let tmp_u64 = tmp_string.parse::<u64>().unwrap_or(0);

    //TODO:impl gnostr-weeble_millis
    //let weeble = now_millis as f64 / tmp_u64 as f64;
    let wobble = seconds as f64 % tmp_u64 as f64;
    Ok(wobble.floor())
}
