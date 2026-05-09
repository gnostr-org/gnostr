use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
};

// Asyncgit's NIP-34 surface lives here.
//
// This module owns repo announcement/state plus the PR, update, status, and
// grasp kinds. `sync::notes` owns the 1617 git-note/PoW permutations.

use git2::Oid;
use serde::{Deserialize, Serialize};

use super::{
    Error, EventKind, EventV3, Id, KeySigner, NAddr, NEvent, Nip19, PreEventV3, PrivateKey,
    PublicKey, Signer, TagV3, Unixtime, UncheckedUrl,
};
use crate::{blockhash::blockhash_sync, blockheight::blockheight_sync, weeble::weeble_sync, wobble::wobble_sync};
use crate::nostr::nip13::NIP13Event;

/// NIP-34 repository announcement kind.
pub const REPO_ANNOUNCEMENT_KIND: u32 = 30617;
/// NIP-34 repository state kind.
pub const REPO_STATE_KIND: u32 = 30618;
/// NIP-34 pull request kind.
pub const PULL_REQUEST_KIND: u32 = 1618;
/// NIP-34 pull request update kind.
pub const PULL_REQUEST_UPDATE_KIND: u32 = 1619;
/// NIP-34 issue kind.
pub const GIT_ISSUE_KIND: u32 = 1621;
/// NIP-34 reply kind.
pub const GIT_REPLY_KIND: u32 = 1622;
/// NIP-34 user grasp list kind.
pub const USER_GRASP_LIST_KIND: u32 = 10317;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventRefType {
    Root,
    Reply,
    Quote,
}

pub type Nip34Kind = EventKind;
pub type Nip34Event = EventV3;
pub type Nip34UnsignedEvent = PreEventV3;

/// A deterministic NIP-34 git note wrapper around git-note storage data.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct GitNote {
    pub note_id: Oid,
    pub annotated_id: Oid,
    pub notes_ref: Option<String>,
    pub message: String,
    pub author: String,
    pub committer: String,
    pub committer_time: i64,
}

impl Serialize for GitNote {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct GitNoteSerde<'a> {
            note_id: String,
            annotated_id: String,
            notes_ref: Option<&'a str>,
            message: &'a str,
            author: &'a str,
            committer: &'a str,
            committer_time: i64,
        }

        GitNoteSerde {
            note_id: self.note_id.to_string(),
            annotated_id: self.annotated_id.to_string(),
            notes_ref: self.notes_ref.as_deref(),
            message: &self.message,
            author: &self.author,
            committer: &self.committer,
            committer_time: self.committer_time,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for GitNote {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct GitNoteSerde {
            note_id: String,
            annotated_id: String,
            notes_ref: Option<String>,
            message: String,
            author: String,
            committer: String,
            committer_time: i64,
        }

        let note = GitNoteSerde::deserialize(deserializer)?;
        Ok(Self {
            note_id: Oid::from_str(&note.note_id)
                .map_err(|error| serde::de::Error::custom(format!("invalid note_id: {error}")))?,
            annotated_id: Oid::from_str(&note.annotated_id).map_err(|error| {
                serde::de::Error::custom(format!("invalid annotated_id: {error}"))
            })?,
            notes_ref: note.notes_ref,
            message: note.message,
            author: note.author,
            committer: note.committer,
            committer_time: note.committer_time,
        })
    }
}

fn print_banner(label: &str) {
    println!();
    println!("==================== {label} ====================");
}

fn repo_announcement_kind() -> EventKind {
    EventKind::from(REPO_ANNOUNCEMENT_KIND)
}

fn repo_state_kind() -> EventKind {
    EventKind::from(REPO_STATE_KIND)
}

fn unique_push<T: PartialEq + Clone>(values: &mut Vec<T>, value: T) {
    if !values.contains(&value) {
        values.push(value);
    }
}

fn default_identifier(root_commit: &str) -> String {
    root_commit.chars().take(7).collect()
}

fn root_commit_tag(root_commit: &str) -> Result<TagV3, Error> {
    if root_commit.is_empty() {
        return Err(Error::InvalidOperation);
    }
    Ok(TagV3::from_strings(vec![
        "r".to_string(),
        root_commit.to_string(),
        "euc".to_string(),
    ]))
}

/// Return the kinds used for git status events.
pub fn status_kinds() -> Vec<EventKind> {
    vec![
        EventKind::GitStatusOpen,
        EventKind::GitStatusApplied,
        EventKind::GitStatusClosed,
        EventKind::GitStatusDraft,
    ]
}

pub fn git_note_event_id(commit_id: &str) -> Result<Id, Error> {
    let private_key = PrivateKey::try_from_hex_string(&padded_note_id(commit_id))?;
    Id::try_from_hex_string(&private_key.public_key().as_hex_string())
}

fn padded_note_id(note_id: &str) -> String {
    format!("{:0>64}", note_id)
}

fn git_note_runtime_values() -> Result<(String, f64, f64), Error> {
    let blockheight = blockheight_sync();
    let weeble = weeble_sync().map_err(|_| Error::InvalidOperation)?;
    let wobble = wobble_sync().map_err(|_| Error::InvalidOperation)?;
    let _ = blockhash_sync();

    Ok((blockheight, weeble, wobble))
}

/// Build the NIP-34 tags for a git note event.
pub fn git_note_tags(note: &GitNote) -> Result<Vec<TagV3>, Error> {
    let event_id = git_note_event_id(&note.annotated_id.to_string())?;
    let (blockheight, weeble, wobble) = git_note_runtime_values()?;

    let mut tags = vec![
        TagV3::new_event(event_id, None, Some("root".to_string())),
        TagV3::new_tag("commit", &note.annotated_id.to_string()),
    ];

    if let Some(notes_ref) = &note.notes_ref {
        tags.push(TagV3::new_tag("notes-ref", notes_ref));
    }

    tags.push(TagV3::new_tag("weeble", &weeble.to_string()));
    tags.push(TagV3::new_tag("blockheight", &blockheight));
    tags.push(TagV3::new_tag("wobble", &wobble.to_string()));

    Ok(tags)
}

fn git_note_preevent(note: &GitNote, pubkey: PublicKey) -> Result<PreEventV3, Error> {
    if note.committer_time < 0 {
        return Err(Error::InvalidOperation);
    }

    Ok(PreEventV3 {
        pubkey,
        created_at: Unixtime(note.committer_time),
        kind: EventKind::Patches,
        tags: git_note_tags(note)?,
        content: note.message.clone(),
    })
}

