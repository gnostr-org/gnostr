use anyhow::Result;
use crate::types::{Client, Error, PublicKey};
use tracing::{info, error};

pub async fn dm_command(
    client: &Client,
    recipient_pubkey: PublicKey,
    message: String,
) -> Result<(), Error> {
    info!("Sending NIP-44 direct message to {}", recipient_pubkey.as_hex_string());

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
