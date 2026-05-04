//! Shared asyncgit-backed message types for chat integrations.
//!
//! This module mirrors the p2p message facade so broader chat flows can use the
//! same Nostr and git-note types without introducing another model.

pub use gnostr_asyncgit::types::*;

#[cfg(test)]
mod tests {
    use super::*;
    use gnostr_asyncgit::{git2::Oid, sync::NoteInfo};

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
        let note = NoteInfo {
            note_id: Oid::from_str("b1d954d11c92c7386f040bba3937f24e64d8f9ec").unwrap(),
            annotated_id: Oid::from_str("431b84edc0d2fa118d63faa3c2db9c73d630a5ae").unwrap(),
            notes_ref: Some("refs/notes/commits".to_string()),
            message: "chat git note".to_string(),
            author: "chat".to_string(),
            committer: "chat".to_string(),
            committer_time: 1777759186,
        };
        let private_key = PrivateKey::generate();

        let event = generate_git_note_event(&note, &private_key).expect("git note event");

        assert_eq!(event.kind, EventKind::Patches);
        assert!(event.tags.iter().any(|tag| tag.tagname() == "commit"));
        assert!(event.tags.iter().any(|tag| tag.tagname() == "notes-ref"));
    }
}
