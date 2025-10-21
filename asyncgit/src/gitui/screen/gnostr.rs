use super::Screen;
use crate::gitui::{config::Config, Res};
use ratatui::prelude::Size;
use std::rc::Rc;

pub(crate) fn create(config: Rc<Config>, size: Size) -> Res<Screen> {
    Screen::new(
        Rc::clone(&config),
        size,
        Box::new(move || {
            // TODO: Fetch gnostr events here
            Ok(vec![])
        }),
        super::ScreenType::Gnostr,
    )
}
