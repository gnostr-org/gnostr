use std::{str::FromStr, time::Duration};

use anyhow::{Error as AnyhowError, Result};
use clap::Args;
use tracing::debug;

use gnostr_asyncgit::types::{Client, Event, EventKind, Filter, Id, IdHex, Keys, PublicKey, PublicKeyHex, Tag, Unixtime};
use crate::utils::create_client;

#[derive(Args, Debug)]
pub struct ListEventsSubCommand {
    /// Ids
    #[arg(short, long, action = clap::ArgAction::Append)]
    pub ids: Option<Vec<String>>,
    /// Authors
    #[arg(short, long, action = clap::ArgAction::Append)]
    pub authors: Option<Vec<String>>,
    /// Kinds
    #[arg(short, long, action = clap::ArgAction::Append, default_values = [
           "30617", "30618", "1617", "1621", "1630", "1631", "1632", "1633"
       ])]
    pub kinds: Option<Vec<u64>>,
    /// e tag
    #[arg(long, action = clap::ArgAction::Append)]
    pub etag: Option<Vec<String>>,
    /// p tag
    #[arg(long, action = clap::ArgAction::Append)]
    pub ptag: Option<Vec<String>>,
    /// d tag
    #[arg(long, action = clap::ArgAction::Append)]
    pub dtag: Option<Vec<String>>,
    /// a tag
    #[arg(long, action = clap::ArgAction::Append)]
    pub atag: Option<Vec<String>>,
    /// Since
    #[arg(short, long, action = clap::ArgAction::Append)]
    pub since: Option<u64>,
    /// Until
    #[arg(short, long, action = clap::ArgAction::Append)]
    pub until: Option<u64>,
    /// Limit
    #[arg(short, long, action = clap::ArgAction::Append)]
    pub limit: Option<usize>,
    /// Output .git/<output>.json
    #[arg(short, long)]
    pub output: Option<String>,
    /// Timeout in seconds
    #[arg(long)]
    pub timeout: Option<u64>,
}

pub async fn list_events(
    relays: Vec<String>,
    sub_command_args: &ListEventsSubCommand,
) -> Result<(), AnyhowError> {
    if relays.is_empty() {
        panic!("No relays specified, at least one relay is required!")
    }

    let keys = Keys::generate();
    let client = create_client(&keys, relays, 0).await?;
    let mut filter = Filter::new();

    // Handle event ids
    if let Some(ids_str) = &sub_command_args.ids {
        let ids: Result<Vec<IdHex>, _> = ids_str.iter().map(|id| IdHex::try_from_str(id)).collect();
        filter.ids = ids?;
    }

    // Handle author public keys
    if let Some(authors_str) = &sub_command_args.authors {
        let authors: Result<Vec<PublicKeyHex>, _> = authors_str
            .iter()
            .map(|author| PublicKeyHex::try_from_str(author))
            .collect();
        filter.authors = authors?;
    }

    // Handle kind numbers
    if let Some(kinds_u64) = &sub_command_args.kinds {
        filter.kinds = kinds_u64
            .iter()
            .map(|k| EventKind::from(*k as u32))
            .collect();
    }

    // Handle e-tags
    if let Some(etags) = &sub_command_args.etag {
        filter.add_tag_value('e', etags.join(","));
    }

    // Handle p-tags
    if let Some(ptags) = &sub_command_args.ptag {
        filter.add_tag_value('p', ptags.join(","));
    }

    // Handle d-tags
    if let Some(dtags) = &sub_command_args.dtag {
        filter.add_tag_value('d', dtags.join(","));
    }

    if let Some(since) = sub_command_args.since {
        filter.since = Some(Unixtime(since as i64));
    }

    if let Some(until) = sub_command_args.until {
        filter.until = Some(Unixtime(until as i64));
    }

    if let Some(limit) = sub_command_args.limit {
        filter.limit = Some(limit);
    }

    let timeout = sub_command_args.timeout.map(Duration::from_secs);

    let events: Vec<Event> = client.get_events_of(vec![filter], timeout).await?;

    if let Some(output) = &sub_command_args.output {
        let file = std::fs::File::create(".git/".to_owned() + output)?;
        serde_json::to_writer_pretty(file, &events)?;
        debug!(
            "Wrote {} event(s) to {}",
            events.len(),
            ".git/".to_owned() + output
        );
    } else {
        println!("{}", serde_json::to_string_pretty(&events)?)
    }

    Ok(())
}
