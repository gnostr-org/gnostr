//! Shared asyncgit-backed message types for chat integrations.
//!
//! This module mirrors the p2p message facade so broader chat flows can use the
//! same Nostr and git-note types without introducing another model.

pub use gnostr_asyncgit::types::*;

#[cfg(test)]
mod tests {
    use super::*;

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
}
