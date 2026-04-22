use anyhow::{Error as AnyhowError, Result};
use clap::Args;

use crate::{
    types::{
        Client, Event, EventBuilder, EventKind, Filter, Id, ImageDimensions, Keys, Metadata,
        PrivateKey, PublicKey, Tag, UncheckedUrl, Unixtime,
    },
    utils::{create_client, parse_private_key},
};

#[derive(Args, Debug)]
pub struct CreateBadgeSubCommand {
    /// Unique identifier for the badge
    #[arg(short, long)]
    id: String,
    /// Badge name
    #[arg(short, long)]
    name: Option<String>,
    /// Badge description
    #[arg(short, long)]
    description: Option<String>,
    /// Badge image url
    #[arg(long)]
    image_url: Option<String>,
    /// Badge image width
    #[arg(long)]
    image_size_width: Option<u64>,
    /// Badge image height
    #[arg(long)]
    image_size_height: Option<u64>,
    /// Badge thumbnail image url
    #[arg(short, long)]
    thumb_url: Option<String>,
    /// Badge thumbnail width
    #[arg(long)]
    thumb_size_width: Option<u64>,
    /// Badge thumbnail height
    #[arg(long)]
    thumb_size_height: Option<u64>,
}

pub async fn create_badge(
    private_key: Option<String>,
    relays: Vec<String>,
    difficulty_target: u8,
    sub_command_args: &CreateBadgeSubCommand,
) -> Result<(), AnyhowError> {
    if relays.is_empty() {
        panic!("No relays specified, at least one relay is required!")
    }

    let keys = parse_private_key(private_key, false).await?;
    let client = create_client(&keys, relays, difficulty_target).await?;

    let image_size = match (
        sub_command_args.image_size_height,
        sub_command_args.image_size_width,
    ) {
        (Some(height), Some(width)) => Some(ImageDimensions { height, width }),
        _ => None,
    };

    let thumbnails = if let Some(thumb_url) = sub_command_args.thumb_url.clone() {
        let thumb_size = match (
            sub_command_args.thumb_size_height,
            sub_command_args.thumb_size_width,
        ) {
            (Some(width), Some(height)) => Some((width, height)),
            _ => None,
        };

        let url = UncheckedUrl::from_string(thumb_url);

        if let Some((width, height)) = thumb_size {
            vec![(url, Some(ImageDimensions { width, height }))]
        } else {
            vec![(url, None)]
        }
    } else {
        Vec::new()
    };

    let image_url: Option<UncheckedUrl> = sub_command_args
        .image_url
        .clone()
        .map(UncheckedUrl::from_string);

    let event_builder = EventBuilder::define_badge(
        sub_command_args.id.clone(),
        sub_command_args.name.clone(),
        sub_command_args.description.clone(),
        image_url.clone(),
        image_size,
        thumbnails.clone(),
    );
    let private_key = keys.secret_key()?;
    let mut event = event_builder.to_pow_event(&private_key, difficulty_target)?; // Add 'image' tag
    if let Some(url) = image_url {
        if let Some(dims) = image_size {
            event
                .tags
                .push(Tag::new_image(url, Some(dims.width), Some(dims.height)));
        } else {
            event.tags.push(Tag::new_image(url, None, None));
        }
    }

    // Add 'thumb' tags
    for (thumb_url, thumb_dims) in thumbnails {
        if let Some(dims) = thumb_dims {
            event.tags.push(Tag::new_thumb(
                thumb_url,
                Some(dims.width),
                Some(dims.height),
            ));
        } else {
            event.tags.push(Tag::new_thumb(thumb_url, None, None));
        }
    }

    // TODO: Sign the event with the keys (this would replace to_pow_event)
    // For now, the dummy event has a dummy signature.

    // Publish event
    let event_id = client.send_event(event).await?;
    println!("Published badge definition with id:");
    println!("Hex: {}", event_id.as_hex_string());
    println!("Bech32: {}", event_id.as_bech32_string());

    Ok(())
}