/// Build and sign a text-note event carrying git note content.
pub fn generate_git_note_event(note: &GitNote, private_key: &PrivateKey) -> Result<EventV3, Error> {
    git_note_sign(note, private_key, None)
}

/// Build, mine, and sign a text-note event carrying git note content.
pub fn generate_git_note_event_with_pow(
    note: &GitNote,
    private_key: &PrivateKey,
    difficulty: u8,
) -> Result<EventV3, Error> {
    git_note_sign(note, private_key, Some(difficulty))
}

fn git_note_sign(
    note: &GitNote,
    private_key: &PrivateKey,
    difficulty: Option<u8>,
) -> Result<EventV3, Error> {
    print_banner("nip34 git note event");
    println!(
        "building git note event as kind Patches; NIP-34 metadata is carried in tags for commit {}",
        note.annotated_id
    );
    let preevent = git_note_preevent(note, private_key.public_key())?;
    let event = match difficulty {
        Some(zero_bits) if zero_bits > 0 => {
            println!(
                "mining git note event with proof-of-work difficulty {}; event kind remains Patches",
                zero_bits
            );
            let signer = KeySigner::from_private_key(private_key.clone(), "", 1)?;
            signer.sign_event_with_pow(preevent, zero_bits, None)?
        }
        _ => EventV3::sign_with_private_key(preevent, private_key)?,
    };

    println!(
        "nip34 git note event created:\n  kind={:?}\n  id={}\n  pubkey={}\n  created_at={}\n  nonce={:?}",
        event.kind,
        event.id.as_hex_string(),
        event.pubkey.as_hex_string(),
        event.created_at,
        event.nonce_data()
    );
    println!(
        "nip34 git note event payload:\n  kind={:?}\n  id={}\n  pubkey={}\n  created_at={}\n  content={}",
        event.kind,
        event.id.as_hex_string(),
        event.pubkey.as_hex_string(),
        event.created_at,
        event.content
    );
    println!("nip34 git note event tags:");
    for tag in &event.tags {
        println!("  - {}", tag.0.join(":"));
    }

    Ok(event)
}

/// Return a tag value by name.
pub fn tag_value(event: &EventV3, tag_name: &str) -> Result<String, Error> {
    event
        .tags
        .iter()
        .find(|tag| tag.tagname() == tag_name)
        .map(|tag| tag.value().to_string())
        .ok_or(Error::TagMismatch)
}

/// Extract the commit id from a patch event.
pub fn get_commit_id_from_patch(event: &EventV3) -> Result<String, Error> {
    if let Ok(value) = tag_value(event, "commit") {
        return Ok(value);
    }

    if event.content.starts_with("From ") && event.content.len() > 45 {
        return Ok(event.content[5..45].to_string());
    }

    Err(Error::InvalidOperation)
}

/// Extract the parent commit id from a patch event.
pub fn get_parent_commit_from_patch(event: &EventV3) -> Result<String, Error> {
    if let Ok(value) = tag_value(event, "parent-commit") {
        return Ok(value);
    }

    if event.content.starts_with("From ") && event.content.len() > 45 {
        return Ok(event.content[5..45].to_string());
    }

    Err(Error::InvalidOperation)
}

/// Return the root event referenced by a reply or revision chain.
pub fn get_event_root(event: &EventV3) -> Result<Id, Error> {
    event
        .tags
        .iter()
        .find(|tag| tag.tagname() == "e" && matches!(tag.marker(), "root" | "revision-root" | "root-revision"))
        .ok_or(Error::TagMismatch)
        .and_then(|tag| tag.parse_event().map(|(id, _, _)| id))
}

/// Check whether the event is the root patch in a patch set.
pub fn event_is_patch_set_root(event: &EventV3) -> bool {
    event.kind == EventKind::Patches
        && event
            .tags
            .iter()
            .any(|tag| tag.tagname() == "e" && tag.marker() == "root")
}

/// Check whether the event is the root of a revision chain.
pub fn event_is_revision_root(event: &EventV3) -> bool {
    (event.kind == EventKind::Patches
        && event.tags.iter().any(|tag| {
            tag.tagname() == "e"
                && matches!(tag.marker(), "revision-root" | "root-revision")
        }))
        || (event.kind == EventKind::from(PULL_REQUEST_KIND)
            && event.tags.iter().any(|tag| tag.tagname() == "e"))
}

/// Check whether a patch event carries commit identifiers.
pub fn patch_supports_commit_ids(event: &EventV3) -> bool {
    if event.kind != EventKind::Patches {
        return false;
    }

    if event
        .tags
        .iter()
        .any(|tag| tag.tagname() == "commit-pgp-sig")
    {
        return true;
    }

    if event
        .tags
        .iter()
        .any(|tag| tag.tagname() == "parent-commit")
    {
        return true;
    }

    event.content.starts_with("From ") && event.content.len() > 45
}

/// Check whether an event is a valid PR or PR update.
pub fn event_is_valid_pr_or_pr_update(event: &EventV3) -> bool {
    [PULL_REQUEST_KIND, PULL_REQUEST_UPDATE_KIND]
        .iter()
        .map(|kind| EventKind::from(*kind))
        .any(|kind| kind == event.kind)
        && event
            .tags
            .iter()
            .any(|tag| tag.tagname() == "c" && git2::Oid::from_str(tag.value()).is_ok())
        && event.tags.iter().any(|tag| tag.tagname() == "clone")
}

