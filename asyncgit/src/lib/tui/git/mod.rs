#[path = "../../gitui/bindings/mod.rs"]
pub mod bindings;
#[path = "../../gitui/cli/mod.rs"]
pub mod cli;
#[path = "../../gitui/cmd_log/mod.rs"]
pub mod cmd_log;
#[path = "../../gitui/config/mod.rs"]
pub mod config;
#[path = "../../gitui/file_watcher/mod.rs"]
pub mod file_watcher;
#[path = "../../gitui/git/mod.rs"]
pub mod git;
#[path = "../../gitui/git2_opts/mod.rs"]
pub mod git2_opts;
#[path = "../../gitui/gitu_diff/mod.rs"]
pub mod gitu_diff;
#[path = "../../gitui/gitui_error/mod.rs"]
pub mod gitui_error;
#[path = "../../gitui/highlight/mod.rs"]
pub mod highlight;
#[path = "../../gitui/items/mod.rs"]
pub mod items;
#[path = "../../gitui/key_parser/mod.rs"]
pub mod key_parser;
#[path = "../../gitui/menu/mod.rs"]
pub mod menu;
#[path = "../../gitui/ops/mod.rs"]
pub mod ops;
#[path = "../../gitui/prompt/mod.rs"]
pub mod prompt;
#[path = "../../gitui/screen/mod.rs"]
pub mod screen;
#[path = "../../gitui/state/mod.rs"]
pub mod state;
#[path = "../../gitui/ui/mod.rs"]
pub mod ui;

#[cfg(test)]
#[path = "../../gitui/tests/mod.rs"]
pub mod tests;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventState, KeyModifiers};
use git2::Repository;
use gitui_error::Error;
use items::Item;
use std::{
    path::{Path, PathBuf},
    process::Command,
    rc::Rc,
    time::Duration,
};
use term::Term;

pub use crate::tui::shared::{syntax_parser, term};

///
pub const LOG_FILE_NAME: &str = "gnostr-asyncgit.log";

///
pub type Res<T> = Result<T, Error>;

///
pub fn run(args: &cli::Args, term: &mut Term) -> Res<()> {
    let dir = find_git_dir()?;
    let repo = open_repo(&dir)?;
    let config = Rc::new(config::init_config()?);

    let mut state = state::State::create(
        Rc::new(repo),
        term.size().map_err(Error::Term)?,
        args,
        config.clone(),
        true,
    )?;

    if let Some(keys_string) = &args.keys {
        let ("", keys) = key_parser::parse_keys(keys_string).expect("Couldn't parse keys") else {
            panic!("Couldn't parse keys");
        };

        for event in keys_to_events(&keys) {
            state.handle_event(term, event)?;
        }
    }

    state.redraw_now(term)?;

    if args.print {
        return Ok(());
    }

    state.run(term, Duration::from_millis(100))?;

    Ok(())
}

///
pub(crate) fn open_repo(dir: &Path) -> Res<Repository> {
    log::debug!("Opening repo");
    let repo = open_repo_from_env()?;
    repo.set_workdir(dir, false).map_err(Error::OpenRepo)?;
    Ok(repo)
}

///
pub(crate) fn find_git_dir() -> Res<PathBuf> {
    log::debug!("Finding git dir");
    let dir = PathBuf::from(
        String::from_utf8(
            Command::new("git")
                .args(["rev-parse", "--show-toplevel"])
                .output()
                .map_err(Error::FindGitDir)?
                .stdout,
        )
        .map_err(Error::GitDirUtf8)?
        .trim_end(),
    );
    Ok(dir)
}

///
fn open_repo_from_env() -> Res<Repository> {
    match Repository::open_from_env() {
        Ok(repo) => Ok(repo),
        Err(err) => Err(Error::OpenRepo(err)),
    }
}

///
pub(crate) fn keys_to_events(keys: &[(KeyModifiers, KeyCode)]) -> Vec<Event> {
    keys.iter()
        .map(|(mods, key)| {
            Event::Key(KeyEvent {
                code: *key,
                modifiers: *mods,
                kind: event::KeyEventKind::Press,
                state: KeyEventState::NONE,
            })
        })
        .collect::<Vec<_>>()
}
