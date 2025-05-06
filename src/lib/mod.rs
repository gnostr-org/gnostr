//! ngit: a git+nostr command line utility and library
/// ngit::cli
pub mod cli;
/// ngit::cli_interactor
pub mod cli_interactor;
/// ngit::client
pub mod client;
/// ngit::git
pub mod git;
/// ngit::git_events
pub mod git_events;
pub mod global_rt;
/// ngit::login
pub mod login;

pub mod chat;

/// ngit::repo_ref
pub mod repo_ref;
/// ngit::repo_state
pub mod repo_state;
/// ngit::sub_commands
pub mod sub_commands;

pub mod utils;

pub mod app;
pub mod args;
pub mod bug_report;
pub mod clipboard;
pub mod cmdbar;
pub mod input;
pub mod notify_mutex;
pub mod options;
pub mod popup_stack;
pub mod queue;
pub mod spinner;
pub mod string_utils;
pub mod strings;
pub mod watcher;

pub mod components;
pub mod keys;
pub mod popups;
pub mod tabs;
pub mod tui;
pub mod ui;

//simple-websockets
pub mod ws;

use anyhow::{anyhow, Result};
use directories::ProjectDirs;

pub fn get_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from("", "", "ngit").ok_or(anyhow!(
        "should find operating system home directories with rust-directories crate"
    ))
}
