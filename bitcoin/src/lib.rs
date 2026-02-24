pub mod ui;
pub mod p2p;
pub mod tor;
pub mod send_raw_tx;
//pub mod widget;
pub mod tx_ui;

// Re-exporting items from submodules to make them accessible at the crate root.
// This allows users to import them like `gnostr_bitcoin::p2p::connect_and_handshake`.

// Re-exports for p2p module
pub use p2p::connect_and_handshake;
pub use p2p::accept_and_handshake;
pub use p2p::read_message;
pub use p2p::build_version_message;
pub use p2p::build_verack_message;
pub use p2p::build_mempool_message;
pub use p2p::build_ping_message;
pub use p2p::build_pong_message;
pub use p2p::build_getaddr_message;
pub use p2p::build_getheaders_message;
pub use p2p::build_getdata_message;
pub use p2p::build_feefilter_message;
pub use p2p::build_sendcmpct_message;
pub use p2p::VarIntReader;

#[derive(Debug, Clone)]
pub struct ActivePeerState {
    pub inbound_traffic: u64,
    pub outbound_traffic: u64,
    pub connection_time: SystemTime,
    pub protocol_version: i32,
    pub user_agent: String,
    pub fee_filter: u64,
}

// Re-exports for tor module
pub use tor::initialize_tor_client;
pub use tor::establish_tor_stream;

// Re-exports for tx_ui module
pub use tx_ui::{TxApp, TxProgress, TxStatus, init_tui as init_tx_tui, restore_tui as restore_tx_tui};

