use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use tracing::{debug, error, info};
use std::fmt::Write as _;
use url::Url;

use crate::query::build_gnostr_query;
use crate::types::{
    Client, ContentEncryptionAlgorithm, Error, Event, EventKind, Filter, Id, Keys, PrivateKey,
    PublicKey, PublicKeyHex, RelayList, RelayListUsage,
};

#[cfg(test)]
const REAL_DM_RELAYS: &[&str] = &["wss://relay.damus.io", "wss://blossom.gnostr.cloud"];

#[async_trait]
pub trait DmClientTrait {
    async fn add_relays(&mut self, relays: Vec<String>) -> Result<(), Error>;
    fn build_nip44_direct_message_event(
        &self,
        recipient_pubkey: PublicKey,
        message: String,
    ) -> Result<Event, Error>;
    async fn send_event(&self, event: Event) -> Result<Id, Error>;
    async fn nip44_direct_message(
        &self,
        recipient_pubkey: PublicKey,
        message: String,
    ) -> Result<Id, Error>;
}

#[async_trait]
impl DmClientTrait for Client {
    async fn add_relays(&mut self, relays: Vec<String>) -> Result<(), Error> {
        self.add_relays(relays).await
    }

    fn build_nip44_direct_message_event(
        &self,
        recipient_pubkey: PublicKey,
        message: String,
    ) -> Result<Event, Error> {
        Client::build_nip44_direct_message_event(self, recipient_pubkey, message)
    }

    async fn send_event(&self, event: Event) -> Result<Id, Error> {
        self.send_event(event).await
    }

    async fn nip44_direct_message(
        &self,
        recipient_pubkey: PublicKey,
        message: String,
    ) -> Result<Id, Error> {
        self.nip44_direct_message(recipient_pubkey, message).await
    }
}

pub async fn dm_command(
    client: &impl DmClientTrait,
    recipient_pubkey: PublicKey,
    message: String,
    verbose: bool,
) -> Result<(), Error> {
    info!(
        "Sending NIP-44 direct message to {}",
        recipient_pubkey.as_hex_string()
    );
    debug!(
        recipient = %recipient_pubkey.as_hex_string(),
        message_len = message.len(),
        verbose,
        "dm command start"
    );

    if verbose {
        debug!("building dm event for verbose output");
        let event = client.build_nip44_direct_message_event(recipient_pubkey, message)?;
        debug!(
            event_id = %event.id,
            kind = u32::from(event.kind),
            content_len = event.content.len(),
            tags_len = event.tags.len(),
            "dm event built"
        );
        let event_json = serde_json::to_string(&event).map_err(|e| Error::Custom(e.into()))?;
        debug!(%event_json, "dm event json");
        println!("{}", event_json);
        debug!("sending verbose dm event");
        match client.send_event(event).await {
            Ok(event_id) => {
                debug!(event_id = %event_id, "dm send result: success");
                info!("Direct message sent successfully! Event ID: {}", event_id);
                println!("DM send result: success");
                println!("Direct message event id: {}", event_id);
                Ok(())
            }
            Err(e) => {
                debug!(error = %e, "dm send result: failure");
                error!("Failed to send direct message: {}", e);
                eprintln!("DM send result: failure: {}", e);
                Err(e)
            }
        }
    } else {
        debug!("sending dm through client.nip44_direct_message");
        match client.nip44_direct_message(recipient_pubkey, message).await {
            Ok(event_id) => {
                debug!(event_id = %event_id, "dm send result: success");
                info!("Direct message sent successfully! Event ID: {}", event_id);
                println!("DM send result: success");
                println!("Direct message event id: {}", event_id);
                Ok(())
            }
            Err(e) => {
                debug!(error = %e, "dm send result: failure");
                error!("Failed to send direct message: {}", e);
                eprintln!("DM send result: failure: {}", e);
                Err(e)
            }
        }
    }
}

