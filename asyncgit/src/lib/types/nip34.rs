use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
    str::FromStr,
};

use git2::Oid;
use serde::{Deserialize, Serialize};

use super::{
    Error, EventKind, EventV3, Id, NAddr, NEvent, Nip19, PreEventV3, PrivateKey, PublicKey,
    TagV3, Unixtime, UncheckedUrl,
};

pub const REPO_ANNOUNCEMENT_KIND: u32 = 30617;
pub const REPO_STATE_KIND: u32 = 30618;
pub const PULL_REQUEST_KIND: u32 = 1618;
pub const PULL_REQUEST_UPDATE_KIND: u32 = 1619;
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

pub fn status_kinds() -> Vec<EventKind> {
    vec![
        EventKind::GitStatusOpen,
        EventKind::GitStatusApplied,
        EventKind::GitStatusClosed,
        EventKind::GitStatusDraft,
    ]
}

pub fn tag_value(event: &EventV3, tag_name: &str) -> Result<String, Error> {
    event
        .tags
        .iter()
        .find(|tag| tag.tagname() == tag_name)
        .map(|tag| tag.value().to_string())
        .ok_or(Error::TagMismatch)
}

pub fn get_commit_id_from_patch(event: &EventV3) -> Result<String, Error> {
    if let Ok(value) = tag_value(event, "commit") {
        return Ok(value);
    }

    if event.content.starts_with("From ") && event.content.len() > 45 {
        return Ok(event.content[5..45].to_string());
    }

    Err(Error::InvalidOperation)
}

pub fn get_parent_commit_from_patch(event: &EventV3) -> Result<String, Error> {
    if let Ok(value) = tag_value(event, "parent-commit") {
        return Ok(value);
    }

    if event.content.starts_with("From ") && event.content.len() > 45 {
        return Ok(event.content[5..45].to_string());
    }

    Err(Error::InvalidOperation)
}

pub fn get_event_root(event: &EventV3) -> Result<Id, Error> {
    event
        .tags
        .iter()
        .find(|tag| tag.tagname() == "e" && matches!(tag.marker(), "root" | "revision-root" | "root-revision"))
        .ok_or(Error::TagMismatch)
        .and_then(|tag| tag.parse_event().map(|(id, _, _)| id))
}

pub fn event_is_patch_set_root(event: &EventV3) -> bool {
    event.kind == EventKind::Patches
        && event
            .tags
            .iter()
            .any(|tag| tag.tagname() == "e" && tag.marker() == "root")
}

pub fn event_is_revision_root(event: &EventV3) -> bool {
    (event.kind == EventKind::Patches
        && event.tags.iter().any(|tag| {
            tag.tagname() == "e"
                && matches!(tag.marker(), "revision-root" | "root-revision")
        }))
        || (event.kind == EventKind::from(PULL_REQUEST_KIND)
            && event.tags.iter().any(|tag| tag.tagname() == "e"))
}

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

/// Repo announcement metadata.
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

/// Repo state snapshot.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepoState {
    pub identifier: String,
    pub state: HashMap<String, String>,
    pub event: EventV3,
}

impl RepoState {
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
    use super::*;

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
    fn repo_state_round_trip_adds_head() {
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
}
