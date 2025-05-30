//! gnostr: a git+nostr workflow utility and library
///
pub mod app;
///
pub mod args;
///
pub mod bug_report;
///
pub mod chat;
///
pub mod cli;
///
pub mod cli_interactor;
///
pub mod client;
///
pub mod clipboard;
///
pub mod cmdbar;
///
pub mod components;
///
pub mod git;
///
pub mod git_events;
///
pub mod global_rt;
///
pub mod gnostr;
///
pub mod input;
///
pub mod keys;
///
pub mod login;
///
pub mod notify_mutex;
///
pub mod options;
///
pub mod popup_stack;
///
pub mod popups;
///
pub mod queue;
///
pub mod repo_ref;
///
pub mod repo_state;
///
pub mod spinner;
///
pub mod ssh;
///
pub mod string_utils;
///
pub mod strings;
///
pub mod sub_commands;
///
pub mod tabs;
///
pub mod tui;
///
pub mod ui;
///
pub mod utils;
///
pub mod watcher;
///

/// simple-websockets
pub mod ws;

use anyhow::{anyhow, Result};
use directories::ProjectDirs;

///
pub fn get_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from("", "", "ngit").ok_or(anyhow!(
        "should find operating system home directories with rust-directories crate"
    ))
}
