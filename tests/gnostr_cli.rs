use anyhow::Result;
use test_utils::{relay::Relay, CliTester, TEST_KEY_1_NSEC, TEST_PASSWORD};
use tokio::runtime::Handle;

#[tokio::test]
async fn test_gnostr_help() -> Result<()> {
    let mut p = CliTester::new(vec!["gnostr", "--help"]);
    p.expect_eventually("gnostr:a git+nostr workflow utility")?;
    p.expect_end_eventually()?;
    Ok(())
}

#[tokio::test]
async fn test_generate_keypair() -> Result<()> {
    let mut p = CliTester::new(vec!["gnostr", "generate-keypair"]);
    let output = p.expect_end_eventually()?;
    assert!(output.contains("private_key"));
    assert!(output.contains("public_key"));
    Ok(())
}

#[tokio::test]
async fn test_convert_key_hex_to_bech32() -> Result<()> {
    let mut p = CliTester::new(vec![
        "gnostr",
        "convert-key",
        "--key",
        "f53e4bcd7a9cdef049cf6467d638a1321958acd3b71eb09823fd6fadb023d768",
        "--prefix",
        "npub",
    ]);
    let output = p.expect_end_eventually()?;
    assert!(output.contains("npub175lyhnt6nn00qjw0v3navw9pxgv43txnku0tpxprl4h6mvpr6a5qlphudg"));
    Ok(())
}

#[tokio::test]
async fn test_convert_key_bech32_to_hex() -> Result<()> {
    let mut p = CliTester::new(vec![
        "gnostr",
        "convert-key",
        "--key",
        "npub175lyhnt6nn00qjw0v3navw9pxgv43txnku0tpxprl4h6mvpr6a5qlphudg",
        "--to-hex",
    ]);
    let output = p.expect_end_eventually()?;
    assert!(output.contains("f53e4bcd7a9cdef049cf6467d638a1321958acd3b71eb09823fd6fadb023d768"));
    Ok(())
}

#[tokio::test]
async fn test_note_broadcast() -> Result<()> {
    let port = 8055;
    let mut relay = Relay::new(port, None, None);
    let relay_handle = Handle::current().spawn(async move { relay.listen_until_close().await });

    let mut p = CliTester::new(vec![
        "gnostr",
        "note",
        "--nsec",
        TEST_KEY_1_NSEC,
        "--password",
        TEST_PASSWORD,
        "--relays",
        &format!("ws://localhost:{}", port),
        "--content",
        "Hello Nostr!",
    ]);
    let output = p.expect_end_eventually()?;
    assert!(output.contains("id"));

    relay_handle.await??;
    Ok(())
}

#[tokio::test]
async fn test_set_metadata() -> Result<()> {
    let port = 8056;
    let mut relay = Relay::new(port, None, None);
    let relay_handle = Handle::current().spawn(async move { relay.listen_until_close().await });

    let mut p = CliTester::new(vec![
        "gnostr",
        "set-metadata",
        "--nsec",
        TEST_KEY_1_NSEC,
        "--password",
        TEST_PASSWORD,
        "--relays",
        &format!("ws://localhost:{}", port),
        "--name",
        "TestUser",
        "--about",
        "This is a test user",
    ]);
    let output = p.expect_end_eventually()?;
    assert!(output.contains("id"));

    relay_handle.await??;
    Ok(())
}
