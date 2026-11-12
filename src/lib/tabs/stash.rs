use anyhow::Result;
use crossterm::event::Event;
use gnostr_asyncgit::{sync::RepoPath, AsyncGitNotification};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Tabs},
};

use crate::{
    app::Environment,
    components::{
        visibility_blocking, CommandBlocking, CommandInfo, CommandText, Component,
        DrawableComponent, EventState,
    },
    keys::{key_match, SharedKeyConfig},
    queue::Action,
    strings,
};

use super::{StashList, Stashing};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StashSubTab {
    Files,
    Stashes,
}

pub struct StashTab {
    files_tab: Stashing,
    stashes_tab: StashList,
    active: StashSubTab,
    key_config: SharedKeyConfig,
}

impl StashTab {
    pub fn new(env: &Environment) -> Self {
        Self {
            files_tab: Stashing::new(env),
            stashes_tab: StashList::new(env),
            active: StashSubTab::Files,
            key_config: env.key_config.clone(),
        }
    }

    pub fn active_is_files(&self) -> bool {
        matches!(self.active, StashSubTab::Files)
    }

    pub fn select_files(&mut self) -> Result<()> {
        self.active = StashSubTab::Files;
        self.stashes_tab.hide();
        self.files_tab.show()
    }

    pub fn select_stashes(&mut self) -> Result<()> {
        self.active = StashSubTab::Stashes;
        self.files_tab.hide();
        self.stashes_tab.show()
    }

    pub fn update(&mut self) -> Result<()> {
        match self.active {
            StashSubTab::Files => self.files_tab.update(),
            StashSubTab::Stashes => self.stashes_tab.update(),
        }
    }

    pub fn update_git(&mut self, ev: AsyncGitNotification) -> Result<()> {
        match self.active {
            StashSubTab::Files => self.files_tab.update_git(ev),
            StashSubTab::Stashes => Ok(()),
        }
    }

    pub fn anything_pending(&self) -> bool {
        self.files_tab.anything_pending()
    }

    pub fn action_confirmed(&mut self, repo: &RepoPath, action: &Action) -> Result<()> {
        self.stashes_tab.action_confirmed(repo, action)
    }

    fn draw_tabs(&self, f: &mut ratatui::Frame, rect: ratatui::layout::Rect) {
        let titles = vec![
            Line::from(strings::stashing_files_title(&self.key_config)),
            Line::from(strings::stashlist_title(&self.key_config)),
        ];

        f.render_widget(
            Tabs::new(titles)
                .block(Block::default().borders(Borders::BOTTOM))
                .style(Style::default())
                .highlight_style(Style::default().add_modifier(Modifier::BOLD))
                .select(match self.active {
                    StashSubTab::Files => 0,
                    StashSubTab::Stashes => 1,
                }),
            rect,
        );
    }
}

impl DrawableComponent for StashTab {
    fn draw(&self, f: &mut ratatui::Frame, rect: ratatui::layout::Rect) -> Result<()> {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(1)].as_ref())
            .split(rect);

        self.draw_tabs(f, chunks[0]);
        match self.active {
            StashSubTab::Files => self.files_tab.draw(f, chunks[1])?,
            StashSubTab::Stashes => self.stashes_tab.draw(f, chunks[1])?,
        }
        Ok(())
    }
}

impl Component for StashTab {
    fn commands(&self, out: &mut Vec<CommandInfo>, force_all: bool) -> CommandBlocking {
        if self.is_visible() || force_all {
            match self.active {
                StashSubTab::Files => {
                    self.files_tab.commands(out, force_all);
                }
                StashSubTab::Stashes => {
                    self.stashes_tab.commands(out, force_all);
                }
            }

            out.push(CommandInfo::new(
                CommandText::new(
                    strings::tab_stashing(&self.key_config),
                    "switch to files to stash",
                    "-- Stash --",
                ),
                !self.active_is_files(),
                true,
            ));
            out.push(CommandInfo::new(
                CommandText::new(
                    strings::tab_stashes(&self.key_config),
                    "switch to stashes",
                    "-- Stash --",
                ),
                matches!(self.active, StashSubTab::Files),
                true,
            ));
        }

        visibility_blocking(self)
    }

    fn event(&mut self, ev: &Event) -> Result<EventState> {
        if self.is_visible() {
            let consumed = match self.active {
                StashSubTab::Files => self.files_tab.event(ev)?,
                StashSubTab::Stashes => self.stashes_tab.event(ev)?,
            };
            if consumed.is_consumed() {
                return Ok(EventState::Consumed);
            }

            if let Event::Key(k) = ev {
                if key_match(k, self.key_config.keys.tab_stashing) {
                    self.select_files()?;
                    return Ok(EventState::Consumed);
                } else if key_match(k, self.key_config.keys.tab_stashes) {
                    self.select_stashes()?;
                    return Ok(EventState::Consumed);
                }
            }
        }

        Ok(EventState::NotConsumed)
    }

    fn is_visible(&self) -> bool {
        match self.active {
            StashSubTab::Files => self.files_tab.is_visible(),
            StashSubTab::Stashes => self.stashes_tab.is_visible(),
        }
    }

    fn hide(&mut self) {
        self.files_tab.hide();
        self.stashes_tab.hide();
    }

    fn show(&mut self) -> Result<()> {
        match self.active {
            StashSubTab::Files => self.files_tab.show()?,
            StashSubTab::Stashes => self.stashes_tab.show()?,
        }

        Ok(())
    }
}