pub async fn recipient_preferred_relays(
    keys: &Keys,
    recipient_pubkey: PublicKey,
    bootstrap_relays: Vec<String>,
) -> Result<Vec<String>, Error> {
    if bootstrap_relays.is_empty() {
        debug!(
            recipient = %recipient_pubkey.as_hex_string(),
            "DM bootstrap relay list is empty; skipping NIP-65 relay lookup"
        );
        return Ok(Vec::new());
    }

    debug!(
        recipient = %recipient_pubkey.as_hex_string(),
        bootstrap_relays = ?bootstrap_relays,
        "DM querying bootstrap relays for recipient NIP-65 relay list"
    );

    let mut client = Client::new(keys, crate::types::client::Options::new());
    client.add_relays(bootstrap_relays).await?;

    let recipient_pubkey_hex: PublicKeyHex = recipient_pubkey.into();
    let mut filter = Filter::new();
    filter.add_author(&recipient_pubkey_hex);
    filter.add_event_kind(EventKind::RelayList);
    filter.limit = Some(1);

    let events = client
        .get_events_of(vec![filter], Some(Duration::from_secs(10)))
        .await?;

    debug!(
        recipient = %recipient_pubkey_hex,
        event_count = events.len(),
        "DM bootstrap query completed"
    );

    let Some(event) = events.into_iter().max_by_key(|event| event.created_at) else {
        debug!(
            recipient = %recipient_pubkey_hex,
            "DM bootstrap query returned no NIP-65 relay list events"
        );
        return Ok(Vec::new());
    };

    let relay_list = RelayList::from_event(&event);
    let relays = relay_list_to_preferred_urls(&relay_list);
    debug!(
        recipient = %recipient_pubkey_hex,
        relays = ?relays,
        "DM extracted preferred relays from NIP-65 relay list"
    );
    Ok(relays)
}

pub async fn dm_inbox_command(
    nsec: Option<String>,
    recipient_pubkey: PublicKey,
    relay_args: Vec<String>,
    fallback_relay_args: Vec<String>,
    limit: Option<i32>,
    json: bool,
) -> anyhow::Result<()> {
    let inbox_limit = limit.unwrap_or(100);
    let recipient_pubkey_hex = recipient_pubkey.as_hex_string();
    let query_string = build_gnostr_query(
        None,
        None,
        Some(inbox_limit),
        None,
        None,
        Some(&recipient_pubkey_hex),
        None,
        Some("4,44"),
        None,
    )
    .map_err(|e| anyhow::anyhow!("Error building DM inbox query: {}", e))?;

    let explicit_relays = parse_relay_urls(&relay_args)?;
    let crawler_relays = parse_relay_urls(&crate::crawler::load_relays_or_bootstrap())?;
    let fallback_relays = parse_relay_urls(&fallback_relay_args)?;
    let relays_to_use = build_dm_inbox_relays(explicit_relays, crawler_relays, fallback_relays);

    debug!("DM inbox query relays: {:?}", relays_to_use);
    println!("DM inbox query relays:");
    for relay in &relays_to_use {
        println!("  {relay}");
    }

    let results = crate::query::send(query_string, relays_to_use, Some(inbox_limit))
        .await
        .map_err(|e| anyhow::anyhow!("Error in dm inbox query: {}", e))?;

    let private_key = parse_private_key(nsec)?;
    for result in results {
        println!("{}", format_query_frame(result, private_key.as_ref(), json)?);
    }

    Ok(())
}

fn relay_list_to_preferred_urls(relay_list: &RelayList) -> Vec<String> {
    let mut all_relays = Vec::new();
    let mut write_relays = Vec::new();

    for (relay_url, usage) in &relay_list.0 {
        let relay = relay_url.as_str().to_string();
        all_relays.push(relay.clone());
        if matches!(usage, RelayListUsage::Outbox | RelayListUsage::Both) {
            write_relays.push(relay);
        }
    }

    let mut relays = if write_relays.is_empty() {
        all_relays
    } else {
        write_relays
    };
    relays.sort();
    relays.dedup();
    relays
}

fn parse_private_key(nsec: Option<String>) -> anyhow::Result<Option<PrivateKey>> {
    match nsec {
        Some(value) if !value.trim().is_empty() => {
            if value.trim().starts_with("nsec") {
                Ok(Some(PrivateKey::try_from_bech32_string(&value)?))
            } else {
                Ok(Some(PrivateKey::try_from_hex_string(&value)?))
            }
        }
        _ => Ok(None),
    }
}

fn parse_relay_urls(relays: &[String]) -> anyhow::Result<Vec<Url>> {
    relays
        .iter()
        .map(|relay| Url::parse(relay).map_err(anyhow::Error::from))
        .collect()
}