/// Convert a NIP-19 or hex reference into a tag.
pub fn event_tag_from_nip19_or_hex(
    reference: &str,
    ref_type: EventRefType,
    allow_npub_reference: bool,
) -> Result<TagV3, Error> {
    let marker = match ref_type {
        EventRefType::Root => Some("root".to_string()),
        EventRefType::Reply => Some("reply".to_string()),
        EventRefType::Quote => None,
    };

    match Nip19::decode(reference) {
        Ok(Nip19::Event(event)) => {
            if ref_type == EventRefType::Quote {
                Ok(TagV3::new_quote(
                    event.event_id,
                    event
                        .relays
                        .first()
                        .map(|relay| UncheckedUrl::from_str(relay.as_str())),
                ))
            } else {
                Ok(TagV3::new_event(
                    event.event_id,
                    event
                        .relays
                        .first()
                        .map(|relay| UncheckedUrl::from_str(relay.as_str())),
                    marker,
                ))
            }
        }
        Ok(Nip19::EventId(id)) => {
            if ref_type == EventRefType::Quote {
                Ok(TagV3::new_quote(id, None))
            } else {
                Ok(TagV3::new_event(id, None, marker))
            }
        }
        Ok(Nip19::Address(addr)) => Ok(TagV3::new_address(
            &NAddr {
                d: addr.identifier,
                relays: addr
                    .relays
                    .into_iter()
                    .map(|relay| UncheckedUrl::from_str(relay.as_str()))
                    .collect(),
                kind: addr.kind,
                author: addr.public_key,
            },
            None,
        )),
        Ok(Nip19::Profile(profile)) if allow_npub_reference => Ok(TagV3::new_pubkey(
            profile.public_key,
            profile
                .relays
                .first()
                .map(|relay| UncheckedUrl::from_str(relay.as_str())),
            None,
        )),
        Ok(Nip19::PublicKey(public_key)) if allow_npub_reference => Ok(TagV3::new_pubkey(
            public_key,
            None,
            None,
        )),
        Ok(_) | Err(_) => Id::try_from_hex_string(reference)
            .map(|id| {
                if ref_type == EventRefType::Quote {
                    TagV3::new_quote(id, None)
                } else {
                    TagV3::new_event(id, None, marker)
                }
            })
            .map_err(Into::into),
    }
}

/// Repository announcement metadata.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepoRef {
    pub name: String,
    pub description: String,
    pub identifier: String,
    pub root_commit: String,
    pub git_server: Vec<String>,
    pub web: Vec<String>,
    pub relays: Vec<UncheckedUrl>,
    pub hashtags: Vec<String>,
    pub maintainers: Vec<PublicKey>,
    pub trusted_maintainer: PublicKey,
    pub events: HashMap<NAddr, EventV3>,
}

impl RepoRef {
    /// Convert the repository announcement into a signed event.
    pub fn to_event(&self, private_key: &PrivateKey) -> Result<EventV3, Error> {
        if self.root_commit.is_empty() {
            return Err(Error::InvalidOperation);
        }

        let identifier = if self.identifier.is_empty() {
            default_identifier(&self.root_commit)
        } else {
            self.identifier.clone()
        };

        let mut tags = vec![
            TagV3::new_identifier(identifier),
            root_commit_tag(&self.root_commit)?,
            TagV3::new_name(self.name.clone()),
            TagV3::new_tag("description", &self.description),
            TagV3::from_strings({
                let mut values = vec!["clone".to_string()];
                values.extend(self.git_server.iter().cloned());
                values
            }),
            TagV3::from_strings({
                let mut values = vec!["web".to_string()];
                values.extend(self.web.iter().cloned());
                values
            }),
            TagV3::from_strings({
                let mut values = vec!["relays".to_string()];
                values.extend(self.relays.iter().map(ToString::to_string));
                values
            }),
            TagV3::from_strings({
                let mut values = vec!["maintainers".to_string()];
                values.extend(self.maintainers.iter().map(PublicKey::as_hex_string));
                values
            }),
            TagV3::new_tag("alt", &format!("git repository: {}", self.name)),
        ];

        tags.extend(self.hashtags.iter().cloned().map(TagV3::new_hashtag));

        let preevent = PreEventV3 {
            pubkey: private_key.public_key(),
            created_at: Unixtime::now(),
            kind: repo_announcement_kind(),
            tags,
            content: "repo announcement".to_string(),
        };

        EventV3::sign_with_private_key(preevent, private_key)
    }

    pub fn coordinates(&self) -> HashSet<NAddr> {
        let mut res = HashSet::new();

        let _ = res.insert(self.coordinate_with_hint());
        for maintainer in &self.maintainers {
            let _ = res.insert(NAddr {
                d: self.identifier.clone(),
                relays: vec![],
                kind: repo_announcement_kind(),
                author: *maintainer,
            });
        }

        res
    }

    pub fn coordinate_with_hint(&self) -> NAddr {
        NAddr {
            d: self.identifier.clone(),
            relays: self.relays.first().cloned().into_iter().collect(),
            kind: repo_announcement_kind(),
            author: self.trusted_maintainer,
        }
    }

    pub fn coordinates_with_timestamps(&self) -> Vec<(NAddr, Option<Unixtime>)> {
        self.coordinates()
            .iter()
            .map(|coordinate| {
                (
                    coordinate.clone(),
                    self.events.get(coordinate).map(|event| event.created_at),
                )
            })
            .collect()
    }
}

impl TryFrom<(EventV3, Option<PublicKey>)> for RepoRef {
    type Error = Error;

    fn try_from(value: (EventV3, Option<PublicKey>)) -> Result<Self, Self::Error> {
        let (event, trusted_maintainer) = value;

        if event.kind != repo_announcement_kind() {
            return Err(Error::WrongEventKind);
        }

        let mut repo = RepoRef {
            name: String::new(),
            description: String::new(),
            identifier: String::new(),
            root_commit: String::new(),
            git_server: Vec::new(),
            web: Vec::new(),
            relays: Vec::new(),
            hashtags: Vec::new(),
            maintainers: Vec::new(),
            trusted_maintainer: trusted_maintainer.unwrap_or(event.pubkey),
            events: HashMap::new(),
        };

        for tag in &event.tags {
            match tag.0.as_slice() {
                [name, identifier, ..] if name == "d" => repo.identifier = identifier.clone(),
                [name, value, ..] if name == "name" => repo.name = value.clone(),
                [name, value, ..] if name == "description" => repo.description = value.clone(),
                [name, values @ ..] if name == "clone" => {
                    repo.git_server.clear();
                    for value in values {
                        unique_push(&mut repo.git_server, value.clone());
                    }
                }
                [name, values @ ..] if name == "web" => {
                    repo.web.clear();
                    for value in values {
                        unique_push(&mut repo.web, value.clone());
                    }
                }
                [name, commit_id] if name == "r" && Oid::from_str(commit_id).is_ok() => {
                    repo.root_commit = commit_id.clone();
                }
                [name, commit_id, marker] if name == "r" && marker == "euc" && Oid::from_str(commit_id).is_ok() => {
                    repo.root_commit = commit_id.clone();
                }
                [name, values @ ..] if name == "relays" => {
                    for relay in values {
                        unique_push(&mut repo.relays, UncheckedUrl::from_str(relay));
                    }
                }
                [name, hashtag, ..] if name == "t" => repo.hashtags.push(hashtag.clone()),
                [name, values @ ..] if name == "maintainers" => {
                    for value in values {
                        let maintainer = PublicKey::try_from_hex_string(value, true)?;
                        unique_push_public_key(&mut repo.maintainers, maintainer);
                    }
                }
                _ => {}
            }
        }

        if repo.identifier.is_empty() {
            return Err(Error::TagMismatch);
        }

        if repo.root_commit.is_empty() {
            return Err(Error::TagMismatch);
        }

        if repo.maintainers.is_empty() {
            repo.maintainers.push(event.pubkey);
        }

        let coordinate = repo.coordinate_for_event(event.pubkey);
        let _ = repo.events.insert(coordinate, event);

        Ok(repo)
    }
}

