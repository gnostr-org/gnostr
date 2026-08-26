use crate::{tui::git::gitui_error::Error, tui::git::Res};
use git2::{Repository, StatusOptions};

pub fn status(repo: &Repository) -> Res<StatusOptions> {
    let mut opts = StatusOptions::new();

    opts.include_untracked(
        repo.config()
            .map_err(Error::ReadGitConfig)?
            .get_bool("status.showUntrackedFiles")
            .ok()
            .unwrap_or(true),
    );

    Ok(opts)
}
