use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::nostr::ureq_async;
use log::debug;

fn synthetic_blockheight() -> u64 {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    std::cmp::max(1, seconds / 600)
}

const BLOCKHEIGHT_URLS: [&str; 4] = [
    "https://bitcoin.gob.sv/api/blocks/tip/height",
    "https://mempool.space/api/blocks/tip/height",
    "https://blockstream.info/api/blocks/tip/height",
    "https://blockchain.info/q/getblockcount",
];

fn fetch_blockheight_sync() -> (Option<String>, u8) {
    for (index, url) in BLOCKHEIGHT_URLS.iter().enumerate() {
        match ureq::get(url).call() {
            Ok(response) => match response.into_string() {
                Ok(val) => return (Some(val), index as u8),
                Err(err) => debug!("blockheight_sync: failed to read {}: {:?}", url, err),
            },
            Err(err) => debug!("blockheight_sync: failed to fetch from {}: {:?}", url, err),
        }
    }
    (None, BLOCKHEIGHT_URLS.len() as u8)
}

async fn fetch_blockheight_async() -> (Option<String>, u8) {
    for (index, url) in BLOCKHEIGHT_URLS.iter().enumerate() {
        match ureq_async((*url).to_string()).await {
            Ok(val) => return (Some(val.trim().to_string()), index as u8),
            Err(err) => debug!("blockheight_async: failed to fetch from {}: {:?}", url, err),
        }
    }
    (None, BLOCKHEIGHT_URLS.len() as u8)
}

pub fn check_curl() {

    //println!("check_curl");
}

pub fn blockheight() -> Result<f64, ascii::AsciiChar> {
    let (raw_blockheight, status) = fetch_blockheight_sync();
    unsafe { env::set_var("BLOCKHEIGHT_STATUS", status.to_string()) };
    let blockheight = raw_blockheight
        .and_then(|val| val.trim().parse::<u64>().ok())
        .unwrap_or_else(synthetic_blockheight) as f64;

    debug!("blockheight: {}", blockheight);
    unsafe { env::set_var("BLOCKHEIGHT", blockheight.to_string()) };
    Ok(blockheight)
}

pub async fn blockheight_async() -> String {
    let (raw_blockheight, status) = fetch_blockheight_async().await;
    unsafe { env::set_var("BLOCKHEIGHT_STATUS", status.to_string()) };
    let blockheight = raw_blockheight
        .and_then(|val| val.trim().parse::<u64>().ok())
        .unwrap_or_else(synthetic_blockheight)
        .to_string();
    debug!("blockheight_async: {}", blockheight);
    unsafe { env::set_var("BLOCKHEIGHT", blockheight.clone()) };
    blockheight
}
pub fn blockheight_sync() -> String {
    let (raw_blockheight, status) = fetch_blockheight_sync();
    unsafe { env::set_var("BLOCKHEIGHT_STATUS", status.to_string()) };
    let blockheight = raw_blockheight
        .and_then(|val| val.trim().parse::<u64>().ok())
        .unwrap_or_else(synthetic_blockheight)
        .to_string();
    debug!("blockheight_sync: {}", blockheight);
    unsafe { env::set_var("BLOCKHEIGHT", blockheight.clone()) };
    blockheight
}

pub fn blockheight_status() -> u8 {
    env::var("BLOCKHEIGHT_STATUS")
        .ok()
        .and_then(|status| status.parse::<u8>().ok())
        .unwrap_or(0)
}

#[cfg(test)]
mod test {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn blockheight_status_defaults_to_zero_when_unset() {
        // Arrange
        unsafe { env::remove_var("BLOCKHEIGHT_STATUS") };

        // Act
        let status = blockheight_status();

        // Assert
        assert_eq!(status, 0);
    }

    #[test]
    #[serial]
    fn blockheight_status_parses_valid_value() {
        // Arrange
        unsafe { env::set_var("BLOCKHEIGHT_STATUS", "2") };

        // Act
        let status = blockheight_status();

        // Assert
        assert_eq!(status, 2);
        unsafe { env::remove_var("BLOCKHEIGHT_STATUS") };
    }

    #[test]
    #[serial]
    fn blockheight_status_falls_back_to_zero_on_garbage() {
        // Arrange
        unsafe { env::set_var("BLOCKHEIGHT_STATUS", "not-a-number") };

        // Act
        let status = blockheight_status();

        // Assert
        assert_eq!(status, 0);
        unsafe { env::remove_var("BLOCKHEIGHT_STATUS") };
    }

    #[test]
    #[serial]
    fn blockheight_status_falls_back_to_zero_on_out_of_range() {
        // Arrange: 300 does not fit in u8
        unsafe { env::set_var("BLOCKHEIGHT_STATUS", "300") };

        // Act
        let status = blockheight_status();

        // Assert
        assert_eq!(status, 0);
        unsafe { env::remove_var("BLOCKHEIGHT_STATUS") };
    }

    #[test]
    #[ignore = "hits live block-height APIs over the network"]
    #[serial]
    fn blockheight_returns_positive_height_and_sets_env() {
        // Arrange / Act
        let height = blockheight().expect("blockheight");

        // Assert
        assert!(height > 0.0, "blockheight must be positive, got {height}");
        let env_height: f64 = env::var("BLOCKHEIGHT")
            .expect("BLOCKHEIGHT env var must be set")
            .parse()
            .expect("BLOCKHEIGHT env var must parse as f64");
        assert_eq!(height, env_height);
    }
}
