use std::{env, time::SystemTime};

use log::debug;

use crate::blockheight::{blockheight_async, blockheight_sync};
/// pub fn wobble() -> Result<f64, ascii::AsciiChar>
pub fn wobble() -> Result<f64, ascii::AsciiChar> {
    wobble_sync()
}
/// pub fn wobble_sync() -> Result<f64, ascii::AsciiChar>
pub fn wobble_sync() -> Result<f64, ascii::AsciiChar> {
    // wobble = utc_secs % blockheight
    let since_the_epoch = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("get millis error");
    let seconds = since_the_epoch.as_secs();
    let subsec_millis = since_the_epoch.subsec_millis() as u64;
    let _now_millis = seconds * 1000 + subsec_millis;
    debug!("now millis: {}", seconds * 1000 + subsec_millis);
    let blockheight = blockheight_sync();
    let tmp_u64 = blockheight.parse::<u64>().unwrap_or(0);
    if tmp_u64 == 0 {
        debug!("wobble_sync: blockheight=0, wobble=0");
        unsafe { env::set_var("WOBBLE", "0") };
        return Ok(0.0);
    }
    let wobble = seconds as f64 % tmp_u64 as f64;
    debug!("wobble_sync: blockheight={}, wobble={}", tmp_u64, wobble);
    unsafe { env::set_var("WOBBLE", wobble.to_string()) };
    Ok(wobble.floor())
}
/// pub fn wobble_millis_sync() -> Result<f64, ascii::AsciiChar>
pub fn wobble_millis_sync() -> Result<f64, ascii::AsciiChar> {
    // wobble = utc_secs % blockheight
    let since_the_epoch = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("get millis error");
    let seconds = since_the_epoch.as_secs();
    let subsec_millis = since_the_epoch.subsec_millis() as u64;
    let now_millis = seconds * 1000 + subsec_millis;
    debug!("now millis: {}", seconds * 1000 + subsec_millis);
    let blockheight = blockheight_sync();
    let tmp_u64 = blockheight.parse::<u64>().unwrap_or(0);
    if tmp_u64 == 0 {
        debug!("wobble_millis_sync: blockheight=0, wobble_millis=0");
        unsafe { env::set_var("WOBBLE_MILLIS", "0") };
        return Ok(0.0);
    }
    //gnostr-chat uses millis
    let wobble = now_millis as f64 % tmp_u64 as f64;
    debug!("wobble_millis_sync: blockheight={}, wobble_millis={}", tmp_u64, wobble);
    unsafe { env::set_var("WOBBLE_MILLIS", wobble.to_string()) };
    Ok(wobble.floor())
}
/// pub async fn wobble_async() -> Result<f64, ascii::AsciiChar>
pub async fn wobble_async() -> Result<f64, ascii::AsciiChar> {
    // wobble = utc_secs / blockheight
    let since_the_epoch = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("get millis error");
    let seconds = since_the_epoch.as_secs();
    let subsec_millis = since_the_epoch.subsec_millis() as u64;
    let _now_millis = seconds * 1000 + subsec_millis;
    debug!("now millis: {}", seconds * 1000 + subsec_millis);
    let blockheight = blockheight_async();
    let tmp_u64 = blockheight.await.parse::<u64>().unwrap_or(0);
    if tmp_u64 == 0 {
        debug!("wobble_async: blockheight=0, wobble=0");
        unsafe { env::set_var("WOBBLE", "0") };
        return Ok(0.0);
    }
    let wobble = seconds as f64 % tmp_u64 as f64;
    debug!("wobble_async: blockheight={}, wobble={}", tmp_u64, wobble);
    unsafe { env::set_var("WOBBLE", wobble.to_string()) };
    Ok(wobble.floor())
}
/// pub fn wobble_millis_async() -> Result<f64, ascii::AsciiChar>
pub async fn wobble_millis_async() -> Result<f64, ascii::AsciiChar> {
    // wobble = utc_secs / blockheight
    let since_the_epoch = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("get millis error");
    let seconds = since_the_epoch.as_secs();
    let subsec_millis = since_the_epoch.subsec_millis() as u64;
    let now_millis = seconds * 1000 + subsec_millis;
    debug!("now millis: {}", seconds * 1000 + subsec_millis);
    let blockheight = blockheight_async().await;
    let tmp_u64 = blockheight.parse::<u64>().unwrap_or(0);
    if tmp_u64 == 0 {
        debug!("wobble_millis_async: blockheight=0, wobble_millis=0");
        unsafe { env::set_var("WOBBLE_MILLIS", "0") };
        return Ok(0.0);
    }
    //gnostr-chat uses millis
    let wobble = now_millis as f64 % tmp_u64 as f64;
    debug!("wobble_millis_async: blockheight={}, wobble_millis={}", tmp_u64, wobble);
    unsafe { env::set_var("WOBBLE_MILLIS", wobble.to_string()) };
    Ok(wobble.floor())
}
