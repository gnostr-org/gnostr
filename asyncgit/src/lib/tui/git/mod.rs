mod bindings;
pub mod cli;
mod cmd_log;
mod file_watcher;
pub mod git;
mod git2_opts;
pub mod gitu_diff;
pub mod gitui_error;
mod highlight;
pub mod items;
mod key_parser;
pub mod menu;
pub mod ops;
mod prompt;
pub mod screen;
pub mod state;
pub mod ui;

#[cfg(test)]
mod tests;

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
use crate::tui::shared::{config, term::Term};

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
