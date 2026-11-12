use std::{env, time::SystemTime};

use log::debug;

use crate::blockheight::{blockheight_async, blockheight_sync};
/// pub fn weeble() -> Result<f64, ascii::AsciiChar>
pub fn weeble() -> Result<f64, ascii::AsciiChar> {
    weeble_sync()
}
/// pub fn weeble_sync() -> Result<f64, ascii::AsciiChar>
pub fn weeble_sync() -> Result<f64, ascii::AsciiChar> {
    // weeble = utc_secs / blockheight
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
        debug!("weeble_sync: blockheight=0, weeble=0");
        unsafe { env::set_var("WEEBLE", "0") };
        return Ok(0.0);
    }
    let weeble = seconds as f64 / tmp_u64 as f64;
    debug!("weeble_sync: blockheight={}, weeble={}", tmp_u64, weeble);
    unsafe { env::set_var("WEEBLE", weeble.to_string()) };
    Ok(weeble.floor())
}
/// pub fn weeble_millis_sync() -> Result<f64, ascii::AsciiChar>
pub fn weeble_millis_sync() -> Result<f64, ascii::AsciiChar> {
    // weeble = utc_secs / blockheight
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
        debug!("weeble_millis_sync: blockheight=0, weeble_millis=0");
        unsafe { env::set_var("WEEBLE_MILLIS", "0") };
        return Ok(0.0);
    }
    //gnostr-chat uses millis
    let weeble = now_millis as f64 / tmp_u64 as f64;
    debug!(
        "weeble_millis_sync: blockheight={}, weeble_millis={}",
        tmp_u64, weeble
    );
    unsafe { env::set_var("WEEBLE_MILLIS", weeble.clone().to_string()) };
    Ok(weeble.floor())
}
/// pub async fn weeble_async() -> Result<f64, ascii::AsciiChar>
pub async fn weeble_async() -> Result<f64, ascii::AsciiChar> {
    let since_the_epoch = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("get millis error");
    let seconds = since_the_epoch.as_secs();
    let subsec_millis = since_the_epoch.subsec_millis() as u64;
    let _now_millis = seconds * 1000 + subsec_millis;
    debug!("now millis: {}", seconds * 1000 + subsec_millis);
    let blockheight = blockheight_async().await;
    let tmp_u64 = blockheight.parse::<u64>().unwrap_or(0);
    if tmp_u64 == 0 {
        debug!("weeble_async: blockheight=0, weeble=0");
        unsafe { env::set_var("WEEBLE", "0") };
        return Ok(0.0);
    }
    let weeble = seconds as f64 / tmp_u64 as f64;
    debug!("weeble_async: blockheight={}, weeble={}", tmp_u64, weeble);
    unsafe { env::set_var("WEEBLE", weeble.to_string()) };
    Ok(weeble.floor())
}

/// pub fn weeble_millis_async() -> Result<f64, ascii::AsciiChar>
pub async fn weeble_millis_async() -> Result<f64, ascii::AsciiChar> {
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
        debug!("weeble_millis_async: blockheight=0, weeble_millis=0");
        unsafe { env::set_var("WEEBLE_MILLIS", "0") };
        return Ok(0.0);
    }
    //gnostr-chat uses millis
    let weeble = now_millis as f64 / tmp_u64 as f64;
    debug!(
        "weeble_millis_async: blockheight={}, weeble_millis={}",
        tmp_u64, weeble
    );
    unsafe { env::set_var("WEEBLE_MILLIS", weeble.to_string()) };
    Ok(weeble.floor())
}

#[cfg(test)]
mod test {
    use super::*;
    use serial_test::serial;

    #[test]
    #[ignore = "hits live block-height APIs over the network"]
    #[serial]
    fn weeble_sync_returns_floored_non_negative_value() {
        // Arrange / Act
        let weeble = weeble_sync().expect("weeble_sync");
        let millis = weeble_millis_sync().expect("weeble_millis_sync");

        // Assert: both are floored (integral) and non-negative
        assert!(weeble >= 0.0, "weeble must be non-negative, got {weeble}");
        assert_eq!(weeble, weeble.floor());
        assert!(
            millis >= 0.0,
            "weeble_millis must be non-negative, got {millis}"
        );
        assert_eq!(millis, millis.floor());
        // Assert: env side effects
        let _ = env::var("WEEBLE").expect("WEEBLE env var must be set");
        let _ = env::var("WEEBLE_MILLIS").expect("WEEBLE_MILLIS env var must be set");
    }

    #[ignore = "hits live block-height APIs over the network"]
    #[serial]
    #[tokio::test]
    async fn weeble_async_matches_sync_shape() {
        // Arrange / Act
        let weeble = weeble_async().await.expect("weeble_async");

        // Assert
        assert!(weeble >= 0.0);
        assert_eq!(weeble, weeble.floor());
    }
}
