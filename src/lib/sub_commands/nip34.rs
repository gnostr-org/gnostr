use std::borrow::Cow;
use clap::{Args, Subcommand};
use nostr_sdk_0_34_0::prelude::*;
use crate::utils::{create_client, parse_private_key};

#[derive(Args, Debug)]
pub struct Nip34SubCommand {
    #[command(subcommand)]
    pub command: Nip34Commands,
}

#[derive(Subcommand, Debug)]
pub enum Nip34Commands {
    /// Announce a Git repository on Nostr (NIP-34, kind 30617)
    AnnounceRepo {
        /// Repository name
        #[arg(short, long)]
        name: String,

        /// Repository description
        #[arg(short, long)]
        description: Option<String>,

        /// Git clone URL (e.g., https://github.com/user/repo.git)
        #[arg(long)]
        clone_url: String,

        /// Maintainer public keys (hex or bech32), comma-separated
        #[arg(long)]
        maintainers: String,

        /// Nostr relays to publish to (wss://...), comma-separated
        #[arg(short, long)]
        relays: String,

        /// Private key to sign the event (hex or bech32)
        #[arg(short, long)]
        private_key: Option<String>,

        /// Print event ID as hex instead of bech32
        #[arg(long, default_value = "false")]
        hex: bool,
    },
}

pub async fn nip34_command(
    sub_command_args: &Nip34SubCommand,
) -> Result<()> {
    match &sub_command_args.command {
        Nip34Commands::AnnounceRepo {
            name,
            description,
            clone_url,
            maintainers,
            relays,
            private_key,
            hex,
        } => {
            let keys = parse_private_key(private_key.clone(), false).await?;
            let relay_urls: Vec<String> = relays.split(',').map(|s| s.trim().to_string()).collect();

            if relay_urls.is_empty() {
                panic!("No relays specified, at least one relay is required!")
            }

            let client = create_client(&keys, relay_urls.clone(), 0).await?;

            let mut tags = vec![
                Tag::new(TagKind::Custom(Cow::Borrowed("name")), &[name.clone()]),
                Tag::new(TagKind::Custom(Cow::Borrowed("clone")), &[clone_url.clone()]),
            ];

            if let Some(desc) = description {
                tags.push(Tag::new(TagKind::Custom(Cow::Borrowed("description")), &[desc.clone()]));
            }

            let maintainer_pks: Vec<PublicKey> = maintainers
                .split(',')
                .map(|s| s.trim())
                .filter_map(|s| PublicKey::from_bech32(s).or_else(|_| PublicKey::from_hex(s)).ok())
                .collect();

            for pk in maintainer_pks {
                tags.push(Tag::public_key(pk));
            }

            for relay_url in relay_urls {
                tags.push(Tag::relay(Url::parse(&relay_url)?));
            }

            let event = EventBuilder::new(
                Kind::Custom(30617), // NIP-34 Repository Announcement
                format!("Repository Announcement: {}", name),
                tags,
            ).to_event(&keys)?;

            let event_id = client.send_event(event).await?;

            if *hex {
                println!("{}", event_id.to_hex());
            } else {
                println!("{}", event_id.to_bech32()?);
            }
            Ok(())
        }
    }
}