use crate::types::{Client, Error, Id, Keys, PublicKey};
use anyhow::Result;
use async_trait::async_trait;
use tracing::{error, info};

#[async_trait]
pub trait DmClientTrait {
    async fn add_relays(&mut self, relays: Vec<String>) -> Result<(), Error>;
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
) -> Result<(), Error> {
    info!(
        "Sending NIP-44 direct message to {}",
        recipient_pubkey.as_hex_string()
    );

    match client.nip44_direct_message(recipient_pubkey, message).await {
        Ok(event_id) => {
            info!("Direct message sent successfully! Event ID: {}", event_id);
            Ok(())
        }
        Err(e) => {
            error!("Failed to send direct message: {}", e);
            Err(e)
        }
    }
}

#[cfg(test)]
mod dm_tests {
    use super::*;
    use crate::types::client::{Client, Options};
    use crate::types::{Keys, PrivateKey};
    use serial_test::serial;
    use tokio;

    #[tokio::test]
    #[serial]
    async fn test_dm_command_success() {
        // Setup real client
        let sender_privkey = PrivateKey::try_from_hex_string(
            "0000000000000000000000000000000000000000000000000000000000000001",
        )
        .unwrap();
        let sender_keys = Keys::new(sender_privkey);
        let mut client = Client::new(&sender_keys, Options::new());

        // Add a gnostr-relay
        client
            .add_relays(vec!["wss://relay.damus.io".to_string(), "ws://localhost:8080".to_string()])
            .await
            .unwrap();

        // Create recipient public key
        let recipient_privkey = PrivateKey::try_from_hex_string(
            "0000000000000000000000000000000000000000000000000000000000000002",
        )
        .unwrap();
        let recipient_keys = Keys::new(recipient_privkey);
        let recipient_pubkey = recipient_keys.public_key();

        let message_content = "gnostr dm sub_command test!".to_string();

        // Call the function under test (this will now use the real nip44_direct_message)
        let result = dm_command(&client, recipient_pubkey.clone(), message_content.clone()).await;

        // Assertions
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[serial]
    async fn test_dm_command_success_bech32_recipient() {
        // Setup real client
        let sender_privkey = PrivateKey::try_from_hex_string(
            "0000000000000000000000000000000000000000000000000000000000000001",
        )
        .unwrap();
        let sender_keys = Keys::new(sender_privkey);
        let mut client = Client::new(&sender_keys, Options::new());

        // Add gnostr-relays
        client
            .add_relays(vec!["wss://relay.damus.io".to_string(), "ws://localhost:8080".to_string()])
            .await
            .unwrap();

        // Create recipient public key from bech32 string
        let recipient_pubkey = PublicKey::try_from_bech32_string(
            "npub1ahaz04ya9tehace3uy39hdhdryfvdkve9qdndkqp3tvehs6h8s5slq45hy",
            false,
        )
        .unwrap();

        let message_content = "gnostr dm sub_command test with bech32 recipient!".to_string();

        // Call the function under test
        let result = dm_command(&client, recipient_pubkey.clone(), message_content.clone()).await;

        // Assertions
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[serial]
    async fn test_dm_command_failure() {
        // Setup real client (we expect nip44_direct_message to potentially fail for other reasons)
        let sender_privkey = PrivateKey::try_from_hex_string(
            "0000000000000000000000000000000000000000000000000000000000000001",
        )
        .unwrap();
        let sender_keys = Keys::new(sender_privkey);
        let mut client = Client::new(&sender_keys, Options::new());

        // Add a dummy relay for the client to connect to
        client
            .add_relays(vec!["wss://relay.damus.io".to_string(), "ws://localhost:8080".to_string()])
            .await
            .unwrap();

        // Create recipient public key (a malformed one to simulate encryption failure if needed, or simply let the real function fail)
        let recipient_pubkey_str = "invalidhexpubkey";
        let recipient_pubkey_result = PublicKey::try_from_hex_string(recipient_pubkey_str, false);
        assert!(recipient_pubkey_result.is_err()); // Ensure it's an invalid key for this test's intent

        let recipient_pubkey = PublicKey::try_from_hex_string(
            "0000000000000000000000000000000000000000000000000000000000000003", // Use a valid but arbitrary pubkey for client call
            false,
        )
        .unwrap();

        let message_content = "Secret message that might fail to encrypt".to_string();

        // Call the function under test (expecting it to fail due to, e.g., connection issues or malformed key in the real nip44_direct_message)
        let result = dm_command(&client, recipient_pubkey.clone(), message_content.clone()).await;

        // Assertions - we now expect a real error from the client's operations
        assert!(result.is_err());
        let actual_error = result.unwrap_err();
        eprintln!("Actual error: {}", actual_error);
        assert!(actual_error
            .to_string()
            .contains("Failed to send event to any configured relay."));
    }
}
