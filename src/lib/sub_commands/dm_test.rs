#![cfg(test)]

use anyhow::anyhow;
use async_trait::async_trait;
use mockall::{automock, predicate::*};
use crate::types::{Client as GnostrClient, Error, Id, Keys, PublicKey};
use crate::sub_commands::dm::dm_command; // Corrected import path for the function to be tested
use crate::sub_commands::dm::DmClientTrait; // Import the new trait

// Mock the DmClientTrait for testing dm_command
#[automock]
#[async_trait]
pub trait DmClientTrait {
    async fn add_relays(&mut self, relays: Vec<String>) -> Result<(), Error>;
    async fn nip44_direct_message(
        &self,
        recipient_pubkey: PublicKey,
        message: String,
    ) -> Result<Id, Error>;
}

mod tests {
    use super::*;
    use crate::types::{Keys, PrivateKey}; // Import PrivateKey
    use tokio; // Import tokio for async tests

    #[tokio::test]
    async fn test_dm_command_success() {
        // Setup mock client
        let mut mock_client = MockGnostrClient::new();

        // Create dummy keys and public key
        let dummy_privkey = PrivateKey::try_from_hex_string(
            "0000000000000000000000000000000000000000000000000000000000000001",
        ).unwrap();
        let dummy_keys = Keys::new(dummy_privkey);
        let recipient_pubkey = dummy_keys.public_key(); // Use the public key from dummy_keys

        let expected_event_id = Id::from_hex("1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").unwrap();
        let message_content = "Hello, world!".to_string();

        // Expect nip44_direct_message to be called and return success
        mock_client
            .expect_nip44_direct_message()
            .with(eq(recipient_pubkey.clone()), eq(message_content.clone()))
            .times(1)
            .returning(move |_, _| Ok(expected_event_id.clone()));

        // Call the function under test
        let result = dm_command(&mock_client, recipient_pubkey.clone(), message_content.clone()).await;

        // Assertions
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_dm_command_failure() {
        // Setup mock client
        let mut mock_client = MockGnostrClient::new();

        // Create dummy keys and public key
        let dummy_privkey = PrivateKey::try_from_hex_string(
            "0000000000000000000000000000000000000000000000000000000000000001",
        ).unwrap();
        let dummy_keys = Keys::new(dummy_privkey);
        let recipient_pubkey = dummy_keys.public_key(); // Use the public key from dummy_keys

        let error_message = "Failed to encrypt message".to_string();
        let message_content = "Secret message".to_string();

        // Expect nip44_direct_message to be called and return an error
        mock_client
            .expect_nip44_direct_message()
            .with(eq(recipient_pubkey.clone()), eq(message_content.clone()))
            .times(1)
            .returning(move |_, _| Err(Error::Custom(anyhow!(error_message.clone()))));

        // Call the function under test
        let result = dm_command(&mock_client, recipient_pubkey.clone(), message_content.clone()).await;

        // Assertions
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Failed to encrypt message"
        );
    }
}