fn build_dm_inbox_relays(
    explicit_relays: Vec<Url>,
    crawler_relays: Vec<Url>,
    fallback_relays: Vec<Url>,
) -> Vec<Url> {
    let mut relays = Vec::new();
    let local_relay = Url::parse("ws://127.0.0.1:8080").ok();

    if explicit_relays.is_empty() {
        if let Some(local_relay) = local_relay.clone() {
            relays.push(local_relay);
        }
    }

    for relay in explicit_relays
        .into_iter()
        .chain(crawler_relays)
        .chain(fallback_relays)
    {
        if !relays.iter().any(|existing| existing == &relay) {
            relays.push(relay);
        }
    }

    if let Some(local_relay) = local_relay {
        if !relays.iter().any(|existing| existing == &local_relay) {
            relays.push(local_relay);
        }
    }

    relays
}

fn format_query_frame(
    result: String,
    private_key: Option<&PrivateKey>,
    json: bool,
) -> anyhow::Result<String> {
    let Some(private_key) = private_key else {
        return Ok(result);
    };

    let mut frame: Value = serde_json::from_str(&result)?;
    let Some(items) = frame.as_array_mut() else {
        return Ok(result);
    };

    if items.len() < 3 || items.first().and_then(Value::as_str) != Some("EVENT") {
        return Ok(result);
    }

    let Some(event) = items.get_mut(2).and_then(Value::as_object_mut) else {
        return Ok(result);
    };

    let kind = event.get("kind").and_then(Value::as_u64);
    if !matches!(kind, Some(4) | Some(44)) {
        return Ok(result);
    }

    let Some(sender_pubkey_hex) = event.get("pubkey").and_then(Value::as_str) else {
        return Ok(result);
    };
    let Some(content) = event.get("content").and_then(Value::as_str) else {
        return Ok(result);
    };

    let sender_pubkey = PublicKey::try_from_hex_string(sender_pubkey_hex, true)?;
    match private_key.decrypt(&sender_pubkey, content) {
        Ok(decrypted) => {
            if json {
                event.insert("content".to_string(), Value::String(decrypted));
                Ok(serde_json::to_string_pretty(&frame)?)
            } else {
                Ok(format_decrypted_event(event, kind.unwrap_or_default(), &decrypted))
            }
        }
        Err(_) => Ok(result),
    }
}

fn format_decrypted_event(
    event: &serde_json::Map<String, Value>,
    kind: u64,
    decrypted: &str,
) -> String {
    let id = event.get("id").and_then(Value::as_str).unwrap_or("(unknown)");
    let pubkey = event
        .get("pubkey")
        .and_then(Value::as_str)
        .unwrap_or("(unknown)");
    let created_at = event
        .get("created_at")
        .and_then(Value::as_i64)
        .unwrap_or_default();
    let recipients: Vec<String> = event
        .get("tags")
        .and_then(Value::as_array)
        .map(|tags| {
            let mut recipients = Vec::new();
            for tag in tags {
                let Some(tag_items) = tag.as_array() else {
                    continue;
                };
                if tag_items.first().and_then(Value::as_str) == Some("p") {
                    if let Some(recipient) = tag_items.get(1).and_then(Value::as_str) {
                        recipients.push(recipient.to_string());
                    }
                }
            }
            recipients
        })
        .unwrap_or_default();

    let mut out = String::new();
    let _ = writeln!(out, "DM event kind {kind}");
    let _ = writeln!(out, "id: {id}");
    let _ = writeln!(out, "from: {pubkey}");
    let _ = writeln!(
        out,
        "to: {}",
        if recipients.is_empty() {
            "(unknown)".to_string()
        } else {
            recipients.join(", ")
        }
    );
    let _ = writeln!(out, "created_at: {created_at}");
    let _ = writeln!(out, "content:");
    let _ = writeln!(out, "{decrypted}");
    out
}

#[cfg(test)]
mod dm_tests {
    use base64::Engine;
    use std::collections::HashMap;
    use serial_test::serial;
    use tokio;

    use super::*;
    use crate::types::{
        client::{Client, Options},
        ContentEncryptionAlgorithm, EventBuilder, EventKind, Keys, PrivateKey, RelayList,
        RelayListUsage, Tag, UncheckedUrl,
    };

    fn log_relays(label: &str, relays: &[&str]) {
        println!("{label} relays: {}", relays.join(", "));
    }

    fn install_rustls_crypto_provider() {
        let _ = rustls::crypto::ring::default_provider().install_default();
    }

    fn default_test_npub() -> String {
        let recipient = PrivateKey(
            crate::git2::default_gnostr_private_key(),
            crate::types::KeySecurity::Weak,
        );
        recipient.public_key().as_bech32_string()
    }

