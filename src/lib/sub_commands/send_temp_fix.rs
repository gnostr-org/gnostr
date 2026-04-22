// This file contains the remaining part of the send.rs file that was corrupted

use anyhow::{Context, Result};
use nostr_sdk_0_34_0::hashes::sha1::Hash as Sha1Hash;

use crate::types::Tag;

/// Convert nostr_0_34_1::Tag to local Tag type
fn convert_nostr_tag_to_local(nostr_tag: &nostr_0_34_1::Tag) -> Result<Tag> {
    // Convert nostr_0_34_1::Tag to Vec<String>
    let tag_vec: Vec<String> = nostr_tag
        .clone()
        .to_vec()
        .iter()
        .map(|field| field.to_string())
        .collect();

    Ok(Tag::from_strings(tag_vec))
}

async fn get_root_proposal_id_and_mentions_from_in_reply_to(
    git_repo_path: &std::path::Path,
    in_reply_to: &[String],
) -> Result<(Option<String>, Vec<Tag>)> {
    let root_proposal_id = if let Some(first) = in_reply_to.first() {
        // Simplified - just return the string as-is for now
        Some(first.clone())
    } else {
        None
    };

    let mut mention_tags = vec![];
    for (i, reply_to) in in_reply_to.iter().enumerate() {
        if i.ne(&0) || root_proposal_id.is_none() {
            // Create a simple mention tag
            mention_tags.push(Tag::from_strings(vec!["p".to_string(), reply_to.clone()]));
        }
    }

    Ok((root_proposal_id, mention_tags))
}
