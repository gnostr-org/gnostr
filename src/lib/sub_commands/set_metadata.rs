use clap::Args;
use anyhow::{Result, Error as AnyhowError};
use crate::types::{
    Client, Event, EventKind, Id, Keys, Metadata, PreEventV3, Tag, Unixtime, UncheckedUrl, KeySigner,
    Signer,
};
use serde_json::Value;

use crate::utils::{create_client, parse_private_key};

#[derive(Args, Debug)]
pub struct SetMetadataSubCommand {
    /// Set profile name
    #[arg(short, long)]
    name: Option<String>,
    /// Set your bio
    #[arg(short, long)]
    about: Option<String>,
    /// Set your profile image URL
    #[arg(short, long)]
    picture: Option<String>,
    /// Set your profile image URL
    #[arg(short, long)]
    banner: Option<String>,
    /// Set your NIP-05
    #[arg(long)]
    nip05: Option<String>,
    /// Set your LUD-06 LNURL
    #[arg(long)]
    lud06: Option<String>,
    /// Set your LUD-16 LN address
    #[arg(long)]
    lud16: Option<String>,
    /// External identities. Use this syntax: "platform:identity:proof"
    #[arg(short, long)]
    identities: Vec<String>,
    /// Arbitrary fields not in the protocol. Use this syntax: "key:value"
    #[arg(short, long, action = clap::ArgAction::Append, default_values_t = ["gnostr:gnostr".to_string()])]
    extra_field: Vec<String>,
    /// Print keys as hex
    #[arg(long, default_value = "false")]
    hex: bool,
}

pub async fn set_metadata(
    private_key: Option<String>,
    relays: Vec<String>,
    difficulty_target: u8,
    sub_command_args: &SetMetadataSubCommand,
) -> Result<(), AnyhowError> {
    if relays.is_empty() {
        panic!("No relays specified, at least one relay is required!")
    }

    let keys = parse_private_key(private_key, false).await?;
    let client = create_client(&keys, relays, difficulty_target).await?;

    let mut metadata = Metadata::new();

    // Name
    if let Some(name) = &sub_command_args.name {
        metadata.name = Some(name.clone());
    }

    // About
    if let Some(about) = &sub_command_args.about {
        metadata.about = Some(about.clone());
    }

    // Picture URL
    if let Some(picture_url) = &sub_command_args.picture {
        metadata.picture = Some(UncheckedUrl::from_str(picture_url));
    };
    // Banner URL
    if let Some(banner_url) = &sub_command_args.banner {
        metadata.other.insert("banner".to_string(), Value::String(banner_url.to_string()));
    };

    // NIP-05 identifier
    if let Some(nip05_identifier) = &sub_command_args.nip05 {
        // TODO: Implement nip05::verify without nostr_sdk
        metadata.nip05 = Some(nip05_identifier.clone());
    }

    // LUD-06 string
    if let Some(lud06) = &sub_command_args.lud06 {
        metadata.other.insert("lud06".to_string(), Value::String(lud06.to_string()));
    }

    // LUD-16 string
    if let Some(lud16) = &sub_command_args.lud16 {
        metadata.other.insert("lud16".to_string(), Value::String(lud16.to_string()));
    }

    // Set custom fields
    for ef in sub_command_args.extra_field.iter() {
        let sef: Vec<&str> = ef.split(':').collect();
        if sef.len() == 2 {
            metadata.other.insert(sef[0].to_string(), Value::String(sef[1].to_string()));
        }
    }

    let mut tags: Vec<Tag> = Vec::new();
    // External identity tags (NIP-39)
    for identity in &sub_command_args.identities {
        let parts: Vec<&str> = identity.split(':').collect();
        if parts.len() == 3 {
            let platform_identity = format!("{}:{}", parts[0], parts[1]);
            let proof = parts[2].to_string();
            tags.push(Tag::new(&["i", &platform_identity, &proof]));
        } else {
            eprintln!("Invalid identity format: {}", identity);
        }
    }

    let pre_event = PreEventV3 {
        pubkey: keys.public_key(),
        created_at: Unixtime::now(),
        kind: EventKind::Metadata,
        tags,
        content: serde_json::to_string(&metadata)?,
    };

    let signer = KeySigner::from_private_key(keys.secret_key()?, "", 1)?;
    let event = signer.sign_event(pre_event)?;
    
    let event_id = client.send_event(event).await?;
    if sub_command_args.hex {
        print!("{{\"id\":\"{}\"}}", event_id.as_hex_string());
    } else {
        print!("{{\"id\":\"{}\"}}", event_id.as_bech32_string());
    }

    Ok(())
}