    #[test]
    fn test_relay_list_to_preferred_urls_prefers_write_relays() {
        let signing_key = PrivateKey::try_from_hex_string(crate::test_utils::TEST_KEY_1_SK_HEX)
            .unwrap();
        let relay_list = RelayList::from_event(
            &EventBuilder::new(
                EventKind::RelayList,
                "".to_string(),
                vec![
                    Tag::new_relay(UncheckedUrl("ws://localhost:8053".to_string()), None),
                    Tag::new_relay(
                        UncheckedUrl("ws://localhost:8054".to_string()),
                        Some("read".to_string()),
                    ),
                    Tag::new_relay(
                        UncheckedUrl("ws://localhost:8055".to_string()),
                        Some("write".to_string()),
                    ),
                ],
            )
            .to_event(&signing_key)
            .unwrap(),
        );
        let relays = relay_list_to_preferred_urls(&relay_list);

        assert_eq!(
            relays,
            vec![
                "ws://localhost:8053".to_string(),
                "ws://localhost:8055".to_string(),
            ]
        );
    }

    #[test]
    fn test_relay_list_to_preferred_urls_falls_back_to_all_relays() {
        let mut relays = HashMap::new();
        relays.insert(
            crate::types::RelayUrl::try_from_str("ws://localhost:8054").unwrap(),
            RelayListUsage::Inbox,
        );
        relays.insert(
            crate::types::RelayUrl::try_from_str("ws://localhost:8055").unwrap(),
            RelayListUsage::Inbox,
        );
        let relay_list = RelayList(relays);
        let urls = relay_list_to_preferred_urls(&relay_list);

        assert_eq!(
            urls,
            vec![
                "ws://localhost:8054".to_string(),
                "ws://localhost:8055".to_string(),
            ]
        );
    }

    #[test]
    fn build_dm_inbox_relays_puts_explicit_relays_first() {
        let relays = build_dm_inbox_relays(
            vec![Url::parse("wss://explicit.example").unwrap()],
            vec![Url::parse("wss://crawler.example").unwrap()],
            vec![Url::parse("wss://fallback.example").unwrap()],
        );

        assert_eq!(
            relays,
            vec![
                Url::parse("wss://explicit.example").unwrap(),
                Url::parse("wss://crawler.example").unwrap(),
                Url::parse("wss://fallback.example").unwrap(),
                Url::parse("ws://127.0.0.1:8080").unwrap(),
            ]
        );
    }

    #[test]
    fn build_dm_inbox_relays_preserves_explicit_relay_order() {
        let relays = build_dm_inbox_relays(
            vec![
                Url::parse("wss://first.example").unwrap(),
                Url::parse("wss://second.example").unwrap(),
            ],
            vec![Url::parse("wss://crawler.example").unwrap()],
            vec![Url::parse("wss://fallback.example").unwrap()],
        );

        assert_eq!(
            &relays[..2],
            &[
                Url::parse("wss://first.example").unwrap(),
                Url::parse("wss://second.example").unwrap(),
            ]
        );
    }

    #[test]
    fn format_query_frame_decrypts_kind_44_when_nsec_is_available() -> anyhow::Result<()> {
        let sender_privkey = PrivateKey(
            secp256k1::SecretKey::from_slice(&[7_u8; 32]).unwrap(),
            crate::types::KeySecurity::Weak,
        );
        let recipient_privkey = PrivateKey(
            crate::git2::default_gnostr_private_key(),
            crate::types::KeySecurity::Weak,
        );
        let recipient_pubkey = recipient_privkey.public_key();
        let ciphertext = sender_privkey.encrypt(
            &recipient_pubkey,
            "secret inbox message",
            ContentEncryptionAlgorithm::Nip44v2,
        )?;
        let frame = serde_json::json!([
            "EVENT",
            "gnostr-query",
            {
                "kind": 44,
                "pubkey": sender_privkey.public_key().as_hex_string(),
                "content": ciphertext,
            }
        ]);

        let decrypted = format_query_frame(serde_json::to_string(&frame)?, Some(&recipient_privkey), false)?;
        assert!(decrypted.contains("DM event kind 44"));
        assert!(decrypted.contains("secret inbox message"));
        assert!(decrypted.contains("to:"));
        Ok(())
    }

