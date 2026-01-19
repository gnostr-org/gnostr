use anyhow::{Context, Result};
use clap::Args;
use console::Style;
use std::path::Path;

use crate::{
    cli_interactor::{Interactor, InteractorPrompt, PromptInputParms, PromptPasswordParms},
    client::{send_events, Client, Connect},
    login,
    types::{nip4, EventBuilder, EventKind, PrivateKey, PublicKey, Tag},
};

#[derive(Debug, Args, Clone)]
pub struct DmArgs {
    /// Recipient's public key (hex or npub)
    #[arg(short, long)]
    pub to: String,

    /// Message content (if not provided, will prompt)
    #[arg(short, long)]
    pub message: Option<String>,

    /// Path to private key file
    #[arg(short = 'k', long)]
    pub key_file: Option<String>,

    /// Private key (hex or nsec)
    #[arg(short = 's', long)]
    pub secret: Option<String>,

    /// Relays to send to (comma-separated)
    #[arg(short, long, value_delimiter = ',')]
    pub relays: Vec<String>,

    /// Disable CLI spinners
    #[arg(long)]
    pub disable_cli_spinners: bool,
}

pub async fn launch(args: &DmArgs) -> Result<()> {
    // Get sender's private key
    let private_key = get_private_key(args)?;

    // Parse recipient public key
    let recipient_pubkey = parse_recipient_pubkey(&args.to)?;

    // Get message content
    let message = get_message_content(args)?;

    // Create encrypted direct message event
    let event = create_dm_event(&private_key, &recipient_pubkey, &message)?;

    // Setup client and send
    let client = <Client as Connect>::default();
    let relays = if args.relays.is_empty() {
        vec![
            "wss://relay.damus.io".to_string(),
            "wss://relay.nostr.band".to_string(),
        ]
    } else {
        args.relays.clone()
    };

    println!("Sending DM to {}", args.to);
    println!("Message: {}", message);

    #[cfg(not(test))]
    send_events(
        &client,
        Path::new("."),
        vec![event],
        relays.into_iter().collect(),
        vec![],
        !args.disable_cli_spinners,
        false,
    )
    .await?;

    let dim = Style::new().color256(247);
    println!("{}", dim.apply_to("DM sent successfully!"));

    Ok(())
}

fn get_private_key(args: &DmArgs) -> Result<PrivateKey> {
    // Try secret key first
    if let Some(secret) = &args.secret {
        return PrivateKey::try_from_bech32_string(secret)
            .or_else(|_| PrivateKey::try_from_hex_string(secret))
            .context("Failed to parse private key from secret argument");
    }

    // Try key file
    if let Some(key_file) = &args.key_file {
        let content = std::fs::read_to_string(key_file)
            .with_context(|| format!("Failed to read key file: {}", key_file))?;
        let trimmed = content.trim();
        return PrivateKey::try_from_bech32_string(trimmed)
            .or_else(|_| PrivateKey::try_from_hex_string(trimmed))
            .context("Failed to parse private key from file");
    }

    // Prompt for key
    let key_input = Interactor::default().password(PromptPasswordParms {
        prompt: "Enter your private key (hex or nsec)".to_string(),
        confirm: false,
    })?;

    PrivateKey::try_from_bech32_string(&key_input)
        .or_else(|_| PrivateKey::try_from_hex_string(&key_input))
        .context("Failed to parse private key")
}

fn parse_recipient_pubkey(to: &str) -> Result<PublicKey> {
    if to.starts_with("npub") {
        PublicKey::try_from_bech32_string(to, false).context("Failed to parse npub")
    } else {
        PublicKey::try_from_hex_string(to, false).context("Failed to parse hex public key")
    }
}

fn get_message_content(args: &DmArgs) -> Result<String> {
    match &args.message {
        Some(msg) => Ok(msg.clone()),
        None => {
            let message = Interactor::default()
                .input(PromptInputParms::default().with_prompt("Enter your message"))?;
            Ok(message)
        }
    }
}

fn create_dm_event(
    private_key: &PrivateKey,
    recipient_pubkey: &PublicKey,
    message: &str,
) -> Result<nostr_0_34_1::Event> {
    use nostr_sdk_0_34_0::prelude::*;

    // Create nostr_sdk keys from our private key
    let nostr_keys =
        nostr_sdk_0_34_0::Keys::parse(&hex::encode(private_key.as_secret_key().secret_bytes()))?;

    // Create EventBuilder for encrypted direct message
    let event_builder = nostr_sdk_0_34_0::EventBuilder::new(
        nostr_sdk_0_34_0::Kind::EncryptedDirectMessage,
        message,
        [nostr_sdk_0_34_0::Tag::public_key(PublicKey::from_hex(
            &hex::encode(recipient_pubkey.as_bytes()),
        )?)],
    );

    // Sign the event
    Ok(event_builder.to_event(&nostr_keys)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_recipient_pubkey() {
        // Test hex format
        let hex_key = "32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245";
        let result = parse_recipient_pubkey(hex_key);
        assert!(result.is_ok());

        // Test invalid format
        let invalid_key = "invalid_key";
        let result = parse_recipient_pubkey(invalid_key);
        assert!(result.is_err());
    }
}
