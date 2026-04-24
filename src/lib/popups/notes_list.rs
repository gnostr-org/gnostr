use std::collections::BTreeSet;

use anyhow::Result;
use crossterm::event::{Event, KeyCode};
use gnostr_asyncgit::sync::{default_notes_ref, list_notes, remove_note, NoteInfo, RepoPathRef};
use ratatui::{
    layout::{Constraint, Margin, Rect},
    text::Span,
    widgets::{Block, BorderType, Borders, Cell, Clear, Row, Table, TableState},
    Frame,
};

use crate::{
    app::Environment,
    components::{
        visibility_blocking, CommandBlocking, CommandInfo, Component, DrawableComponent,
        EventState, ScrollType,
    },
    keys::{key_match, SharedKeyConfig},
    queue::{InternalEvent, Queue},
    strings,
    ui::{self, style::SharedTheme},
    utils::truncate_chars,
};

pub struct NotesListPopup {
    repo: RepoPathRef,
    theme: SharedTheme,
    queue: Queue,
    key_config: SharedKeyConfig,
    visible: bool,
    notes_ref: String,
    notes: Vec<NoteInfo>,
    table_state: std::cell::Cell<TableState>,
    current_height: std::cell::Cell<usize>,
    selected: BTreeSet<git2::Oid>,
}

impl NotesListPopup {
    pub fn new(env: &Environment) -> Self {
        Self {
            repo: env.repo.clone(),
            theme: env.theme.clone(),
            queue: env.queue.clone(),
            key_config: env.key_config.clone(),
            visible: false,
            notes_ref: String::new(),
            notes: Vec::new(),
            table_state: std::cell::Cell::new(TableState::default()),
            current_height: std::cell::Cell::new(0),
            selected: BTreeSet::new(),
        }
    }

    pub fn open(&mut self) -> Result<()> {
        self.notes_ref = default_notes_ref(&self.repo.borrow())?;
        self.selected.clear();
        self.refresh_notes()?;
        self.table_state
            .get_mut()
            .select((!self.notes.is_empty()).then_some(0));
        self.show()?;

        Ok(())
    }

    fn refresh_notes(&mut self) -> Result<()> {
        self.notes = list_notes(&self.repo.borrow(), Some(self.notes_ref.as_str()))?;
        self.selected
            .retain(|note_id| self.notes.iter().any(|note| note.annotated_id == *note_id));
        self.clamp_selection();
        Ok(())
    }

    fn clamp_selection(&mut self) {
        let mut table_state = self.table_state.take();
        let selected = table_state.selected();
        let selected = selected.and_then(|idx| {
            if self.notes.is_empty() {
                None
            } else {
                Some(idx.min(self.notes.len() - 1))
            }
        });
        table_state.select(selected);
        self.table_state.set(table_state);
    }

    fn current_note(&self) -> Option<&NoteInfo> {
        self.notes.get(self.current_index())
    }

    fn current_index(&self) -> usize {
        let table_state = self.table_state.take();
        let index = table_state.selected().unwrap_or(0);
        self.table_state.set(table_state);
        index
    }

    fn delete_selected(&mut self) -> Result<()> {
        let targets: Vec<git2::Oid> = if self.selected.is_empty() {
            self.current_note()
                .map(|note| vec![note.annotated_id])
                .unwrap_or_default()
        } else {
            self.selected.iter().copied().collect()
        };

        if targets.is_empty() {
            return Ok(());
        }

        for target in targets {
            remove_note(&self.repo.borrow(), target, Some(self.notes_ref.as_str()))?;
        }

        self.refresh_notes()?;
        Ok(())
    }

    fn amend_selected(&mut self) {
        let targets: Vec<_> = if self.selected.is_empty() {
            self.current_note()
                .map(|note| vec![note.annotated_id])
                .unwrap_or_default()
        } else {
            self.notes
                .iter()
                .filter(|note| self.selected.contains(&note.annotated_id))
                .map(|note| note.annotated_id)
                .collect()
        };

        if let Some((first, rest)) = targets.split_first() {
            self.hide();
            self.queue.push(InternalEvent::OpenGitNoteBatch(
                (*first).into(),
                Some(self.notes_ref.clone()),
                rest.iter().copied().map(Into::into).collect(),
            ));
        }
    }

    fn toggle_selected(&mut self) {
        if let Some(note_id) = self.current_note().map(|note| note.annotated_id) {
            if !self.selected.insert(note_id) {
                self.selected.remove(&note_id);
            }
        }
    }

    fn move_selection(&mut self, scroll_type: ScrollType) -> bool {
        let mut table_state = self.table_state.take();

        let old_selection = table_state.selected().unwrap_or(0);
        let max_selection = self.notes.len().saturating_sub(1);

        let new_selection = match scroll_type {
            ScrollType::Up => old_selection.saturating_sub(1),
            ScrollType::Down => old_selection.saturating_add(1).min(max_selection),
            ScrollType::Home => 0,
            ScrollType::End => max_selection,
            ScrollType::PageUp => {
                old_selection.saturating_sub(self.current_height.get().saturating_sub(1))
            }
            ScrollType::PageDown => old_selection
                .saturating_add(self.current_height.get().saturating_sub(1))
                .min(max_selection),
        };

        let needs_update = new_selection != old_selection;

        table_state.select(Some(new_selection));
        self.table_state.set(table_state);

        needs_update
    }

