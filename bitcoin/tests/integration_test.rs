use gnostr_bitcoin::*;

#[test]
fn test_integration_build_verack_message() {
    let verack_msg = build_verack_message().expect("Failed to build verack message");
    assert_eq!(&verack_msg[0..4], MAGIC_BYTES);
    assert_eq!(std::str::from_utf8(&verack_msg[4..10]).unwrap(), "verack");
    assert_eq!(verack_msg.len(), 24);
}

#[test]
fn test_integration_build_ping_message() {
    let nonce = [10, 20, 30, 40, 50, 60, 70, 80];
    let ping_msg = build_ping_message(nonce).expect("Failed to build ping message");
    assert_eq!(&ping_msg[0..4], MAGIC_BYTES);
    assert_eq!(std::str::from_utf8(&ping_msg[4..8]).unwrap(), "ping");
    assert_eq!(ping_msg.len(), 24 + 8); // Header (24) + Nonce (8)
    assert_eq!(&ping_msg[24..32], nonce);
}
