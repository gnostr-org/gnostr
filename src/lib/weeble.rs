use crate::blockheight::{blockheight_async, blockheight_sync};
use log::debug;
use std::time::SystemTime;

/// pub fn weeble() -> Result<f64, ascii::AsciiChar>
///
pub fn weeble() -> Result<f64, ascii::AsciiChar> {
    weeble_sync()
}
/// pub fn weeble_sync() -> Result<f64, ascii::AsciiChar>
///
pub fn weeble_sync() -> Result<f64, ascii::AsciiChar> {
    //! weeble = utc_secs / blockheight
    let since_the_epoch = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("get millis error");
    let seconds = since_the_epoch.as_secs();
    let subsec_millis = since_the_epoch.subsec_millis() as u64;
    let _now_millis = seconds * 1000 + subsec_millis;
    debug!("now millis: {}", seconds * 1000 + subsec_millis);
    let blockheight = blockheight_sync();
    let tmp_u64 = blockheight.parse::<u64>().unwrap_or(0);
    let weeble = seconds as f64 / tmp_u64 as f64;
    return Ok(weeble.floor());
}
/// pub fn weeble_millis_sync() -> Result<f64, ascii::AsciiChar>
///
pub fn weeble_millis_sync() -> Result<f64, ascii::AsciiChar> {
    //! weeble = utc_secs / blockheight
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
    let weeble = now_millis as f64 / tmp_u64 as f64;
    return Ok(weeble.floor());
}
/// pub async fn weeble_async() -> Result<f64, ascii::AsciiChar>
///
pub async fn weeble_async() -> Result<f64, ascii::AsciiChar> {
    //! weeble = utc_secs / blockheight
    let since_the_epoch = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("get millis error");
    let seconds = since_the_epoch.as_secs();
    let subsec_millis = since_the_epoch.subsec_millis() as u64;
    let _now_millis = seconds * 1000 + subsec_millis;
    debug!("now millis: {}", seconds * 1000 + subsec_millis);
    let blockheight = blockheight_async();
    let tmp_u64 = blockheight.await.parse::<u64>().unwrap_or(0);
    let weeble = seconds as f64 / tmp_u64 as f64;
    return Ok(weeble.floor());
}
/// pub fn weeble_millis_async() -> Result<f64, ascii::AsciiChar>
///
pub async fn weeble_millis_async() -> Result<f64, ascii::AsciiChar> {
    //! weeble = utc_secs / blockheight
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
    let weeble = now_millis as f64 / tmp_u64 as f64;
    return Ok(weeble.floor());
}
