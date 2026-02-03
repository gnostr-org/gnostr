//! asyncgit

#![allow(missing_docs)]
#![allow(
    unused_imports,
    unused_must_use,
    dead_code,
    unstable_name_collisions,
    unused_assignments
)]
#![allow(clippy::all, clippy::perf, clippy::nursery, clippy::pedantic)]
#![allow(
	clippy::filetype_is_file,
	clippy::cargo,
	clippy::unwrap_used,
	clippy::panic,
	clippy::match_like_matches_macro,
	clippy::needless_update
	//TODO: get this in someday since expect still leads us to crashes sometimes
	// clippy::expect_used
)]
#![allow(
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::missing_errors_doc
)]
//TODO:
#![allow(
    clippy::significant_drop_tightening,
    clippy::missing_panics_doc,
    clippy::multiple_crate_versions,
    clippy::needless_pass_by_ref_mut,
    clippy::too_long_first_doc_paragraph,
    clippy::set_contains_or_insert,
    clippy::empty_docs
)]

/// pub mod css
pub mod css;
/// pub mod js
pub mod js;
/// pub mod theme
pub mod theme;

/// pub mod web
pub mod web;

/// pub mod gitui
pub mod gitui;

/// pub mod gnostr
pub mod gnostr;

pub mod asyncjob;
mod blame;
mod branches;
pub mod cached;
mod commit_files;
mod diff;
mod error;
mod fetch_job;
mod filter_commits;
mod progress;
mod pull;
mod push;
mod push_tags;
pub mod remote_progress;
pub mod remote_tags;
mod revlog;
mod status;
pub mod sync;
mod tags;
mod treefiles;

use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

pub use git2::message_prettify;

pub use crate::{
    blame::{AsyncBlame, BlameParams},
    branches::AsyncBranchesJob,
    commit_files::{AsyncCommitFiles, CommitFilesParams},
    diff::{AsyncDiff, DiffParams, DiffType},
    error::{Error, Result},
    fetch_job::AsyncFetchJob,
    filter_commits::{AsyncCommitFilterJob, CommitFilterResult},
    progress::ProgressPercent,
    pull::{AsyncPull, FetchRequest},
    push::{AsyncPush, PushRequest},
    push_tags::{AsyncPushTags, PushTagsRequest},
    remote_progress::{RemoteProgress, RemoteProgressState},
    revlog::{AsyncLog, FetchStatus},
    status::{AsyncStatus, StatusParams},
    sync::{
        diff::{DiffLine, DiffLineType, FileDiff},
        remotes::push::PushType,
        status::{StatusItem, StatusItemType},
    },
    tags::AsyncTags,
    treefiles::AsyncTreeFilesJob,
};

// Re-export web-related constants and modules for Askama templates
pub use crate::web::{CRATE_VERSION, GLOBAL_CSS_HASH, GNOSTR_SVG_HASH, LOADER_FRAGMENT_SVG_HASH, LOGO_INVERTED_SVG_HASH, LOGO_SVG_HASH, HOME_SVG_HASH, HOME_ACTIVE_SVG_HASH, MESSAGES_SVG_HASH, MESSAGES_ACTIVE_SVG_HASH, NOTIFICATIONS_SVG_HASH, NOTIFICATIONS_ACTIVE_SVG_HASH, SETTINGS_SVG_HASH, SETTINGS_ACTIVE_SVG_HASH, NEW_NOTE_SVG_HASH, NO_USER_SVG_HASH, PROFILE_WEBSITE_SVG_HASH, PROFILE_ZAP_SVG_HASH, MESSAGE_USER_SVG_HASH, PUBKEY_SVG_HASH, ADD_RELAY_SVG_HASH, CLOSE_MODAL_SVG_HASH, EVENT_LIKE_SVG_HASH, EVENT_LIKED_SVG_HASH, EVENT_DELETE_SVG_HASH, EVENT_REPLY_SVG_HASH, EVENT_SHARE_SVG_HASH, EVENT_OPTIONS_SVG_HASH, GNOSTR_NOTIF_SVG_HASH, JS_BUNDLE_HASH, HIGHLIGHT_CSS_HASH, DARK_HIGHLIGHT_CSS_HASH};
pub use crate::web::layers;
pub use crate::web::git as git;

/// this type is used to communicate events back through the channel
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AsyncGitNotification {
    /// this indicates that no new state was fetched but that a async
    /// process finished
    FinishUnchanged,
    ///
    Status,
    ///
    Diff,
    ///
    Log,
    ///
    FileLog,
    ///
    CommitFiles,
    ///
    Tags,
    ///
    Push,
    ///
    PushTags,
    ///
    Pull,
    ///
    Blame,
    ///
    RemoteTags,
    ///
    Fetch,
    ///
    Branches,
    ///
    TreeFiles,
    ///
    CommitFilter,
}

/// helper function to calculate the hash of an arbitrary type that
/// implements the `Hash` trait
pub fn hash<T: Hash + ?Sized>(v: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    v.hash(&mut hasher);
    hasher.finish()
}

///
#[cfg(feature = "trace-libgit")]
pub fn register_tracing_logging() -> bool {
    fn git_trace(level: git2::TraceLevel, msg: &[u8]) {
        if let Ok(msg) = std::str::from_utf8(msg) {
            log::info!("[{:?}]: {}", level, msg);
        }
    }
    git2::trace_set(git2::TraceLevel::Trace, git_trace).is_ok()
}

///
#[cfg(not(feature = "trace-libgit"))]
pub fn register_tracing_logging() -> bool {
    true
}
