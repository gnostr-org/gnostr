#![allow(missing_docs)]
use anyhow::Result;
use crossterm::event::{Event, KeyCode};
use gnostr_asyncgit::sync::{add_note, list_notes, NoteInfo, RepoPathRef};
use git2::Oid;
use ratatui::{
    layout::{Alignment, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use tui_input::{backend::crossterm::EventHandler, Input};

use crate::{
    app::Environment,
    components::{CommandBlocking, CommandInfo},
    keys::{key_match, SharedKeyConfig},
    queue::Queue,
    strings,
    ui::style::SharedTheme,
};

#[derive(Default, PartialEq, Eq)]
enum InputMode {
    #[default]
    Normal,
    Editing,
}

/// Reusable notes editor/viewer for a selected git object.
pub struct NotesComponent {
    repo: RepoPathRef,
    queue: Queue,
    theme: SharedTheme,
    key_config: SharedKeyConfig,
    target: Option<Oid>,
    notes_ref: Option<String>,
    notes: Vec<NoteInfo>,
    input: Input,
    input_mode: InputMode,
}

impl NotesComponent {
    pub fn new(env: &Environment) -> Self {
        Self {
            repo: env.repo.clone(),
            queue: env.queue.clone(),
            theme: env.theme.clone(),
            key_config: env.key_config.clone(),
            target: None,
            notes_ref: None,
            notes: Vec::new(),
            input: Input::default(),
            input_mode: InputMode::Normal,
        }
    }

    pub fn clear(&mut self) {
        self.target = None;
        self.notes.clear();
        self.input.reset();
        self.input_mode = InputMode::Normal;
    }

    pub fn is_editing(&self) -> bool {
        self.input_mode == InputMode::Editing
    }

    pub fn set_target(&mut self, target: Option<Oid>) {
        if self.target != target {
            self.target = target;
            self.refresh();
        }
    }

    pub fn set_notes_ref(&mut self, notes_ref: Option<String>) {
        if self.notes_ref != notes_ref {
            self.notes_ref = notes_ref;
            self.refresh();
        }
    }

    fn refresh(&mut self) {
        let Some(commit_id) = self.target else {
            self.notes.clear();
            return;
        };

        let notes_result = {
            let repo = self.repo.borrow();
            list_notes(&repo, self.notes_ref.as_deref())
        };

        match notes_result {
            Ok(mut notes) => {
                notes.retain(|note| note.annotated_id == commit_id);
                notes.sort_by(|a, b| {
                    a.committer_time
                        .cmp(&b.committer_time)
                        .then_with(|| a.note_id.to_string().cmp(&b.note_id.to_string()))
                });
                self.notes = notes;
            }
            Err(err) => {
                log::error!("failed to load notes: {}", err);
                self.queue.push(crate::queue::InternalEvent::ShowErrorMsg(format!(
                    "failed to load notes:\n{}",
                    err
                )));
                self.notes.clear();
            }
        }
    }

    pub fn open_editor(&mut self) {
        self.refresh();

        if let Some(note) = self.notes.first() {
            self.input = Input::new(note.message.clone());
        } else {
            self.input.reset();
        }

        if self.target.is_some() {
            self.input_mode = InputMode::Editing;
        }
    }

    fn save(&mut self) -> Result<()> {
        let Some(target) = self.target else {
            self.queue.push(crate::queue::InternalEvent::ShowErrorMsg(
                "no commit selected".to_string(),
            ));
            return Ok(());
        };

        let note = self.input.value().to_string();
        {
            let repo = self.repo.borrow();
            add_note(
                &repo,
                target,
                note.as_str(),
                self.notes_ref.as_deref(),
                true,
            )?;
        }

        self.refresh();
        self.input.reset();
        self.input_mode = InputMode::Normal;
        Ok(())
    }

    pub fn event(&mut self, ev: &Event) -> Result<bool> {
        let consumed = match self.input_mode {
            InputMode::Normal => {
                if let Event::Key(k) = ev {
                    if key_match(k, self.key_config.keys.log_comment_commit)
                        || k.code == KeyCode::Char('n')
                        || k.code == KeyCode::Char('i')
                    {
                        self.open_editor();
                        return Ok(true);
                    }
                }

                false
            }
            InputMode::Editing => match ev {
                Event::Key(key) => match key.code {
                    KeyCode::Enter => {
                        self.save()?;
                        true
                    }
                    KeyCode::Esc => {
                        self.input_mode = InputMode::Normal;
                        self.input.reset();
                        true
                    }
                    _ => {
                        self.input.handle_event(ev);
                        true
                    }
                },
                _ => false,
            },
        };

        Ok(consumed)
    }

    pub fn commands(&self, out: &mut Vec<CommandInfo>, force_all: bool) -> CommandBlocking {
        match self.input_mode {
            InputMode::Normal => {
                out.push(CommandInfo::new(
                    strings::commands::note_open(),
                    self.target.is_some(),
                    self.target.is_some() || force_all,
                ));
                CommandBlocking::PassingOn
            }
            InputMode::Editing => {
                out.clear();
                out.push(CommandInfo::new(
                    strings::commands::note_save(),
                    true,
                    true,
                ));
                out.push(CommandInfo::new(
                    strings::commands::note_cancel(),
                    true,
                    true,
                ));
                CommandBlocking::Blocking
            }
        }
    }

    fn get_notes_text(&self, height: usize, width: usize) -> Vec<Line<'_>> {
        let mut txt: Vec<Line> = Vec::with_capacity(height);

        if self.notes.is_empty() {
            txt.push(Line::from("No notes"));
            return txt;
        }

        let notes_ref = self
            .notes
            .first()
            .and_then(|note| note.notes_ref.as_deref())
            .unwrap_or("refs/notes/commits");

        for note in self.notes.iter().take(height) {
            if txt.len() >= height {
                break;
            }

            let header = format!(
                "note@{} {} {}",
                note.note_id.to_string().chars().take(7).collect::<String>(),
                notes_ref,
                crate::components::time_to_string(note.committer_time, true)
            );
            txt.push(Line::from(vec![Span::styled(
                header,
                self.theme.commit_hash(false),
            )]));

            for line in note.message.lines() {
                if txt.len() >= height {
                    break;
                }

                let message = crate::utils::truncate_chars(line, width.saturating_sub(2));
                txt.push(Line::from(vec![Span::styled(
                    format!("  {}", message),
                    self.theme.text(true, false),
                )]));
            }
        }

        txt
    }

    pub fn draw(&self, f: &mut Frame, notes_area: Rect, input_area: Rect) {
        let notes_height = notes_area.height as usize;
        let notes_width = notes_area.width as usize;

        f.render_widget(
            Paragraph::new(self.get_notes_text(notes_height, notes_width))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Notes")
                        .border_style(self.theme.block(false)),
                )
                .alignment(Alignment::Left),
            notes_area,
        );

        let width = input_area.width.max(3) - 3;
        let scroll = self.input.visual_scroll(width as usize);
        let input = Paragraph::new(self.input.value())
            .style(match self.input_mode {
                InputMode::Normal => Style::default(),
                InputMode::Editing => Style::default().fg(ratatui::style::Color::Cyan),
            })
            .scroll((0, scroll as u16))
            .block(Block::default().borders(Borders::ALL).title("Note"));
        f.render_widget(input, input_area);

        if self.input_mode == InputMode::Editing {
            f.set_cursor_position((
                input_area.x + ((self.input.visual_cursor()).max(scroll) - scroll) as u16 + 1,
                input_area.y + 1,
            ));
        }
    }
}
