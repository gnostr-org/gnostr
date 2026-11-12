use anyhow::Result;
use crossterm::event::Event;
use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::{
    app::Environment,
    components::{visibility_blocking, CommandBlocking, CommandInfo, Component, DrawableComponent, EventState},
};

/// DockerTab
pub struct DockerTab {
    visible: bool,
}

impl DockerTab {
    /// new
    pub fn new(_env: &Environment) -> Self {
        Self { visible: false }
    }

    /// update
    pub fn update(&mut self) -> Result<()> {
        Ok(())
    }
}

impl DrawableComponent for DockerTab {
    fn draw(&self, f: &mut Frame, rect: Rect) -> Result<()> {
        if self.visible {
            let p = Paragraph::new("Docker")
                .block(Block::default().borders(Borders::ALL).title("Docker"));
            f.render_widget(p, rect);
        }
        Ok(())
    }
}

impl Component for DockerTab {
    fn commands(&self, _out: &mut Vec<CommandInfo>, _force_all: bool) -> CommandBlocking {
        visibility_blocking(self)
    }

    fn event(&mut self, _ev: &Event) -> Result<EventState> {
        if self.visible {
            return Ok(EventState::NotConsumed);
        }
        Ok(EventState::NotConsumed)
    }

    fn is_visible(&self) -> bool {
        self.visible
    }

    fn hide(&mut self) {
        self.visible = false;
    }

    fn show(&mut self) -> Result<()> {
        self.visible = true;
        self.update()?;
        Ok(())
    }
}
