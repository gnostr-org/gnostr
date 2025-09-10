use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    terminal::Frame,
    text::Text,
    widgets::{Block, Borders, Paragraph},
};

use crate::prelude::handlers::config::CompleteConfig;

pub fn draw_ui<T: Backend>(frame: &mut Frame<T>, config: &CompleteConfig) {
    let vertical_chunk_constraints = vec![Constraint::Min(1)];

    let margin = config.frontend.margin;
    let default_message = Text::from(String::from(config.frontend.default_message.to_owned()));

    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .horizontal_margin(margin)
        .vertical_margin(2)
        .constraints(vertical_chunk_constraints)
        .split(frame.size());

    let table =
        Paragraph::new(Text::from(default_message)).block(Block::default().borders(Borders::ALL));

    frame.render_widget(table, vertical_chunks[0]);
}
