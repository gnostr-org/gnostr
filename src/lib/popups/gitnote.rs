use std::{io, process::Command};

use anyhow::Result;
use crossterm::{
    event::Event,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use gnostr_asyncgit::sync::{default_notes_ref, utils::repo_work_dir, CommitId, RepoPath};
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};
use scopeguard::defer;

use crate::{
    app::Environment,
    components::{
        visibility_blocking, CommandBlocking, CommandInfo, Component, DrawableComponent, EventState,
    },
    keys::SharedKeyConfig,
    strings,
    ui::{self, style::SharedTheme},
};

pub struct GitnotePopup {
    visible: bool,
    theme: SharedTheme,
    key_config: SharedKeyConfig,
}

impl GitnotePopup {
    pub fn new(env: &Environment) -> Self {
        Self {
            visible: false,
            theme: env.theme.clone(),
            key_config: env.key_config.clone(),
        }
    }

    pub fn open_note_in_editor(
        repo: &RepoPath,
        oid: &CommitId,
        notes_ref: Option<&str>,
    ) -> Result<()> {
        let work_dir = repo_work_dir(repo)?;
        let notes_ref = notes_ref
            .map(str::to_owned)
            .unwrap_or(default_notes_ref(repo)?);

        io::stdout().execute(LeaveAlternateScreen)?;
        defer! {
            io::stdout().execute(EnterAlternateScreen).expect("reset terminal");
        }

        Command::new("git")
            .current_dir(work_dir)
            .args(["notes", "--ref", &notes_ref, "edit", "--allow-empty", &oid.to_string()])
            .status()?;

        Ok(())
    }
}

impl DrawableComponent for GitnotePopup {
    fn draw(&self, f: &mut Frame, _rect: Rect) -> Result<()> {
        if self.visible {
            let txt = Line::from(
                strings::msg_opening_editor(&self.key_config)
                    .split('\n')
                    .map(|string| Span::raw::<String>(string.to_string()))
                    .collect::<Vec<Span>>(),
            );

            let area = ui::centered_rect_absolute(25, 3, f.area());
            f.render_widget(Clear, area);
            f.render_widget(
                Paragraph::new(txt)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_type(BorderType::Thick)
                            .border_style(self.theme.block(true)),
                    )
                    .style(self.theme.block(true)),
                area,
            );
        }

        Ok(())
    }
}

impl Component for GitnotePopup {
    fn commands(&self, out: &mut Vec<CommandInfo>, force_all: bool) -> CommandBlocking {
        if self.visible && !force_all {
            out.clear();
        }

        visibility_blocking(self)
    }

    fn event(&mut self, _ev: &Event) -> Result<EventState> {
        if self.visible {
            return Ok(EventState::Consumed);
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
        Ok(())
    }
}
