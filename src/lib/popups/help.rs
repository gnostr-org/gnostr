use std::{borrow::Cow, cmp};

use anyhow::Result;
use crossterm::event::Event;
use gnostr_asyncgit::hash;
use itertools::Itertools;
use serde_json::Value;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};
use ui::style::SharedTheme;

use crate::{
    app::Environment,
    components::{
        visibility_blocking, CommandBlocking, CommandInfo, Component, DrawableComponent, EventState,
    },
    keys::{key_match, SharedKeyConfig},
    strings, ui,
};

pub struct HelpPopup {
    cmds: Vec<CommandInfo>,
    visible: bool,
    selection: u16,
    theme: SharedTheme,
    key_config: SharedKeyConfig,
}

impl DrawableComponent for HelpPopup {
    fn draw(&self, f: &mut Frame, _rect: Rect) -> Result<()> {
        if self.visible {
            const SIZE: (u16, u16) = (65, 24);
            let scroll_threshold = SIZE.1 / 3;
            let scroll = self.selection.saturating_sub(scroll_threshold);

            let area = ui::centered_rect_absolute(SIZE.0, SIZE.1, f.area());

            f.render_widget(Clear, area);
            f.render_widget(
                Block::default()
                    .title(strings::help_title(&self.key_config))
                    .borders(Borders::ALL)
                    .border_type(BorderType::Thick),
                area,
            );

            let chunks = Layout::default()
                .vertical_margin(1)
                .horizontal_margin(1)
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(4)].as_ref())
                .split(area);

            f.render_widget(
                Paragraph::new(self.get_text())
                    .scroll((scroll, 0))
                    .alignment(Alignment::Left),
                chunks[0],
            );

            f.render_widget(
                Paragraph::new(self.get_footer_text())
                .alignment(Alignment::Right),
                chunks[1],
            );
        }

        Ok(())
    }
}

impl Component for HelpPopup {
    fn commands(&self, out: &mut Vec<CommandInfo>, force_all: bool) -> CommandBlocking {
        // only if help is open we have no other commands available
        if self.visible && !force_all {
            out.clear();
        }

        if self.visible {
            out.push(CommandInfo::new(
                strings::commands::scroll(&self.key_config),
                true,
                true,
            ));

            out.push(CommandInfo::new(
                strings::commands::close_popup(&self.key_config),
                true,
                true,
            ));
        }

        if !self.visible || force_all {
            out.push(
                CommandInfo::new(strings::commands::help_open(&self.key_config), true, true)
                    .order(99),
            );
        }

        visibility_blocking(self)
    }

    fn event(&mut self, ev: &Event) -> Result<EventState> {
        if self.visible {
            if let Event::Key(e) = ev {
                if key_match(e, self.key_config.keys.exit_popup) {
                    self.hide();
                } else if key_match(e, self.key_config.keys.move_down) {
                    self.move_selection(true);
                } else if key_match(e, self.key_config.keys.move_up) {
                    self.move_selection(false);
                }
            }

            Ok(EventState::Consumed)
        } else if let Event::Key(k) = ev {
            if key_match(k, self.key_config.keys.open_help) {
                self.show()?;
                Ok(EventState::Consumed)
            } else {
                Ok(EventState::NotConsumed)
            }
        } else {
            Ok(EventState::NotConsumed)
        }
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

impl HelpPopup {
    pub fn new(env: &Environment) -> Self {
        Self {
            cmds: vec![],
            visible: false,
            selection: 0,
            theme: env.theme.clone(),
            key_config: env.key_config.clone(),
        }
    }
    pub fn set_cmds(&mut self, cmds: Vec<CommandInfo>) {
        self.cmds = cmds
            .into_iter()
            .filter(|e| !e.text.hide_help)
            .collect::<Vec<_>>();
        self.cmds.sort_by_key(|e| e.text.clone());
        self.cmds.dedup_by_key(|e| e.text.clone());
        self.cmds.sort_by_key(|e| hash(&e.text.group));
    }

    fn move_selection(&mut self, inc: bool) {
        let mut new_selection = self.selection;

        new_selection = if inc {
            new_selection.saturating_add(1)
        } else {
            new_selection.saturating_sub(1)
        };
        new_selection = cmp::max(new_selection, 0);

        if let Ok(max) = u16::try_from(self.cmds.len().saturating_sub(1)) {
            self.selection = cmp::min(new_selection, max);
        }
    }

    fn get_text(&self) -> Vec<Line<'_>> {
        let mut txt: Vec<Line> = Vec::new();

        let mut processed = 0_u16;

        for (key, group) in &self.cmds.iter().chunk_by(|e| e.text.group) {
            txt.push(Line::from(Span::styled(
                Cow::from(key.to_string()),
                Style::default().add_modifier(Modifier::REVERSED),
            )));

            for command_info in group {
                let is_selected = self.selection == processed;

                processed += 1;

                txt.push(Line::from(Span::styled(
                    Cow::from(if is_selected {
                        format!(">{}", command_info.text.name)
                    } else {
                        format!(" {}", command_info.text.name)
                    }),
                    self.theme.text(true, is_selected),
                )));

                if is_selected {
                    txt.push(Line::from(Span::styled(
                        Cow::from(format!("  {}\n", command_info.text.desc)),
                        self.theme.text(true, is_selected),
                    )));
                }
            }
        }

        txt
    }

    fn get_footer_text(&self) -> Vec<Line<'static>> {
        let metadata = app_metadata();
        let app_name = metadata
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("gnostr");
        let description = metadata
            .get("description")
            .and_then(Value::as_str)
            .unwrap_or("");
        let repository = metadata
            .get("repository")
            .and_then(Value::as_str)
            .unwrap_or("");
        let build_name = metadata
            .get("build_name")
            .and_then(Value::as_str)
            .unwrap_or(env!("GITUI_BUILD_NAME"));

        let mut lines = vec![Line::from(Span::styled(
            Cow::from("gnostr-tui"),
            Style::default(),
        ))];
        lines.push(Line::from(Span::styled(
            Cow::from(build_name.to_string()),
            Style::default(),
        )));
        lines.push(Line::from(Span::styled(
            Cow::from(app_name.to_string()),
            Style::default(),
        )));
        if !description.is_empty() {
            lines.push(Line::from(Span::styled(
                Cow::from(description.to_string()),
                Style::default(),
            )));
        } else if !repository.is_empty() {
            lines.push(Line::from(Span::styled(
                Cow::from(repository.to_string()),
                Style::default(),
            )));
        }

        lines
    }
}

fn app_metadata() -> Value {
    serde_json::from_str(env!("GITUI_APP_METADATA_JSON"))
        .unwrap_or_else(|error| panic!("invalid GITUI_APP_METADATA_JSON: {error}"))
}