impl TryFrom<EventV3> for RepoRef {
    type Error = Error;

    fn try_from(event: EventV3) -> Result<Self, Self::Error> {
        Self::try_from((event, None))
    }
}

fn unique_push_public_key(values: &mut Vec<PublicKey>, value: PublicKey) {
    if !values.contains(&value) {
        values.push(value);
    }
}

impl RepoRef {
    fn coordinate_for_event(&self, author: PublicKey) -> NAddr {
        NAddr {
            d: self.identifier.clone(),
            relays: vec![],
            kind: repo_announcement_kind(),
            author,
        }
    }
}

/// Repository state snapshot.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepoState {
    pub identifier: String,
    pub state: HashMap<String, String>,
    pub event: EventV3,
}

impl RepoState {
    /// Build a signed repository state event from branch refs.
    pub fn try_from(mut state_events: Vec<EventV3>) -> Result<Self, Error> {
        if state_events.is_empty() {
            return Err(Error::InvalidOperation);
        }

        state_events.sort_by_key(|event| event.created_at);
        let event = state_events.last().cloned().ok_or(Error::InvalidOperation)?;

        if event.kind != repo_state_kind() {
            return Err(Error::WrongEventKind);
        }

        let mut state = HashMap::new();
        for tag in &event.tags {
            if let Some(name) = tag.0.first() {
                if ["refs/heads/", "refs/tags", "HEAD"]
                    .iter()
                    .any(|prefix| name.starts_with(prefix))
                {
                    if let Some(value) = tag.0.get(1) {
                        if Oid::from_str(value).is_ok() || value.contains("ref: refs/") {
                            let _ = state.insert(name.clone(), value.clone());
                        }
                    }
                }
            }
        }

        add_head(&mut state);

        let identifier = event
            .tags
            .iter()
            .find_map(|tag| tag.0.first().zip(tag.0.get(1)))
            .filter(|(name, _)| *name == "d")
            .map(|(_, value)| value.clone())
            .ok_or(Error::TagMismatch)?;

        Ok(RepoState {
            identifier,
            state,
            event,
        })
    }

    /// Build a signed repository state event.
    pub fn build(
        identifier: String,
        mut state: HashMap<String, String>,
        private_key: &PrivateKey,
    ) -> Result<Self, Error> {
        add_head(&mut state);

        let mut tags = vec![TagV3::new_identifier(identifier.clone())];
        let mut keys: Vec<_> = state.keys().cloned().collect();
        keys.sort();
        for key in keys {
            tags.push(TagV3::from_strings(vec![key.clone(), state[&key].clone()]));
        }

        let event = EventV3::sign_with_private_key(
            PreEventV3 {
                pubkey: private_key.public_key(),
                created_at: Unixtime::now(),
                kind: repo_state_kind(),
                tags,
                content: String::new(),
            },
            private_key,
        )?;

        Ok(RepoState {
            identifier,
            state,
            event,
        })
    }
}

