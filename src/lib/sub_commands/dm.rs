use anyhow::Result;
use crate::types::{Client, Error, PublicKey, Id, Keys};
use tracing::{info, error};
use async_trait::async_trait;

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
