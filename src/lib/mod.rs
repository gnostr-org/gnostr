pub mod cli_interactor;
pub mod client;
pub mod git;
pub mod git_events;
pub mod login;
pub mod repo_ref;
pub mod repo_state;
pub mod sub_commands;
pub mod utils;
pub mod core;

// Added missing modules
pub mod cli;
pub mod dashboard;
pub mod p2p;

// Re-export from gnostr_asyncgit
pub use gnostr_asyncgit::blockheight;
pub use gnostr_asyncgit::weeble;
pub use gnostr_asyncgit::wobble;

use anyhow::{Result, anyhow};
use directories::ProjectDirs;

pub fn get_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from("", "", "ngit").ok_or(anyhow!(
        "should find operating system home directories with rust-directories crate"
    ))
}
