
use gnostr_bitcoin::send_raw_tx::{
    build_version_msg, read_msg, send_msg, tor_v3_onion_from_pubkey,
};

use bitcoin::{
    p2p::{message::NetworkMessage, ServiceFlags},
    Transaction,
};
use tokio::runtime::Runtime;

// A valid, simple transaction for testing purposes
fn create_dummy_tx() -> Transaction {
    bitcoin::consensus::deserialize(&hex::decode("01000000010000000000000000000000000000000000000000000000000000000000000000ffffffff03020000ffffffff0100f2052a010000001976a914000000000000000000000000000000000000000088ac00000000").unwrap()).unwrap()
}

#[test]
fn test_build_version_msg() {
    let msg = build_version_msg();
    assert_eq!(msg.version, 70016);
    assert!(msg.services.has(ServiceFlags::NETWORK));
    assert!(msg.services.has(ServiceFlags::WITNESS));
    assert_eq!(msg.user_agent, "/Satoshi:27.0.0/");
    assert!(msg.relay);
}

#[test]
fn test_tor_v3_onion_from_pubkey() {
    // Test vector from Bitcoin Core: https://github.com/bitcoin/bitcoin/blob/master/src/test/netbase_tests.cpp#L383
    let pubkey_hex = "d22f42a01e3507b40a3625d2d21d55188a9a59c8431891a301318a43a87865f7";
    let pubkey_bytes = hex::decode(pubkey_hex).unwrap();
    let mut pubkey = [0u8; 32];
    pubkey.copy_from_slice(&pubkey_bytes);

    let expected_onion = "2ixufia6gud3icrwexjnehkvdcfjuwoiimmjdiybggfehkdymx34dkyd.onion";
    let actual_onion = tor_v3_onion_from_pubkey(&pubkey);

    assert_eq!(actual_onion, expected_onion);
}

#[tokio::test]
async fn test_msg_read_write() {
    let (mut client_stream, mut server_stream) = tokio::io::duplex(1024);

    let tx = create_dummy_tx();
    let network_msg = NetworkMessage::Tx(tx);

    // Send a message from the client side
    let msg_to_send = network_msg.clone();
    tokio::spawn(async move {
        send_msg(&mut client_stream, msg_to_send)
            .await
            .unwrap();
    });

    // Give the sender a moment to write
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;

    // Read it from the server side
    let raw_msg = read_msg(&mut server_stream).await.unwrap();

    assert_eq!(raw_msg.payload(), &network_msg);
}

#[test]
fn test_send_msg_encoding() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let mut buf = Vec::new();
        let version_msg = build_version_msg();
        let network_msg = NetworkMessage::Version(version_msg);

        send_msg(&mut buf, network_msg).await.unwrap();

        // 24 byte header
        // magic
        assert_eq!(&buf[0..4], bitcoin::p2p::Magic::BITCOIN.to_bytes());
        // command
        assert_eq!(&buf[4..16], b"version\0\0\0\0\0");
        // payload length
        let len = u32::from_le_bytes(buf[16..20].try_into().unwrap());
        assert_eq!(buf.len(), 24 + len as usize);
    });
}
