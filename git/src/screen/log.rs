use std::rc::Rc;

use git2::{Oid, Repository};
use ratatui::prelude::Rect;

use super::Screen;
use crate::config::Config;
use crate::items::log;
use crate::Res;

pub(crate) fn create(
    config: Rc<Config>,
    repo: Rc<Repository>,
    size: Rect,
    rev: Option<Oid>,
) -> Res<Screen> {
    Screen::new(
        Rc::clone(&config),
        size,
        Box::new(move || log(&config, &repo, usize::MAX, rev)),
    )
}
