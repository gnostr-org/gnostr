use crate::blockheight::{blockheight_async, blockheight_sync};
use log::debug;
use reqwest;
use reqwest::Url;
use std::io::Read;
use std::time::SystemTime;

/// pub fn wobble() -> Result<f64, ascii::AsciiChar>
///
pub fn wobble() -> Result<f64, ascii::AsciiChar> {
    wobble_sync()
}
/// pub fn wobble_sync() -> Result<f64, ascii::AsciiChar>
///
pub fn wobble_sync() -> Result<f64, ascii::AsciiChar> {
    //! wobble = utc_secs / blockheight
    let since_the_epoch = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("get millis error");
    let seconds = since_the_epoch.as_secs();
    let subsec_millis = since_the_epoch.subsec_millis() as u64;
    let _now_millis = seconds * 1000 + subsec_millis;
    debug!("now millis: {}", seconds * 1000 + subsec_millis);
    let blockheight = blockheight_sync();
    let tmp_u64 = blockheight.parse::<u64>().unwrap_or(0);
    let wobble = seconds as f64 % tmp_u64 as f64;
    return Ok(wobble.floor());
}
/// pub fn wobble_millis_sync() -> Result<f64, ascii::AsciiChar>
///
pub fn wobble_millis_sync() -> Result<f64, ascii::AsciiChar> {
    //! wobble = utc_secs / blockheight
    let since_the_epoch = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("get millis error");
    let seconds = since_the_epoch.as_secs();
    let subsec_millis = since_the_epoch.subsec_millis() as u64;
    let now_millis = seconds * 1000 + subsec_millis;
    debug!("now millis: {}", seconds * 1000 + subsec_millis);
    let blockheight = blockheight_sync();
    let tmp_u64 = blockheight.parse::<u64>().unwrap_or(0);
    //gnostr-chat uses millis
    let wobble = now_millis as f64 % tmp_u64 as f64;
    return Ok(wobble.floor());
}
/// pub async fn wobble_async() -> Result<f64, ascii::AsciiChar>
///
pub async fn wobble_async() -> Result<f64, ascii::AsciiChar> {
    //! wobble = utc_secs / blockheight
    let since_the_epoch = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("get millis error");
    let seconds = since_the_epoch.as_secs();
    let subsec_millis = since_the_epoch.subsec_millis() as u64;
    let _now_millis = seconds * 1000 + subsec_millis;
    debug!("now millis: {}", seconds * 1000 + subsec_millis);
    let blockheight = blockheight_async();
    let tmp_u64 = blockheight.await.parse::<u64>().unwrap_or(0);
    let wobble = seconds as f64 % tmp_u64 as f64;
    return Ok(wobble.floor());
}
/// pub fn wobble_millis_async() -> Result<f64, ascii::AsciiChar>
///
pub async fn wobble_millis_async() -> Result<f64, ascii::AsciiChar> {
    //! wobble = utc_secs / blockheight
    let since_the_epoch = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("get millis error");
    let seconds = since_the_epoch.as_secs();
    let subsec_millis = since_the_epoch.subsec_millis() as u64;
    let now_millis = seconds * 1000 + subsec_millis;
    debug!("now millis: {}", seconds * 1000 + subsec_millis);
    let blockheight = blockheight_async().await;
    let tmp_u64 = blockheight.parse::<u64>().unwrap_or(0);
    //gnostr-chat uses millis
    let wobble = now_millis as f64 % tmp_u64 as f64;
    return Ok(wobble.floor());
}
