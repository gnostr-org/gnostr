//! Shared Nostr and git-note message types for the middle layer.
//!
//! `gnostr-p2p` re-exports the canonical types from `gnostr_asyncgit`, and
//! `gnostr-chat` re-exports this module so the chain stays
//! `asyncgit -> p2p -> chat`.

pub use crate::git2::types::*;

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeMap,
        fs,
        path::Path,
    };

    use super::*;
    use gnostr_asyncgit::{
        sync::{add_note, commit, default_notes_ref, show_note, stage_add_file, RepoPath},
        types::{generate_git_note_event, PrivateKey},
    };
    use serde_json;
    use tempfile::tempdir;

    fn real_trace_fixture() -> (GitNote, Event, SubscriptionId, Filter) {
        let temp_dir = tempdir().expect("temp repo");
        let repo = gnostr_asyncgit::git2::Repository::init(temp_dir.path()).expect("init repo");
        {
            let mut config = repo.config().expect("repo config");
            config.set_str("user.name", "gnostr-trace").expect("user name");
            config
                .set_str("user.email", "trace@gnostr.org")
                .expect("user email");
        }

        let repo_path: RepoPath = temp_dir
            .path()
            .to_str()
            .expect("repo path")
            .into();

        let trace_file = temp_dir.path().join("trace.txt");
        fs::write(&trace_file, "real p2p trace").expect("write trace file");
        stage_add_file(&repo_path, Path::new("trace.txt")).expect("stage trace file");

        let commit_id = commit(&repo_path, "real p2p commit").expect("commit");
        let notes_ref = default_notes_ref(&repo_path).expect("default notes ref");
        add_note(
            &repo_path,
            commit_id,
            "real p2p note",
            Some(&notes_ref),
            false,
        )
        .expect("add note");

        let note = show_note(&repo_path, commit_id, Some(&notes_ref))
            .expect("show note")
            .expect("note exists");
        let git_note = GitNote::from(&note);
        let private_key = PrivateKey::generate();
        let event = generate_git_note_event(&git_note, &private_key).expect("git note event");

        let mut tags = BTreeMap::new();
        for tag in &event.tags {
            if let Some(letter) = tag.tagname().chars().next() {
                tags.entry(letter)
                    .or_insert_with(Vec::new)
                    .push(tag.value().to_string());
            }
        }

        let mut filter = Filter::default();
        filter.ids = vec![event.id.into()];
        filter.authors = vec![event.pubkey.into()];
        filter.kinds = vec![event.kind];
        filter.tags = tags;
        filter.since = Some(event.created_at);
        filter.until = Some(event.created_at);
        filter.limit = Some(event.tags.len());

        let subscription_id = SubscriptionId(event.id.as_hex_string());

        (git_note, event, subscription_id, filter)
    }

    #[test]
    fn real_asyncgit_note_is_reexported_and_serializes_through_p2p() {
        let (git_note, event, subscription_id, filter) = real_trace_fixture();

        let event_kind: EventKind = event.kind;
        let relay_message = RelayMessage::Event(subscription_id.clone(), Box::new(event.clone()));
        let client_message = ClientMessage::Req(subscription_id.clone(), vec![filter.clone()]);
        let typed_note: GitNote = git_note.clone();

        let relay_json = serde_json::to_string(&relay_message).expect("serialize relay message");
        let client_json = serde_json::to_string(&client_message).expect("serialize client message");
        let note_json = serde_json::to_string(&typed_note).expect("serialize git note");

        assert_eq!(event_kind, EventKind::Patches);
        assert_eq!(typed_note.note.message, event.content);
        assert_eq!(
            serde_json::from_str::<RelayMessage>(&relay_json).expect("deserialize relay message"),
            relay_message
        );
        assert_eq!(
            serde_json::from_str::<ClientMessage>(&client_json).expect("deserialize client message"),
            client_message
        );
        assert_eq!(
            serde_json::from_str::<GitNote>(&note_json).expect("deserialize git note"),
            typed_note
        );
        assert!(filter.ids.contains(&event.id.into()));
        assert!(filter.authors.contains(&event.pubkey.into()));
        assert!(filter.kinds.contains(&event.kind));
        assert_eq!(subscription_id.0, event.id.as_hex_string());
    }

    #[test]
    fn reexports_nip34_kinds_and_aliases_through_p2p() {
        let _nip34_kind: Nip34Kind = Nip34Kind::from(crate::git2::types::nip34::REPO_ANNOUNCEMENT_KIND);
        let _nip34_event: Option<Nip34Event> = None;
        let _nip34_unsigned: Option<Nip34UnsignedEvent> = None;

        assert_eq!(
            Nip34Kind::from(crate::git2::types::nip34::REPO_ANNOUNCEMENT_KIND),
            EventKind::RepositoryAnnouncement
        );
        assert_eq!(
            Nip34Kind::from(crate::git2::types::nip34::REPO_STATE_KIND),
            EventKind::GitRepoAnnouncement
        );
        assert_eq!(
            Nip34Kind::from(crate::git2::types::nip34::PULL_REQUEST_KIND),
            EventKind::Other(1618)
        );
        assert_eq!(
            Nip34Kind::from(crate::git2::types::nip34::PULL_REQUEST_UPDATE_KIND),
            EventKind::Other(1619)
        );
        assert_eq!(
            Nip34Kind::from(crate::git2::types::nip34::GIT_ISSUE_KIND),
            EventKind::GitIssue
        );
        assert_eq!(
            Nip34Kind::from(crate::git2::types::nip34::GIT_REPLY_KIND),
            EventKind::GitReply
        );
        assert_eq!(
            Nip34Kind::from(crate::git2::types::nip34::USER_GRASP_LIST_KIND),
            EventKind::Replaceable(10317)
        );

        let _ = (_nip34_kind, _nip34_event, _nip34_unsigned);
    }

    #[test]
    fn nip34_events_traverse_the_p2p_middle_layer() {
        let (git_note, event, subscription_id, filter) = real_trace_fixture();
        let asyncgit_event: gnostr_asyncgit::types::EventV3 = event.clone();
        let p2p_event: Nip34Event = asyncgit_event.clone();
        let unsigned_event: Nip34UnsignedEvent = Nip34UnsignedEvent {
            pubkey: event.pubkey,
            created_at: event.created_at,
            kind: event.kind,
            tags: event.tags.clone(),
            content: event.content.clone(),
        };

        let relay_message = RelayMessage::Event(subscription_id.clone(), Box::new(p2p_event.clone()));
        let client_message = ClientMessage::Req(subscription_id.clone(), vec![filter.clone()]);

        assert_eq!(unsigned_event.hash().expect("unsigned hash"), event.id);
        assert_eq!(asyncgit_event.kind, EventKind::Patches);
        assert_eq!(git_note.note.message, asyncgit_event.content);

        let event_json = serde_json::to_string(&p2p_event).expect("serialize p2p event");
        let unsigned_json = serde_json::to_string(&unsigned_event).expect("serialize unsigned event");
        let relay_json = serde_json::to_string(&relay_message).expect("serialize relay message");
        let client_json = serde_json::to_string(&client_message).expect("serialize client message");

        let decoded_event: Nip34Event = serde_json::from_str(&event_json).expect("deserialize p2p event");
        let decoded_unsigned: Nip34UnsignedEvent =
            serde_json::from_str(&unsigned_json).expect("deserialize unsigned event");
        let decoded_relay: RelayMessage = serde_json::from_str(&relay_json).expect("deserialize relay message");
        let decoded_client: ClientMessage = serde_json::from_str(&client_json).expect("deserialize client message");

        let asyncgit_round_trip_event: gnostr_asyncgit::types::EventV3 = decoded_event.clone();

        assert_eq!(decoded_event, p2p_event);
        assert_eq!(decoded_unsigned, unsigned_event);
        assert_eq!(asyncgit_round_trip_event, event);
        match decoded_relay {
            RelayMessage::Event(id, boxed_event) => {
                assert_eq!(id, subscription_id);
                assert_eq!(*boxed_event, p2p_event);
            }
            other => panic!("unexpected relay message: {:?}", other),
        }
        assert_eq!(decoded_client, client_message);
    }
}