    #[test]
    fn format_query_frame_decrypts_kind_44_as_json_when_requested() -> anyhow::Result<()> {
        let sender_privkey = PrivateKey(
            secp256k1::SecretKey::from_slice(&[8_u8; 32]).unwrap(),
            crate::types::KeySecurity::Weak,
        );
        let recipient_privkey = PrivateKey(
            crate::git2::default_gnostr_private_key(),
            crate::types::KeySecurity::Weak,
        );
        let recipient_pubkey = recipient_privkey.public_key();
        let ciphertext = sender_privkey.encrypt(
            &recipient_pubkey,
            "json inbox message",
            ContentEncryptionAlgorithm::Nip44v2,
        )?;
        let frame = serde_json::json!([
            "EVENT",
            "gnostr-query",
            {
                "kind": 4,
                "pubkey": sender_privkey.public_key().as_hex_string(),
                "content": ciphertext,
            }
        ]);

        let decrypted = format_query_frame(serde_json::to_string(&frame)?, Some(&recipient_privkey), true)?;
        let parsed: Value = serde_json::from_str(&decrypted)?;
        assert_eq!(parsed[2]["content"], "json inbox message");
        assert_eq!(parsed[2]["kind"], 4);
        Ok(())
    }

    struct FailingDmClient;

    #[async_trait]
    impl DmClientTrait for FailingDmClient {
        async fn add_relays(&mut self, _relays: Vec<String>) -> Result<(), Error> {
            Ok(())
        }

        fn build_nip44_direct_message_event(
            &self,
            _recipient_pubkey: PublicKey,
            _message: String,
        ) -> Result<Event, Error> {
            Err(Error::Custom("build not implemented for mock".into()))
        }

        async fn send_event(&self, _event: Event) -> Result<Id, Error> {
            Err(Error::Custom("send not implemented for mock".into()))
        }

        async fn nip44_direct_message(
            &self,
            _recipient_pubkey: PublicKey,
            _message: String,
        ) -> Result<Id, Error> {
            Err(Error::Custom(
                "Failed to send event to any configured relay.".into(),
            ))
        }
    }