fn add_head(state: &mut HashMap<String, String>) {
    if state.contains_key("HEAD") {
        return;
    }

    if state.contains_key("refs/heads/master") {
        let _ = state.insert("HEAD".to_string(), "ref: refs/heads/master".to_string());
    } else if state.contains_key("refs/heads/main") {
        let _ = state.insert("HEAD".to_string(), "ref: refs/heads/main".to_string());
    } else if let Some(key) = state.keys().find(|key| key.starts_with("refs/heads/")).cloned() {
        let _ = state.insert("HEAD".to_string(), format!("ref: {key}"));
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        env,
        fs,
        path::PathBuf,
        str::FromStr,
    };

    use actix_test::start;
    use git2::Oid;
    use gnostr_relay::App as GnostrRelayApp;
    use serial_test::serial;
    use tempfile::tempdir;

    use super::*;
    use crate::get_leading_zero_bits;

    struct EnvVarGuard {
        key: &'static str,
        value: Option<std::ffi::OsString>,
    }

    impl EnvVarGuard {
        fn set(key: &'static str, value: impl AsRef<std::ffi::OsStr>) -> Self {
            let previous = env::var_os(key);
            unsafe {
                env::set_var(key, value);
            }
            Self {
                key,
                value: previous,
            }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            match &self.value {
                Some(value) => unsafe {
                    env::set_var(self.key, value);
                },
                None => unsafe {
                    env::remove_var(self.key);
                },
            }
        }
    }

    fn create_relay_config() -> (tempfile::TempDir, PathBuf) {
        let config_dir = tempdir().expect("relay config dir");
        let config_path = config_dir.path().join("relay.toml");
        fs::write(
            &config_path,
            r#"
[server]
port = 0
host = "127.0.0.1"

[database]
path = ":memory:"
"#,
        )
        .expect("write relay config");
        (config_dir, config_path)
    }

    #[test]
    fn repo_ref_round_trip() {
        let private_key = PrivateKey::mock();
        let trusted_maintainer = private_key.public_key();
        let repo_ref = RepoRef {
            name: "gnostr".to_string(),
            description: "A git implementation on nostr".to_string(),
            identifier: "gnostr".to_string(),
            root_commit: "abcdef1234567890abcdef1234567890abcdef12".to_string(),
            git_server: vec!["https://github.com/gnostr-org/gnostr.git".to_string()],
            web: vec!["https://github.com/gnostr-org/gnostr".to_string()],
            relays: vec![UncheckedUrl::from_str("wss://relay.damus.io")],
            hashtags: vec!["gnostr".to_string()],
            maintainers: vec![trusted_maintainer],
            trusted_maintainer,
            events: HashMap::new(),
        };

        let event = repo_ref.to_event(&private_key).unwrap();
        let parsed = RepoRef::try_from((event.clone(), None)).unwrap();

        assert_eq!(event.kind, repo_announcement_kind());
        assert_eq!(parsed.identifier, repo_ref.identifier);
        assert_eq!(parsed.root_commit, repo_ref.root_commit);
        assert_eq!(parsed.name, repo_ref.name);
        assert_eq!(parsed.description, repo_ref.description);
        assert_eq!(parsed.git_server, repo_ref.git_server);
        assert_eq!(parsed.web, repo_ref.web);
        assert_eq!(parsed.relays, repo_ref.relays);
        assert_eq!(parsed.hashtags, repo_ref.hashtags);
        assert_eq!(parsed.maintainers, repo_ref.maintainers);
        assert_eq!(parsed.trusted_maintainer, repo_ref.trusted_maintainer);
        assert_eq!(parsed.events.len(), 1);
    }

    #[test]
    fn repo_ref_defaults_identifier_from_root_commit() {
        let private_key = PrivateKey::mock();
        let trusted_maintainer = private_key.public_key();
        let repo_ref = RepoRef {
            name: "gnostr".to_string(),
            description: "A git implementation on nostr".to_string(),
            identifier: String::new(),
            root_commit: "abcdef1234567890abcdef1234567890abcdef12".to_string(),
            git_server: vec![],
            web: vec![],
            relays: vec![],
            hashtags: vec![],
            maintainers: vec![trusted_maintainer],
            trusted_maintainer,
            events: HashMap::new(),
        };

        let event = repo_ref.to_event(&private_key).unwrap();
        let parsed = RepoRef::try_from(event).unwrap();

        assert_eq!(parsed.identifier, "abcdef1");
    }

    #[test]
    fn repo_ref_coordinates_include_relay_hint_and_all_maintainers() {
        let private_key = PrivateKey::mock();
        let trusted_maintainer = private_key.public_key();
        let other_maintainer = PrivateKey::mock().public_key();
        let repo_ref = RepoRef {
            name: "gnostr".to_string(),
            description: "A git implementation on nostr".to_string(),
            identifier: "gnostr".to_string(),
            root_commit: "abcdef1234567890abcdef1234567890abcdef12".to_string(),
            git_server: vec![],
            web: vec![],
            relays: vec![
                UncheckedUrl::from_str("wss://relay.damus.io"),
                UncheckedUrl::from_str("wss://blossom.gnostr.cloud"),
            ],
            hashtags: vec![],
            maintainers: vec![trusted_maintainer, other_maintainer],
            trusted_maintainer,
            events: HashMap::new(),
        };

        let hinted = repo_ref.coordinate_with_hint();
        assert_eq!(hinted.relays.len(), 1);
        assert_eq!(hinted.relays[0], UncheckedUrl::from_str("wss://relay.damus.io"));

        let coordinates = repo_ref.coordinates();
        assert_eq!(coordinates.len(), 2);
        assert!(coordinates.iter().any(|coordinate| coordinate.author == trusted_maintainer));
        assert!(coordinates.iter().any(|coordinate| coordinate.author == other_maintainer));
    }

    #[test]
    fn event_tag_from_nip19_or_hex_accepts_npub_when_allowed() {
        let public_key = PublicKey::mock();
        let tag = event_tag_from_nip19_or_hex(
            &public_key.as_bech32_string(),
            EventRefType::Root,
            true,
        )
        .unwrap();

        assert_eq!(tag.tagname(), "p");
        assert_eq!(tag.parse_pubkey().unwrap().0, public_key);
    }

    #[test]
    fn repo_state_round_trip_adds_head() {
        println!("[asyncgit] repo_state_round_trip_adds_head");
        let private_key = PrivateKey::mock();
        let mut state = HashMap::new();
        let _ = state.insert(
            "refs/heads/main".to_string(),
            "0123456789abcdef0123456789abcdef01234567".to_string(),
        );
        let _ = state.insert(
            "refs/tags/v0.1.0".to_string(),
            "89abcdef0123456789abcdef0123456789abcdef".to_string(),
        );

        let repo_state = RepoState::build("gnostr".to_string(), state, &private_key).unwrap();
        let parsed = RepoState::try_from(vec![repo_state.event.clone()]).unwrap();

        assert_eq!(parsed.identifier, "gnostr");
        assert_eq!(
            parsed.state.get("HEAD"),
            Some(&"ref: refs/heads/main".to_string())
        );
        assert_eq!(
            parsed.state.get("refs/heads/main"),
            Some(&"0123456789abcdef0123456789abcdef01234567".to_string())
        );
        assert_eq!(
            parsed.state.get("refs/tags/v0.1.0"),
            Some(&"89abcdef0123456789abcdef0123456789abcdef".to_string())
        );
        assert_eq!(parsed.event.kind, repo_state_kind());
    }

    fn note_fixture() -> GitNote {
        GitNote {
            note_id: Oid::from_str("b1d954d11c92c7386f040bba3937f24e64d8f9ec").unwrap(),
            annotated_id: Oid::from_str("431b84edc0d2fa118d63faa3c2db9c73d630a5ae").unwrap(),
            notes_ref: Some("refs/notes/commits".to_string()),
            message: "nip34:git note protocol example:deterministically linked git note"
                .to_string(),
            author: "randymcmillan".to_string(),
            committer: "randymcmillan".to_string(),
            committer_time: 1777759186,
        }
    }

    #[test]
    fn git_note_tags_reference_commit_and_notes_ref() {
        let note = note_fixture();
        let tags = git_note_tags(&note).unwrap();
        let expected_root = git_note_event_id(&note.annotated_id.to_string())
            .unwrap()
            .as_hex_string();

        assert!(tags.iter().any(|tag| tag.tagname() == "e" && tag.marker() == "root" && tag.value() == expected_root));
        assert!(tags.iter().any(|tag| tag.tagname() == "commit" && tag.value() == note.annotated_id.to_string()));
        assert!(tags.iter().any(|tag| tag.tagname() == "notes-ref" && tag.value() == "refs/notes/commits"));
        assert!(tags.iter().any(|tag| tag.tagname() == "weeble"));
        assert!(tags.iter().any(|tag| tag.tagname() == "blockheight"));
        assert!(tags.iter().any(|tag| tag.tagname() == "wobble"));
    }

    #[test]
    fn generate_git_note_event_uses_the_note_message() {
        println!("[asyncgit] generate_git_note_event_uses_the_note_message");
        let note = note_fixture();
        let private_key = PrivateKey::mock();
        let event = generate_git_note_event(&note, &private_key).unwrap();

        assert_eq!(event.kind, EventKind::Patches);
        assert_eq!(event.content, note.message);
        assert_eq!(event.created_at, Unixtime(note.committer_time));
    }

    #[test]
    fn generate_git_note_event_with_pow_adds_nonce() {
        println!("[asyncgit] generate_git_note_event_with_pow_adds_nonce");
        let note = note_fixture();
        let private_key = PrivateKey::mock();
        let event = generate_git_note_event_with_pow(&note, &private_key, 4).unwrap();

        assert_eq!(event.kind, EventKind::Patches);
        assert!(event.tags.iter().any(|tag| tag.tagname() == "nonce"));
        assert!(event.nonce_data().is_some());
        assert!(get_leading_zero_bits(&event.id.0) >= 4);
    }

    #[tokio::test]
    #[serial]
    async fn nip34_event_matrix_covers_all_kinds_and_git_notes() {
        println!("[asyncgit] nip34_event_matrix_covers_all_kinds_and_git_notes");

        let home_dir = tempdir().expect("home dir");
        let _home_guard = EnvVarGuard::set("HOME", home_dir.path());
        let _xdg_guard = EnvVarGuard::set("XDG_CONFIG_HOME", home_dir.path().join("config"));

        let (_relay_config_dir, relay_config_path) = create_relay_config();
        let relay_srv = start(move || {
            let app_data = GnostrRelayApp::create(
                Some(relay_config_path.to_str().expect("relay config path")),
                true,
                Some("NOSTR".to_owned()),
                None,
            )
            .expect("failed to create relay app");
            app_data.setting.write().add_nip(34);
            app_data.web_app()
        });
        let mut relay_url = relay_srv.url("/");
        relay_url = relay_url.replace("http", "ws");

        let private_key = PrivateKey::mock();
        let trusted_maintainer = private_key.public_key();
        let note_base = note_fixture();
        let root_commit = note_base.annotated_id.to_string();
        let repo_url = "https://github.com/gnostr-org/gnostr.git".to_string();
        let mut client = crate::nostr::Client::new(
            &crate::nostr::Keys::new(private_key.clone()),
            crate::nostr::Options::new().wait_for_send(true),
        );
        client
            .add_relays(vec![relay_url.clone()])
            .await
            .expect("add relay");
        client.connect().await;

        let repo_ref = RepoRef {
            name: "gnostr".to_string(),
            description: "A git implementation on nostr".to_string(),
            identifier: "gnostr".to_string(),
            root_commit: root_commit.clone(),
            git_server: vec![repo_url.clone()],
            web: vec!["https://github.com/gnostr-org/gnostr".to_string()],
            relays: vec![UncheckedUrl::from_str("wss://relay.damus.io")],
            hashtags: vec!["gnostr".to_string()],
            maintainers: vec![trusted_maintainer],
            trusted_maintainer,
            events: HashMap::new(),
        };

        let repo_announcement = repo_ref.to_event(&private_key).unwrap();
        let mut state = HashMap::new();
        let _ = state.insert(
            "refs/heads/main".to_string(),
            "0123456789abcdef0123456789abcdef01234567".to_string(),
        );
        let _ = state.insert(
            "refs/tags/v0.1.0".to_string(),
            "89abcdef0123456789abcdef0123456789abcdef".to_string(),
        );
        let repo_state = RepoState::build("gnostr".to_string(), state, &private_key).unwrap();
        let parsed_repo_state = RepoState::try_from(vec![repo_state.event.clone()]).unwrap();

        let root_patch = {
            let root_reference = Id::try_from_hex_string(
                "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            )
            .unwrap();
            signed_event(
                &private_key,
                EventKind::Patches,
                "From 0123456789abcdef0123456789abcdef01234567 Mon Sep 17 00:00:00 2001\nSubject: [PATCH 0/1] example title\n\nexample description",
                vec![
                    TagV3::new_event(root_reference, None, Some("root".to_string())),
                    TagV3::new_tag("commit", &root_commit),
                    TagV3::new_tag("clone", &repo_url),
                    TagV3::new_tag("description", "example description"),
                ],
                1_777_759_186,
            )
        };

        let pr_event = signed_event(
            &private_key,
            EventKind::from(PULL_REQUEST_KIND),
            "example description",
            vec![
                TagV3::new_event(root_patch.id, None, Some("root".to_string())),
                TagV3::new_tag("subject", "example title"),
                TagV3::new_tag("alt", "git Pull Request: example title"),
                TagV3::new_tag("branch-name", "feature/nip34"),
                TagV3::new_pubkey(trusted_maintainer, None, None),
                TagV3::new_tag("c", &root_commit),
                TagV3::new_tag("clone", &repo_url),
                TagV3::from_strings(vec![
                    "r".to_string(),
                    root_commit.clone(),
                    "euc".to_string(),
                ]),
            ],
            1_777_759_187,
        );

        let pr_update_event = signed_event(
            &private_key,
            EventKind::from(PULL_REQUEST_UPDATE_KIND),
            String::new(),
            vec![
                TagV3::new_tag("alt", "git Pull Request Update"),
                TagV3::from_strings(vec!["E".to_string(), pr_event.id.as_hex_string()]),
                TagV3::from_strings(vec!["P".to_string(), pr_event.pubkey.as_hex_string()]),
                TagV3::new_tag("c", &root_commit),
                TagV3::new_tag("clone", &repo_url),
                TagV3::from_strings(vec![
                    "r".to_string(),
                    root_commit.clone(),
                    "euc".to_string(),
                ]),
            ],
            1_777_759_188,
        );
        let issue_event = signed_event(
            &private_key,
            EventKind::from(GIT_ISSUE_KIND),
            "please provide feedback\nthis is an asyncgit issue used to exercise NIP-34".to_string(),
            vec![
                TagV3::new_tag("r", &root_commit),
                TagV3::from_strings(vec![
                    "a".to_string(),
                    format!("30617:{}:{}", trusted_maintainer.as_hex_string(), repo_ref.identifier),
                    repo_url.clone(),
                    "root".to_string(),
                ]),
                TagV3::new_pubkey(trusted_maintainer, None, None),
            ],
            1_777_759_188,
        );
        let reply_event = signed_event(
            &private_key,
            EventKind::from(GIT_REPLY_KIND),
            "replying to the asyncgit issue".to_string(),
            vec![
                TagV3::new_event(issue_event.id, None, Some("root".to_string())),
                TagV3::from_strings(vec![
                    "a".to_string(),
                    format!("30617:{}:{}", trusted_maintainer.as_hex_string(), repo_ref.identifier),
                    repo_url.clone(),
                    "reply".to_string(),
                ]),
                TagV3::new_pubkey(trusted_maintainer, None, None),
            ],
            1_777_759_188,
        );

        let status_open = signed_event(
            &private_key,
            EventKind::GitStatusOpen,
            String::new(),
            vec![
                TagV3::new_tag("alt", "git proposal status: open"),
                TagV3::new_tag("c", &root_commit),
                TagV3::new_tag("clone", &repo_url),
                TagV3::new_pubkey(trusted_maintainer, None, None),
            ],
            1_777_759_189,
        );
        let status_applied = signed_event(
            &private_key,
            EventKind::GitStatusApplied,
            String::new(),
            vec![
                TagV3::new_tag("alt", "git proposal status: applied"),
                TagV3::new_tag("c", &root_commit),
                TagV3::new_tag("clone", &repo_url),
                TagV3::new_pubkey(trusted_maintainer, None, None),
            ],
            1_777_759_190,
        );
        let status_draft = signed_event(
            &private_key,
            EventKind::GitStatusDraft,
            String::new(),
            vec![
                TagV3::new_tag("alt", "git proposal status: draft"),
                TagV3::new_tag("c", &root_commit),
                TagV3::new_tag("clone", &repo_url),
                TagV3::new_pubkey(trusted_maintainer, None, None),
            ],
            1_777_759_191,
        );
        let status_closed = signed_event(
            &private_key,
            EventKind::GitStatusClosed,
            String::new(),
            vec![
                TagV3::new_tag(
                    "alt",
                    "Git patch closed as forthcoming update is too large. Replacing with Pull Request",
                ),
                TagV3::new_event(pr_event.id, None, Some("root".to_string())),
                TagV3::new_tag("c", &root_commit),
                TagV3::new_tag("clone", &repo_url),
                TagV3::new_pubkey(trusted_maintainer, None, None),
            ],
            1_777_759_192,
        );

        let grasp_list = signed_event(
            &private_key,
            EventKind::from(USER_GRASP_LIST_KIND),
            String::new(),
            vec![
                TagV3::from_strings(vec!["g".to_string(), "wss://grasp.example.com".to_string()]),
                TagV3::from_strings(vec![
                    "g".to_string(),
                    "wss://another-grasp.example.com".to_string(),
                ]),
            ],
            1_777_759_193,
        );

        let carriers = vec![
            ("repo announcement", repo_announcement, repo_announcement_kind()),
            ("repo state", repo_state.event.clone(), repo_state_kind()),
            ("repo patch root", root_patch, EventKind::Patches),
            ("pull request", pr_event.clone(), EventKind::from(PULL_REQUEST_KIND)),
            (
                "pull request update",
                pr_update_event.clone(),
                EventKind::from(PULL_REQUEST_UPDATE_KIND),
            ),
            ("issue", issue_event, EventKind::from(GIT_ISSUE_KIND)),
            ("reply", reply_event, EventKind::from(GIT_REPLY_KIND)),
            ("status open", status_open, EventKind::GitStatusOpen),
            ("status applied", status_applied, EventKind::GitStatusApplied),
            ("status draft", status_draft, EventKind::GitStatusDraft),
            ("status closed", status_closed, EventKind::GitStatusClosed),
            ("user grasp list", grasp_list, EventKind::from(USER_GRASP_LIST_KIND)),
        ];

        let note_variants = [("plain git note", false), ("pow git note", true)];

        assert_eq!(parsed_repo_state.identifier, "gnostr");
        assert_eq!(parsed_repo_state.state.get("HEAD"), Some(&"ref: refs/heads/main".to_string()));

        for (carrier_label, carrier_event, expected_kind) in carriers {
            log_event_summary(carrier_label, &carrier_event);
            let published = client
                .send_event(carrier_event.clone())
                .await
                .expect("publish carrier event");
            assert_eq!(published, carrier_event.id);
            assert_eq!(carrier_event.kind, expected_kind);

            match carrier_label {
                "repo patch root" => {
                    assert!(event_is_patch_set_root(&carrier_event));
                    assert!(patch_supports_commit_ids(&carrier_event));
                }
                "pull request" => {
                    assert!(event_is_valid_pr_or_pr_update(&carrier_event));
                    assert!(event_is_revision_root(&carrier_event));
                }
                "pull request update" => {
                    assert!(event_is_valid_pr_or_pr_update(&carrier_event));
                    assert!(!event_is_revision_root(&carrier_event));
                }
                "issue" => {
                    assert!(carrier_event.tags.iter().any(|tag| tag.tagname() == "a"));
                    assert!(carrier_event.tags.iter().any(|tag| tag.tagname() == "p"));
                }
                "reply" => {
                    assert!(carrier_event.tags.iter().any(|tag| tag.tagname() == "e"));
                    assert!(carrier_event.tags.iter().any(|tag| tag.tagname() == "a"));
                    assert!(carrier_event.tags.iter().any(|tag| tag.tagname() == "p"));
                }
                "status closed" => {
                    assert!(carrier_event.tags.iter().any(|tag| tag.tagname() == "alt"));
                }
                "user grasp list" => {
                    assert_eq!(carrier_event.content, "");
                    assert!(carrier_event.tags.iter().any(|tag| tag.tagname() == "g"));
                }
                _ => {}
            }

            for (note_label, use_pow) in note_variants {
                let mut note = note_base.clone();
                note.message = format!("{carrier_label}: {note_label}");
                let git_note_event = if use_pow {
                    generate_git_note_event_with_pow(&note, &private_key, 4).unwrap()
                } else {
                    generate_git_note_event(&note, &private_key).unwrap()
                };

                log_event_summary(
                    &format!("{carrier_label} / {note_label}"),
                    &git_note_event,
                );
                let published = client
                    .send_event(git_note_event.clone())
                    .await
                    .expect("publish git note event");
                assert_eq!(published, git_note_event.id);
                assert_eq!(git_note_event.kind, EventKind::Patches);
                assert_eq!(git_note_event.content, note.message);
                assert_eq!(git_note_event.created_at, Unixtime(note.committer_time));
                assert!(git_note_event
                    .tags
                    .iter()
                    .any(|tag| tag.tagname() == "commit" && tag.value() == root_commit));
                assert!(git_note_event
                    .tags
                    .iter()
                    .any(|tag| tag.tagname() == "notes-ref" && tag.value() == "refs/notes/commits"));
                assert!(git_note_event
                    .tags
                    .iter()
                    .any(|tag| tag.tagname() == "e" && tag.marker() == "root"));

                if use_pow {
                    assert!(git_note_event.tags.iter().any(|tag| tag.tagname() == "nonce"));
                    assert!(git_note_event.nonce_data().is_some());
                } else {
                    assert!(git_note_event.nonce_data().is_none());
                }
            }
        }
    }

    #[test]
    fn repo_ref_requires_repo_kind() {
        let event = EventV3::new_dummy();
        assert!(matches!(
            RepoRef::try_from((event, None)),
            Err(Error::WrongEventKind)
        ));
    }

    #[test]
    fn patch_helpers_work() {
        let mut event = EventV3::new_dummy();
        event.kind = EventKind::Patches;
        event.content = "From 0123456789abcdef0123456789abcdef01234567 Mon Sep 17 00:00:00 2001".to_string();
        event.tags = vec![
            TagV3::new_event(
                Id::try_from_hex_string(
                    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
                )
                .unwrap(),
                None,
                Some("root".to_string()),
            ),
            TagV3::new_tag("commit", "abcdef1234567890abcdef1234567890abcdef12"),
            TagV3::new_tag("parent-commit", "fedcba9876543210fedcba9876543210fedcba98"),
            TagV3::new_tag("c", "0123456789abcdef0123456789abcdef01234567"),
            TagV3::new_tag("clone", "https://example.com/repo.git"),
        ];

        assert_eq!(
            tag_value(&event, "commit").unwrap(),
            "abcdef1234567890abcdef1234567890abcdef12"
        );
        assert_eq!(
            get_commit_id_from_patch(&event).unwrap(),
            "abcdef1234567890abcdef1234567890abcdef12"
        );
        assert!(patch_supports_commit_ids(&event));
        assert!(event_is_patch_set_root(&event));
        assert_eq!(status_kinds().len(), 4);

        let mut revision_event = EventV3::new_dummy();
        revision_event.kind = EventKind::Patches;
        revision_event.tags = vec![TagV3::new_event(
            Id::try_from_hex_string(
                "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            )
            .unwrap(),
            None,
            Some("revision-root".to_string()),
        )];
        assert!(event_is_revision_root(&revision_event));

        let mut pr_event = EventV3::new_dummy();
        pr_event.kind = EventKind::from(PULL_REQUEST_KIND);
        pr_event.tags = vec![
            TagV3::new_tag("c", "0123456789abcdef0123456789abcdef01234567"),
            TagV3::new_tag("clone", "https://example.com/repo.git"),
        ];
        assert!(event_is_valid_pr_or_pr_update(&pr_event));

        let note_ref = "note1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq";
        let _ = event_tag_from_nip19_or_hex("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef", EventRefType::Reply, false).unwrap();

        let nevent = NEvent::mock().as_bech32_string();
        let tag = event_tag_from_nip19_or_hex(&nevent, EventRefType::Quote, false).unwrap();
        assert_eq!(tag.tagname(), "q");

        let naddr = NAddr::mock().as_bech32_string();
        let tag = event_tag_from_nip19_or_hex(&naddr, EventRefType::Root, false).unwrap();
        assert_eq!(tag.tagname(), "a");

        let tag = event_tag_from_nip19_or_hex(note_ref, EventRefType::Root, false);
        assert!(tag.is_err());

        let root_tag = TagV3::new_event(
            Id::try_from_hex_string(
                "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            )
            .unwrap(),
            None,
            Some("root".to_string()),
        );
        let root_event = EventV3 {
            tags: vec![root_tag],
            ..EventV3::new_dummy()
        };
        assert!(get_event_root(&root_event).is_ok());
    }

    fn signed_event(
        private_key: &PrivateKey,
        kind: EventKind,
        content: impl Into<String>,
        tags: Vec<TagV3>,
        created_at: i64,
    ) -> EventV3 {
        EventV3::sign_with_private_key(
            PreEventV3 {
                pubkey: private_key.public_key(),
                created_at: Unixtime(created_at),
                kind,
                tags,
                content: content.into(),
            },
            private_key,
        )
        .unwrap()
    }

    fn log_event_summary(label: &str, event: &EventV3) {
        print_banner(label);
        let nonce = event.nonce_data();
        let kind_number = u32::from(event.kind);
        let kind_label = match kind_number {
            1617 => "Patches",
            1618 => "PullRequest",
            1619 => "PullRequestUpdate",
            1621 => "Issue",
            1622 => "Reply",
            10317 => "UserGraspList",
            30617 => "RepositoryAnnouncement",
            30618 => "RepositoryState",
            1630 => "StatusOpen",
            1631 => "StatusApplied",
            1632 => "StatusClosed",
            1633 => "StatusDraft",
            _ => "Other",
        };
        let tags = event
            .tags
            .iter()
            .map(|tag| tag.0.join(":"))
            .collect::<Vec<_>>();
        println!(
            "[asyncgit] {label}:\n  kind={}({kind_number})\n  id={}\n  pubkey={}\n  created_at={}\n  nonce={:?}\n  content={}",
            kind_label,
            event.id.as_hex_string(),
            event.pubkey.as_hex_string(),
            event.created_at,
            nonce,
            event.content
        );
        println!("[asyncgit] {label}: tags:");
        for tag in tags {
            println!("  - {tag}");
        }
    }

}