use std::{
    fs::{self, File},
    io::{Read, Write},
    net::{TcpStream, ToSocketAddrs},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use log::{LevelFilter, debug, error, info, warn};
use sha2::{Digest, Sha256};
use simplelog::{CombinedLogger, Config, WriteLogger};

use std::io::{Write as IoWrite};

// --- Constants ---

/// Magic bytes for the Bitcoin P2P network (Mainnet).
/// These bytes prefix every valid Bitcoin P2P message.
pub const MAGIC_BYTES: [u8; 4] = [0xF9, 0xBE, 0xB4, 0xD9]; // Mainnet

/// Default port for the Bitcoin P2P network.
pub const DEFAULT_PORT: u16 = 8333;

/// Protocol version supported by this client.
pub const PROTOCOL_VERSION: i32 = 70016;

/// Services supported by this client.
/// Currently set to 1, indicating a node that supports basic services.
pub const SERVICES: u64 = 1;
use anyhow::Result;

/// List of DNS seeds for discovering initial Bitcoin peers.
/// These domains are used to get a list of active nodes on the Bitcoin network.
pub const DNS_SEEDS: &[&str] = &[
    "dnsseed.bluematt.me",
    "dnsseed.bitcoin.dashjr-list-of-p2p-nodes.us",
    "seed.bitcoinstats.com",
    "seed.btc.petertodd.net",
    "seed.bitcoin.sprovoost.nl",
    "dnsseed.emzy.de",
    "seed.bitcoin.wiz.biz",
    "seed.bitcoin.sipa.be",
    "seed.bitcoin.jonasschnelli.ch",
    "seed.mainnet.achownodes.xyz",
    "127.0.0.1",
];
use std::path::PathBuf;
use directories::ProjectDirs;

/// Initializes the logging system.
/// Creates a log directory if it doesn't exist and sets up a logger that writes
/// to a file.
pub fn init_logger(data_dir: Option<PathBuf>) -> Result<()> {
    let log_dir = if let Some(dir) = data_dir {
        dir
    } else if let Some(proj_dirs) = ProjectDirs::from("org", "gnostr", "gnostr") {
        let mut p = proj_dirs.data_dir().to_path_buf();
        p.push("bitcoin");
        p
    } else {
        let home_dir =
            dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        let gnostr_dir = home_dir.join(".gnostr");
        gnostr_dir.join("bitcoin")
    };
    
    let log_file_path = log_dir.join("gnostr-bitcoin.log");

    fs::create_dir_all(&log_dir)?;

    CombinedLogger::init(vec![WriteLogger::new(
        LevelFilter::Info,
        Config::default(),
        File::create(log_file_path)?,
    )])
    .map_err(anyhow::Error::from)
}

// --- Testing Module (FINAL FIXED STRUCTURE) ---

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use crate::p2p::encode_varint;

    // Mock Data for P2P tests (moved from original lib.rs)
    const MOCK_HEIGHT_LE: [u8; 4] = [0x60, 0x5c, 0x0c, 0x00]; // 810000
    const MOCK_USER_AGENT: &[u8] = b"/mock-test-client/";
    const MOCK_RELAY: [u8; 1] = [0x00];

    fn create_mock_version_payload_prefix() -> Vec<u8> {
        let mut payload_prefix = Vec::new();
        let now: i64 = 0; // Mock timestamp for consistency
        payload_prefix.extend_from_slice(&PROTOCOL_VERSION.to_le_bytes());
        payload_prefix.extend_from_slice(&SERVICES.to_le_bytes());
        payload_prefix.extend_from_slice(&now.to_le_bytes());
        payload_prefix.extend_from_slice(&[0u8; 26]); // addr_recv (placeholder)
        payload_prefix.extend_from_slice(&[0u8; 26]); // addr_from (placeholder)
        payload_prefix.extend_from_slice(&0u64.to_le_bytes()); // nonce (placeholder)
        payload_prefix
    }

    fn create_mock_verack_response() -> Vec<u8> {
        let payload = Vec::new();
        let mut msg = Vec::new();
        msg.extend_from_slice(&MAGIC_BYTES);
        let mut cmd = [0u8; 12];
        cmd[0..6].copy_from_slice(b"verack");
        msg.extend_from_slice(&cmd);
        msg.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        msg.extend_from_slice(&p2p::calculate_checksum(&payload)); // Use p2p::calculate_checksum
        msg
    }

    fn create_mock_version_response() -> Vec<u8> {
        let mut payload = create_mock_version_payload_prefix();
        payload.push(MOCK_USER_AGENT.len() as u8);
        payload.extend_from_slice(MOCK_USER_AGENT);
        payload.extend_from_slice(&MOCK_HEIGHT_LE);
        payload.extend_from_slice(&MOCK_RELAY);

        let mut msg = Vec::new();
        msg.extend_from_slice(&MAGIC_BYTES);
        let mut cmd = [0u8; 12];
        cmd[0..7].copy_from_slice(b"version");
        msg.extend_from_slice(&cmd);
        msg.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        msg.extend_from_slice(&p2p::calculate_checksum(&payload)); // Use p2p::calculate_checksum
        msg.extend_from_slice(&payload);
        msg
    }

    #[test]
    fn test_mock_seeder_handshake_and_height_check() -> Result<()> {
        let mock_version = create_mock_version_response();
        let mock_verack = create_mock_verack_response();

        let mut mock_response = Vec::new();
        mock_response.extend_from_slice(&mock_version);
        mock_response.extend_from_slice(&mock_verack);

        let mock_stream = Cursor::new(mock_response);

        // Use the p2p module's handshake function
        let height = p2p::perform_test_handshake(mock_stream)?;

        assert_eq!(height, 810080, "Block height extraction failed.");
        Ok(())
    }

    #[test]
    fn test_varint_decoding_robust() -> Result<()> {
        let payload_2b: [u8; 3] = [0xfd, 0x00, 0x01];
        assert_eq!(payload_2b.read_varint_and_advance(0)?, (256, 3));
        let payload_4b: [u8; 5] = [0xfe, 0x00, 0x00, 0x01, 0x00];
        assert_eq!(payload_4b.read_varint_and_advance(0)?, (65536, 5));
        Ok(())
    }

    #[test]
    fn test_varint_decoding_single_byte() -> Result<()> {
        let payload = [0x7b]; // Value 123
        assert_eq!(payload.read_varint_and_advance(0)?, (123, 1));
        Ok(())
    }

    #[test]
    fn test_varint_decoding_max_single_byte() -> Result<()> {
        let payload = [0xfc]; // Value 252
        assert_eq!(payload.read_varint_and_advance(0)?, (252, 1));
        Ok(())
    }

    #[test]
    fn test_varint_decoding_max_2_byte() -> Result<()> {
        let payload = [0xfd, 0xff, 0xff]; // Value 65535
        assert_eq!(payload.read_varint_and_advance(0)?, (65535, 3));
        Ok(())
    }

    #[test]
    fn test_varint_decoding_max_4_byte() -> Result<()> {
        let payload = [0xfe, 0xff, 0xff, 0xff, 0xff]; // Value 4294967295
        assert_eq!(payload.read_varint_and_advance(0)?, (4294967295, 5));
        Ok(())
    }

    #[test]
    fn test_varint_decoding_max_8_byte() -> Result<()> {
        let payload = [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]; // Value 18446744073709551615
        assert_eq!(
            payload.read_varint_and_advance(0)?,
            (18446744073709551615, 9)
        );
        Ok(())
    }

    #[test]
    fn test_varint_decoding_incomplete_2_byte() -> Result<()> {
        let payload = [0xfd, 0x00];
        let err = payload.read_varint_and_advance(0).unwrap_err();
        assert_eq!(err.to_string(), "Incomplete 2-byte varint.");
        Ok(())
    }

    #[test]
    fn test_varint_decoding_incomplete_4_byte() -> Result<()> {
        let payload = [0xfe, 0x00, 0x00, 0x01];
        let err = payload.read_varint_and_advance(0).unwrap_err();
        assert_eq!(err.to_string(), "Incomplete 4-byte varint.");
        Ok(())
    }

    #[test]
    fn test_varint_decoding_incomplete_8_byte() -> Result<()> {
        let payload = [0xff, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01];
        let err = payload.read_varint_and_advance(0).unwrap_err();
        assert_eq!(err.to_string(), "Incomplete 8-byte varint.");
        Ok(())
    }

    #[test]
    fn test_varint_decoding_offset_out_of_bounds() -> Result<()> {
        let payload = [0x01];
        let err = payload.read_varint_and_advance(1).unwrap_err();
        assert_eq!(err.to_string(), "VarInt read failed: Offset out of bounds.");
        Ok(())
    }

    #[test]
    fn test_encode_varint_single_byte() {
        assert_eq!(encode_varint(123), vec![0x7b]);
        assert_eq!(encode_varint(0xfc), vec![0xfc]);
    }

    #[test]
    fn test_encode_varint_2_bytes() {
        assert_eq!(encode_varint(253), vec![0xfd, 0xfd, 0x00]);
        assert_eq!(encode_varint(0xffff), vec![0xfd, 0xff, 0xff]);
    }

    #[test]
    fn test_encode_varint_4_bytes() {
        assert_eq!(encode_varint(0x10000), vec![0xfe, 0x00, 0x00, 0x01, 0x00]);
        assert_eq!(
            encode_varint(0xffffffff),
            vec![0xfe, 0xff, 0xff, 0xff, 0xff]
        );
    }

    #[test]
    fn test_encode_varint_8_bytes() {
        assert_eq!(
            encode_varint(0x100000000),
            vec![0xff, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00]
        );
        assert_eq!(
            encode_varint(0xffffffffffffffff),
            vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]
        );
    }

    #[test]
    fn test_mempool_message_encoding() -> Result<()> {
        let mempool_msg = p2p::build_mempool_message()?;
        assert_eq!(str::from_utf8(&mempool_msg[4..11])?, "mempool");
        assert_eq!(
            u32::from_le_bytes(mempool_msg[16..20].try_into().unwrap()),
            0
        );
        assert_eq!(mempool_msg.len(), 24);
        Ok(())
    }

    #[test]
    fn test_build_version_message() -> Result<()> {
        let (version_msg, payload_len) = p2p::build_version_message(crate::PROTOCOL_VERSION)?;

        assert_eq!(&version_msg[0..4], MAGIC_BYTES);
        assert_eq!(str::from_utf8(&version_msg[4..11])?, "version");
        assert_eq!(
            u32::from_le_bytes(version_msg[16..20].try_into().unwrap()),
            payload_len as u32
        );
        assert_eq!(version_msg.len(), 24 + payload_len);

        assert_eq!(
            u32::from_le_bytes(version_msg[24..28].try_into().unwrap()),
            PROTOCOL_VERSION as u32
        );
        Ok(())
    }

    #[test]
    fn test_build_verack_message() -> Result<()> {
        let verack_msg = p2p::build_verack_message()?;

        assert_eq!(&verack_msg[0..4], MAGIC_BYTES);
        assert_eq!(str::from_utf8(&verack_msg[4..10])?, "verack");
        assert_eq!(
            u32::from_le_bytes(verack_msg[16..20].try_into().unwrap()),
            0
        );
        assert_eq!(verack_msg.len(), 24);
        Ok(())
    }

    #[test]
    fn test_build_ping_message() -> Result<()> {
        let nonce = [1, 2, 3, 4, 5, 6, 7, 8];
        let ping_msg = p2p::build_ping_message(nonce)?;

        assert_eq!(&ping_msg[0..4], MAGIC_BYTES);
        assert_eq!(str::from_utf8(&ping_msg[4..8])?, "ping");
        assert_eq!(
            u32::from_le_bytes(ping_msg[16..20].try_into().unwrap()),
            8
        );
        assert_eq!(ping_msg.len(), 24 + 8);
        assert_eq!(&ping_msg[24..32], nonce);
        Ok(())
    }

    #[test]
    fn test_build_pong_message() -> Result<()> {
        let nonce = [8, 7, 6, 5, 4, 3, 2, 1];
        let pong_msg = p2p::build_pong_message(nonce)?;

        assert_eq!(&pong_msg[0..4], MAGIC_BYTES);
        assert_eq!(str::from_utf8(&pong_msg[4..8])?, "pong");
        assert_eq!(
            u32::from_le_bytes(pong_msg[16..20].try_into().unwrap()),
            8
        );
        assert_eq!(pong_msg.len(), 24 + 8);
        assert_eq!(&pong_msg[24..32], nonce);
        Ok(())
    }

    #[test]
    fn test_build_getaddr_message() -> Result<()> {
        let getaddr_msg = p2p::build_getaddr_message()?;

        assert_eq!(&getaddr_msg[0..4], MAGIC_BYTES);
        assert_eq!(str::from_utf8(&getaddr_msg[4..11])?, "getaddr");
        assert_eq!(
            u32::from_le_bytes(getaddr_msg[16..20].try_into().unwrap()),
            0
        );
        assert_eq!(getaddr_msg.len(), 24);
        Ok(())
    }

    #[test]
    fn test_build_getheaders_message() -> Result<()> {
        let locator_hashes = vec![[0u8; 32], [1u8; 32]];
        let stop_hash = [2u8; 32];
        let getheaders_msg = p2p::build_getheaders_message(locator_hashes.clone(), stop_hash)?;

        assert_eq!(&getheaders_msg[0..4], MAGIC_BYTES);
        assert_eq!(str::from_utf8(&getheaders_msg[4..14])?, "getheaders");

        let expected_payload_len = 4 + 1 + (2 * 32) + 32;
        assert_eq!(
            u32::from_le_bytes(getheaders_msg[16..20].try_into().unwrap()),
            expected_payload_len as u32
        );
        assert_eq!(getheaders_msg.len(), 24 + expected_payload_len);

        let mut offset = 24; // Start of payload
        assert_eq!(
            u32::from_le_bytes(getheaders_msg[offset..offset + 4].try_into().unwrap()),
            PROTOCOL_VERSION as u32
        );
        offset += 4;

        assert_eq!(getheaders_msg[offset], 2); // hash_count varint
        offset += 1;

        assert_eq!(&getheaders_msg[offset..offset + 32], &locator_hashes[0]);
        offset += 32;
        assert_eq!(&getheaders_msg[offset..offset + 32], &locator_hashes[1]);
        offset += 32;

        assert_eq!(&getheaders_msg[offset..offset + 32], &stop_hash);

        Ok(())
    }

    #[test]
    fn test_calculate_checksum() {
        let empty_payload = vec![];
        let expected_checksum_empty = [0x5d, 0xf6, 0xe0, 0xe2];
        assert_eq!(p2p::calculate_checksum(&empty_payload), expected_checksum_empty);

        let hello_world_payload = b"hello world".to_vec();
        let expected_checksum_hello_world = [0xbc, 0x62, 0xd4, 0xb8];
        assert_eq!(
            p2p::calculate_checksum(&hello_world_payload),
            expected_checksum_hello_world
        );

        let long_payload = vec![0; 100];
        let expected_checksum_long = [0x71, 0x81, 0x6d, 0xf1];
        assert_eq!(p2p::calculate_checksum(&long_payload), expected_checksum_long);
    }
}
