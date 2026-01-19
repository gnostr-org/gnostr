#![cfg(test)]

use anyhow::anyhow;
use crate::types::{Error, Id, Keys, PublicKey};
use crate::sub_commands::dm::dm_command;
use crate::types::client::{Client, Options};
use crate::types::RelayUrl;

mod tests {
    use super::*;
    use crate::types::{Keys, PrivateKey}; // Import PrivateKey
    use tokio; // Import tokio for async tests

    #[tokio::test]
    async fn test_dm_command_success() {
        // Setup real client
        let sender_privkey = PrivateKey::try_from_hex_string(
            "0000000000000000000000000000000000000000000000000000000000000001",
        ).unwrap();
        let sender_keys = Keys::new(sender_privkey);
        let mut client = Client::new(&sender_keys, Options::new());

        // Add a dummy relay for the client to connect to
        client.add_relays(vec!["ws://localhost:8008".to_string()]).await.unwrap();

        // Create recipient public key
        let recipient_privkey = PrivateKey::try_from_hex_string(
            "0000000000000000000000000000000000000000000000000000000000000002",
        ).unwrap();
        let recipient_keys = Keys::new(recipient_privkey);
        let recipient_pubkey = recipient_keys.public_key();

        let message_content = "Hello, world!".to_string();

        // Call the function under test (this will now use the real nip44_direct_message)
        let result = dm_command(&client, recipient_pubkey.clone(), message_content.clone()).await;

        // Assertions
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_dm_command_failure() {
        // Setup real client (we expect nip44_direct_message to potentially fail for other reasons)
        let sender_privkey = PrivateKey::try_from_hex_string(
            "0000000000000000000000000000000000000000000000000000000000000001",
        ).unwrap();
        let sender_keys = Keys::new(sender_privkey);
        let mut client = Client::new(&sender_keys, Options::new());

        // Add a dummy relay for the client to connect to
        client.add_relays(vec!["ws://localhost:8008".to_string()]).await.unwrap();

        // Create recipient public key (a malformed one to simulate encryption failure if needed, or simply let the real function fail)
        let recipient_pubkey_str = "invalidhexpubkey";
        let recipient_pubkey_result = PublicKey::try_from_hex_string(recipient_pubkey_str, false);
        assert!(recipient_pubkey_result.is_err()); // Ensure it's an invalid key for this test's intent

        let recipient_pubkey = PublicKey::try_from_hex_string(
            "0000000000000000000000000000000000000000000000000000000000000003", // Use a valid but arbitrary pubkey for client call
            false,
        ).unwrap();

        let message_content = "Secret message that might fail to encrypt".to_string();

        // Call the function under test (expecting it to fail due to, e.g., connection issues or malformed key in the real nip44_direct_message)
        let result = dm_command(&client, recipient_pubkey.clone(), message_content.clone()).await;

        // Assertions - we now expect a real error from the client's operations
        assert!(result.is_err());
        // The error message will now be from the actual nip44_direct_message implementation, likely related to connection or key conversion if any
        // For localhost:8008, it will likely be a connection refused error.
        assert!(result.unwrap_err().to_string().contains("Failed to connect to relay"));
    }
}
