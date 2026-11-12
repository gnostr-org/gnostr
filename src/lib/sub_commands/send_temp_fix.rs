use anyhow::Result;

use crate::types::Tag;

/// Convert a local tag to the local `Tag` representation.
fn convert_nostr_tag_to_local(nostr_tag: &Tag) -> Result<Tag> {
    Ok(Tag::from_strings(nostr_tag.clone().into_inner()))
}

async fn get_root_proposal_id_and_mentions_from_in_reply_to(
    _git_repo_path: &std::path::Path,
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