    fn get_rows(&self) -> Vec<Row<'_>> {
        self.notes.iter().map(|note| self.get_row(note)).collect()
    }

    fn get_row(&self, note: &NoteInfo) -> Row<'_> {
        let is_selected = self.selected.contains(&note.annotated_id);
        let note_id = note.note_id.to_string().chars().take(8).collect::<String>();
        let annotated_id = note
            .annotated_id
            .to_string()
            .chars()
            .take(8)
            .collect::<String>();
        let committer_time = crate::components::time_to_string(note.committer_time, true);
        let message = note
            .message
            .lines()
            .next()
            .map(|line| truncate_chars(line, 48))
            .unwrap_or_default();

        Row::new(vec![
            Cell::from(if is_selected {
                strings::symbol::CHECKMARK
            } else {
                " "
            })
            .style(self.theme.text_danger()),
            Cell::from(note_id).style(self.theme.commit_hash(false)),
            Cell::from(annotated_id).style(self.theme.commit_hash(false)),
            Cell::from(note.committer.clone()).style(self.theme.commit_author(false)),
            Cell::from(committer_time).style(self.theme.commit_time(false)),
            Cell::from(message).style(self.theme.text(true, false)),
        ])
    }
}

impl DrawableComponent for NotesListPopup {
    fn draw(&self, f: &mut Frame, rect: Rect) -> Result<()> {
        if self.visible {
            const PERCENT_SIZE: crate::ui::Size = crate::ui::Size::new(85, 60);
            const MIN_SIZE: crate::ui::Size = crate::ui::Size::new(70, 20);

            let area = ui::centered_rect(PERCENT_SIZE.width, PERCENT_SIZE.height, f.area());
            let area = ui::rect_inside(MIN_SIZE, f.area().into(), area);
            let area = area.intersection(rect);

            let rows = self.get_rows();
            let number_of_rows = rows.len();

            let table = Table::new(
                rows,
                [
                    Constraint::Length(1),
                    Constraint::Length(8),
                    Constraint::Length(8),
                    Constraint::Length(18),
                    Constraint::Length(10),
                    Constraint::Min(10),
                ],
            )
            .column_spacing(1)
            .highlight_style(self.theme.text(true, true))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(Span::styled(
                        format!("{} ({})", strings::title_notes(), self.notes_ref),
                        self.theme.title(true),
                    ))
                    .border_style(self.theme.block(true))
                    .border_type(BorderType::Thick),
            );

            let mut table_state = self.table_state.take();

            f.render_widget(Clear, area);
            f.render_stateful_widget(table, area, &mut table_state);

            let area = area.inner(Margin {
                vertical: 1,
                horizontal: 0,
            });

            ui::draw_scrollbar(
                f,
                area,
                &self.theme,
                number_of_rows,
                table_state.selected().unwrap_or(0),
                ui::Orientation::Vertical,
            );

            self.table_state.set(table_state);
            self.current_height.set(area.height.into());
        }

        Ok(())
    }
}

impl Component for NotesListPopup {
    fn commands(&self, out: &mut Vec<CommandInfo>, force_all: bool) -> CommandBlocking {
        if self.visible || force_all {
            if !force_all {
                out.clear();
            }

            let has_note = self.current_note().is_some();

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
            out.push(CommandInfo::new(
                strings::commands::select_note(),
                has_note,
                true,
            ));
            out.push(CommandInfo::new(
                strings::commands::amend_note(),
                has_note,
                true,
            ));
            out.push(CommandInfo::new(
                strings::commands::delete_note_popup(),
                has_note,
                true,
            ));
        }

        visibility_blocking(self)
    }

    fn event(&mut self, event: &Event) -> Result<EventState> {
        if self.visible {
            if let Event::Key(key) = event {
                if key_match(key, self.key_config.keys.exit_popup) {
                    self.hide();
                } else if key_match(key, self.key_config.keys.move_up) {
                    self.move_selection(ScrollType::Up);
                } else if key_match(key, self.key_config.keys.move_down) {
                    self.move_selection(ScrollType::Down);
                } else if key_match(key, self.key_config.keys.shift_up)
                    || key_match(key, self.key_config.keys.home)
                {
                    self.move_selection(ScrollType::Home);
                } else if key_match(key, self.key_config.keys.shift_down)
                    || key_match(key, self.key_config.keys.end)
                {
                    self.move_selection(ScrollType::End);
                } else if key_match(key, self.key_config.keys.page_down) {
                    self.move_selection(ScrollType::PageDown);
                } else if key_match(key, self.key_config.keys.page_up) {
                    self.move_selection(ScrollType::PageUp);
                } else if key.code == KeyCode::Char(' ') {
                    self.toggle_selected();
                } else if key.code == KeyCode::Char('a') {
                    self.amend_selected();
                } else if key.code == KeyCode::Char('d') {
                    self.delete_selected()?;
                }
            }

            Ok(EventState::Consumed)
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
