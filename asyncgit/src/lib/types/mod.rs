//! asyncgit compatibility surface for the shared Nostr types.

pub use gnostr_types::nostr::*;

pub mod nip34;

pub use nip34::{
    event_is_patch_set_root, event_is_revision_root, event_is_valid_pr_or_pr_update,
    event_tag_from_nip19_or_hex, generate_git_note_event, generate_git_note_event_with_pow,
    get_commit_id_from_patch, get_event_root, get_parent_commit_from_patch, git_note_tags,
    patch_supports_commit_ids, status_kinds, EventRefType, GitNote, Nip34Event, Nip34Kind,
    Nip34UnsignedEvent, RepoRef, RepoState, REPO_ANNOUNCEMENT_KIND, REPO_STATE_KIND,
};
