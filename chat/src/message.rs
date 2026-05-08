//! Shared p2p-backed message types for chat integrations.
//!
//! Chat consumes Nostr and git-note types through the p2p facade, keeping the
//! protocol chain aligned as `asyncgit -> p2p -> chat`.

pub use gnostr_p2p::message::*;

#[cfg(test)]
mod tests {
    use super::*;
    use gnostr_p2p::message as p2p_message;

    #[test]
    fn reexports_p2p_nostr_types() {
        let _event_kind: EventKind = p2p_message::EventKind::TextNote;
        let _relay_message: RelayMessage = p2p_message::RelayMessage::Notice("ok".to_string());
        let _git_note: GitNote = p2p_message::GitNote {
            note: gnostr_asyncgit::sync::NoteInfo {
                note_id: gnostr_asyncgit::git2::Oid::from_str(
                    "b1d954d11c92c7386f040bba3937f24e64d8f9ec",
                )
                .unwrap(),
                annotated_id: gnostr_asyncgit::git2::Oid::from_str(
                    "431b84edc0d2fa118d63faa3c2db9c73d630a5ae",
                )
                .unwrap(),
                notes_ref: Some("refs/notes/commits".to_string()),
                message: "chat git note".to_string(),
                author: "chat".to_string(),
                committer: "chat".to_string(),
                committer_time: 1777759186,
            },
        };

        let _ = (_event_kind, _relay_message, _git_note);
    }

    #[test]
    fn exports_nostr_and_git_note_types() {
        let _event_kind = EventKind::TextNote;
        let _repo_ref: Option<RepoRef> = None;
        let _repo_state: Option<RepoState> = None;
        let _nip34_kind = Nip34Kind::from(REPO_ANNOUNCEMENT_KIND);
        let _nip34_event: Option<Nip34Event> = None;
        let _nip34_unsigned: Option<Nip34UnsignedEvent> = None;

        let _ = (
            _event_kind,
            _repo_ref,
            _repo_state,
            _nip34_kind,
            _nip34_event,
            _nip34_unsigned,
        );
    }

    #[test]
    fn builds_git_note_nostr_event() {
        let note = GitNote {
            note: gnostr_asyncgit::sync::NoteInfo {
                note_id: gnostr_asyncgit::git2::Oid::from_str(
                    "b1d954d11c92c7386f040bba3937f24e64d8f9ec",
                )
                .unwrap(),
                annotated_id: gnostr_asyncgit::git2::Oid::from_str(
                    "431b84edc0d2fa118d63faa3c2db9c73d630a5ae",
                )
                .unwrap(),
                notes_ref: Some("refs/notes/commits".to_string()),
                message: "chat git note".to_string(),
                author: "chat".to_string(),
                committer: "chat".to_string(),
                committer_time: 1777759186,
            },
        };
        let private_key = PrivateKey::generate();

        let event = generate_git_note_event(&note, &private_key).expect("git note event");

        assert_eq!(event.kind, EventKind::Patches);
        assert!(event.tags.iter().any(|tag| tag.tagname() == "commit"));
        assert!(event.tags.iter().any(|tag| tag.tagname() == "notes-ref"));
    }
}
