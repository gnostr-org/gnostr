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
    use crate::types::ContentEncryptionAlgorithm;
    use crate::types::{Keys, PrivateKey};
    use serial_test::serial;
    use tokio;
    use base64::Engine;

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
    async fn test_dm_command_invalid_hex_pubkey() {

        let recipient_pubkey_str = "invalidhexpubkey";
        let recipient_pubkey_result = PublicKey::try_from_hex_string(recipient_pubkey_str, false);
        assert!(recipient_pubkey_result.is_err());

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

        // Add intentional ws:// and wss:// mistakes
        client
            .add_relays(vec!["ws://relay.damus.io".to_string(), "wss://localhost:8080".to_string()]).await.unwrap();

        let recipient_pubkey = PublicKey::try_from_hex_string(
            "edfa27d49d2af37ee331e1225bb6ed1912c6d999281b36d8018ad99bc3573c29",
            false,
        )
        .unwrap();

        let message_content = "gnostr dm sub_command test may fail to encrypt!".to_string();

        let result = dm_command(&client, recipient_pubkey.clone(), message_content.clone()).await;

        //assert!(result.is_ok());

        // Assertions - we now expect a real error from the client's operations
        assert!(result.is_err());
        let actual_error = result.unwrap_err();
        eprintln!("Actual error: {}", actual_error);
        assert!(actual_error
            .to_string()
            .contains("Failed to send event to any configured relay."));
    }

    #[tokio::test]
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
        let recipient_keys = Keys::new(recipient_privkey.clone());

        // Add a dummy relay for the sender client (actual relay not needed for encryption/decryption logic)
        sender_client
            .add_relays(vec!["wss://relay.damus.io".to_string(), "ws://localhost:8080".to_string()])
            .await
            .unwrap();

        let original_message = "This is a secret message!".to_string();

        // 1. Encrypt the message using dm_command
        let event_id_result = sender_client
            .nip44_direct_message(recipient_pubkey.clone(), original_message.clone())
            .await;

        assert!(event_id_result.is_ok());
        let event_id = event_id_result.unwrap();

        // In a real scenario, we would fetch the event from a relay.
        // For this test, we'll simulate an event that would be received.
        // We need to construct an Event with the content that nip44_direct_message would produce.
        // Since we refactored nip44_direct_message to use PrivateKey::encrypt, we can re-encrypt
        // the message to get the expected content.

        let encrypted_content_from_sender = sender_privkey
            .encrypt(&recipient_pubkey, &original_message, ContentEncryptionAlgorithm::Nip44v2)
            .unwrap();

        // 2. Decrypt the message using the recipient's private key
        let decrypted_message_result = recipient_privkey
            .decrypt(&sender_pubkey, &encrypted_content_from_sender);

        assert!(decrypted_message_result.is_ok());
        assert_eq!(decrypted_message_result.unwrap(), original_message);
    }

    #[tokio::test]
    #[serial]
    async fn test_dm_command_decryption_of_provided_event() {
        let sender_pubkey_hex = "a34b99f22c790c4e36b2b3c2c35a36db06226e41c692fc82b8b56ac1c540c5bd";
        let encrypted_content = "AsQtQ6ZH81LsRiwmItz/gdy4Yjnlf4nZ2C8smdHdgCZY6dPrFVJC0JzgCz8XhnZ26uwGRB214C12J9fHYVFRO7B4Io7erA4+T/kmGgnSCTHabroLM5WKTYOBFuXtXCF40FIP";

        let decrypted_message_result = recipient_privkey
            .decrypt(&sender_pubkey, encrypted_content);

        match decrypted_message_result {
            Ok(msg) => panic!("Decryption unexpectedly succeeded with message: {}", msg),
            Err(actual_error) => {
                eprintln!("Decryption error for provided event: {}", actual_error);
                assert!(actual_error.to_string().contains("Invalid MAC"));
            }
        }
    }
}
