use std::str::FromStr;

use clap::Args;
use anyhow::{Result, Error as AnyhowError};
use crate::types::{Client, Event, EventKind, Keys, PreEventV3, PublicKey, Tag, Unixtime, UncheckedUrl, KeySigner};
use serde::Deserialize;

use crate::utils::{create_client, parse_private_key};

#[derive(Args, Debug)]
pub struct PublishContactListCsvSubCommand {
    /// Path to CSV file. CSV file should be have the following format:
    /// pubkey,relay_url,petname. See example in resources/contact_list.csv
    #[arg(short, long)]
    filepath: String,
    // Print keys as hex
    #[arg(long, default_value = "false")]
    hex: bool,
}

// nostr_rust ContactListTag struct does not derive "Deserialize", therefore we need this custom implementation
#[derive(Debug, Clone, Deserialize)]
pub struct ContactListTag {
    /// 32-bytes hex key - the public key of the contact
    pub pubkey: String,
    /// main relay URL
    pub relay: Option<String>,
    /// Petname
    pub petname: Option<String>,
}

pub async fn publish_contact_list_from_csv_file(
    private_key: Option<String>,
    relays: Vec<String>,
    difficulty_target: u8,
    sub_command_args: &PublishContactListCsvSubCommand,
) -> Result<(), AnyhowError> {
    if relays.is_empty() {
        panic!("No relays specified, at least one relay is required!")
    }

    let keys = parse_private_key(private_key, false).await?;
    let client = create_client(&keys, relays, difficulty_target).await?;

    let mut rdr = csv::Reader::from_path(&sub_command_args.filepath)?;
    let mut tags: Vec<Tag> = vec![];
    for result in rdr.deserialize() {
        let tag_data: ContactListTag = result?;
        let pubkey = PublicKey::try_from_hex_string(&tag_data.pubkey, true)?;
        let mut tag_vec = vec!["p".to_string(), pubkey.as_hex_string()];
        if let Some(relay) = tag_data.relay {
            tag_vec.push(relay);
        }
        if let Some(petname) = tag_data.petname {
            tag_vec.push(petname);
        }
        tags.push(Tag::from_strings(tag_vec));
    }

    let pre_event = PreEventV3 {
        pubkey: keys.public_key(),
        created_at: Unixtime::now(),
        kind: EventKind::ContactList,
        tags,
        content: "".to_string(),
    };

    let signer = KeySigner::from_private_key(keys.secret_key()?, "", 1)?;
    let event = signer.sign_event(pre_event)?;

    client.send_event(event).await?;
    println!("Contact list imported!");
    Ok(())
}