    #[tokio::test]
    #[ignore]
    #[serial]
    async fn test_dm_command_success() {
        install_rustls_crypto_provider();
        // Setup real client
        let sender_privkey = PrivateKey::try_from_hex_string(
            "0000000000000000000000000000000000000000000000000000000000000001",
        )
        .unwrap();
        let sender_keys = Keys::new(sender_privkey);
        let mut client = Client::new(&sender_keys, Options::new());

        log_relays("test_dm_command_success", REAL_DM_RELAYS);
        client
            .add_relays(REAL_DM_RELAYS.iter().map(|relay| relay.to_string()).collect())
            .await
            .unwrap();

        // Create recipient public key
        let recipient_privkey = PrivateKey::try_from_hex_string(
            "0000000000000000000000000000000000000000000000000000000000000002",
        )
        .unwrap();
        let _recipient_keys = Keys::new(recipient_privkey);
        let recipient_pubkey = _recipient_keys.public_key();

        let message_content = "gnostr dm sub_command test!".to_string();

        // Call the function under test (this will now use the real
        // nip44_direct_message)
        let result = dm_command(
            &client,
            recipient_pubkey.clone(),
            message_content.clone(),
            false,
        )
        .await;

        // Assertions
        assert!(result.is_ok());
        let _event_id = result.unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_dm_command_success_bech32_recipient() {
        install_rustls_crypto_provider();
        // Setup real client
        let sender_privkey = PrivateKey::try_from_hex_string(
            "0000000000000000000000000000000000000000000000000000000000000001",
        )
        .unwrap();
        let _sender_keys = Keys::new(sender_privkey);
        let mut client = Client::new(&_sender_keys, Options::new());

        log_relays("test_dm_command_success_bech32_recipient", REAL_DM_RELAYS);
        client
            .add_relays(REAL_DM_RELAYS.iter().map(|relay| relay.to_string()).collect())
            .await
            .unwrap();

        // Create recipient public key from bech32 string
        let recipient_pubkey = PublicKey::try_from_bech32_string(&default_test_npub(), false)
            .unwrap();

        let message_content = "gnostr dm sub_command test with bech32 recipient!".to_string();

        // Call the function under test
        let result = dm_command(
            &client,
            recipient_pubkey.clone(),
            message_content.clone(),
            false,
        )
        .await;

        // Assertions
        assert!(result.is_ok());
        let _event_id = result.unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_dm_command_invalid_hex_pubkey() {
        let recipient_pubkey_str = "invalidhexpubkey";
        let recipient_pubkey_result = PublicKey::try_from_hex_string(recipient_pubkey_str, false);
        assert!(recipient_pubkey_result.is_err());
    }

    #[tokio::test]
    #[serial]
    async fn test_dm_command_failure() {
        install_rustls_crypto_provider();
        let recipient_pubkey = PublicKey::try_from_hex_string(
            "edfa27d49d2af37ee331e1225bb6ed1912c6d999281b36d8018ad99bc3573c29",
            false,
        )
        .unwrap();

        let message_content = "gnostr dm sub_command test may fail to encrypt!".to_string();

        let client = FailingDmClient;
        let result = dm_command(
            &client,
            recipient_pubkey.clone(),
            message_content.clone(),
            false,
        )
        .await;
        assert!(result.is_err());
        let actual_error = result.unwrap_err();
        eprintln!("Actual error: {}", actual_error);
        assert!(actual_error
            .to_string()
            .contains("Failed to send event to any configured relay."));
    }

    #[tokio::test]
    #[ignore]
    #[serial]
    async fn test_dm_command_decryption_success() {
        // Setup sender and receiver keypairs
        let sender_privkey = PrivateKey::try_from_hex_string(
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        )
        .unwrap();
        let sender_pubkey = sender_privkey.public_key();
        let sender_keys = Keys::new(sender_privkey.clone());
        let mut sender_client = Client::new(&sender_keys, Options::new());

        let recipient_privkey = PrivateKey::try_from_hex_string(
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        )
        .unwrap();
        let recipient_pubkey = recipient_privkey.public_key();
        let _recipient_keys = Keys::new(recipient_privkey.clone());

        sender_client
            .add_relays(REAL_DM_RELAYS.iter().map(|relay| relay.to_string()).collect())
            .await
            .unwrap();

        log_relays("test_dm_command_decryption_success", REAL_DM_RELAYS);

        let original_message = "This is a secret message!".to_string();

        // 1. Encrypt the message using dm_command
        let event_id_result = sender_client
            .nip44_direct_message(recipient_pubkey.clone(), original_message.clone())
            .await;

        assert!(event_id_result.is_ok());
        let _event_id = event_id_result.unwrap();

        // In a real scenario, we would fetch the event from a relay.
        // For this test, we'll simulate an event that would be received.
        // We need to construct an Event with the content that nip44_direct_message
        // would produce. Since we refactored nip44_direct_message to use
        // PrivateKey::encrypt, we can re-encrypt the message to get the
        // expected content.

        let encrypted_content_from_sender = sender_privkey
            .encrypt(
                &recipient_pubkey,
                &original_message,
                ContentEncryptionAlgorithm::Nip44v2,
            )
            .unwrap();

        // 2. Decrypt the message using the recipient's private key
        let decrypted_message_result =
            recipient_privkey.decrypt(&sender_pubkey, &encrypted_content_from_sender);

        assert!(decrypted_message_result.is_ok());

        let decrypted_message = decrypted_message_result.unwrap();

        assert_eq!(decrypted_message, original_message);

        println!("{} == {}", decrypted_message, original_message);
    }

    #[tokio::test]
    #[serial]
    async fn test_dm_command_decryption_of_provided_event() {
        let sender_pubkey_hex = "a34b99f22c790c4e36b2b3c2c35a36db06226e41c692fc82b8b56ac1c540c5bd";
        let encrypted_content = "AsQtQ6ZH81LsRiwmItz/gdy4Yjnlf4nZ2C8smdHdgCZY6dPrFVJC0JzgCz8XhnZ26uwGRB214C12J9fHYVFRO7B4Io7erA4+T/kmGgnSCTHabroLM5WKTYOBFuXtXCF40FIP";

        // Define keys within the test function to ensure they are in scope
        let recipient_privkey = PrivateKey::generate(); // Generate a new private key for the recipient
        let sender_pubkey = PublicKey::try_from_hex_string(sender_pubkey_hex, false).unwrap();

        let decrypted_message_result = recipient_privkey.decrypt(&sender_pubkey, encrypted_content);

        // Negative Test
        match decrypted_message_result {
            Ok(msg) => panic!("Decryption unexpectedly succeeded with message: {}", msg),
            Err(actual_error) => {
                eprintln!("Decryption error for provided event: {}", actual_error);
                assert!(actual_error.to_string().contains("Invalid MAC"));
            }
        }
    }
}
