#![allow(missing_docs)]
use anyhow::Result;
use crossterm::event::Event;
use gnostr_asyncgit::{
    sync::{default_notes_ref, NoteInfo, RepoPathRef},
    AsyncGitNotification, AsyncNotes,
};
use git2::Oid;
use std::time::Duration;
use ratatui::{
    layout::{Alignment, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::{
    app::Environment,
    components::{CommandBlocking, CommandInfo},
    keys::SharedKeyConfig,
    queue::Queue,
    strings,
    ui::style::SharedTheme,
};

/// Reusable notes editor/viewer for a selected git object.
pub struct NotesComponent {
    repo: RepoPathRef,
    queue: Queue,
    theme: SharedTheme,
    key_config: SharedKeyConfig,
    // Mirrors the repository snapshot that was last delivered through the
    // async notes notification path.
    async_notes: AsyncNotes,
    target: Option<Oid>,
    loaded_target: Option<Oid>,
    notes_ref: Option<String>,
    // Keep only the last delivered snapshot here; the queue/update path is
    // responsible for refreshing it when async work completes.
    notes: Vec<NoteInfo>,
}

impl NotesComponent {
    pub fn new(env: &Environment) -> Self {
        Self {
            repo: env.repo.clone(),
            queue: env.queue.clone(),
            theme: env.theme.clone(),
            key_config: env.key_config.clone(),
            async_notes: AsyncNotes::new(env.repo.borrow().clone(), &env.sender_git),
            target: None,
            loaded_target: None,
            notes_ref: None,
            notes: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.target = None;
        self.loaded_target = None;
        self.notes.clear();
    }

    pub fn set_target(&mut self, target: Option<Oid>) {
        if self.target != target {
            self.target = target;
            self.request_refresh();
        }
    }

    pub fn set_notes_ref(&mut self, notes_ref: Option<String>) {
        if self.notes_ref != notes_ref {
            self.notes_ref = notes_ref;
            self.request_refresh();
        }
    }

    /// Queue a notes fetch and let the async notification path deliver the
    /// new snapshot later.
    fn request_refresh(&mut self) {
        let Some(_) = self.target else {
            self.loaded_target = None;
            self.notes.clear();
            return;
        };

        self.loaded_target = None;

        if let Err(err) = self
            .async_notes
            .request(Duration::from_secs(0), true, self.notes_ref.as_deref())
        {
            log::error!("failed to request notes: {}", err);
            self.queue.push(crate::queue::InternalEvent::ShowErrorMsg(format!(
                "failed to request notes:\n{}",
                err
            )));
        }
    }

    /// Drain the latest notes snapshot from the async job queue.
    fn refresh(&mut self) {
        match self.async_notes.refresh() {
            Ok(true) => {
                if let Ok(Some(mut notes)) = self.async_notes.last() {
                    if let Some(target) = self.target {
                        notes.retain(|note| note.annotated_id == target);
                        self.loaded_target = Some(target);
                    }

                    notes.sort_by(|a, b| {
                        a.committer_time
                            .cmp(&b.committer_time)
                            .then_with(|| a.note_id.to_string().cmp(&b.note_id.to_string()))
                    });
                    self.notes = notes;
                }
            }
            Ok(false) => {}
            Err(err) => {
                log::error!("failed to refresh notes: {}", err);
                self.queue.push(crate::queue::InternalEvent::ShowErrorMsg(format!(
                    "failed to refresh notes:\n{}",
                    err
                )));
                self.notes.clear();
            }
        }
    }

    /// Queue the system note editor through the app's existing editor flow.
    pub fn open_editor(&mut self) {
        if let Some(target) = self.target {
            let notes_ref = self
                .notes
                .first()
                .and_then(|note| note.notes_ref.clone())
                .or_else(|| self.notes_ref.clone())
                .or_else(|| default_notes_ref(&self.repo.borrow()).ok());

            self.queue
                .push(crate::queue::InternalEvent::OpenGitNote(target.into(), notes_ref));
        }
    }

    /// Apply async-git notifications from the app's queue lifecycle.
    pub fn update_git(&mut self, ev: AsyncGitNotification) {
        if matches!(ev, AsyncGitNotification::Notes) {
            self.refresh();
        }
    }

    pub fn event(&mut self, ev: &Event) -> Result<bool> {
        Ok(false)
    }

    pub fn is_editing(&self) -> bool {
        self.target.is_some()
    }

    pub fn commands(&self, out: &mut Vec<CommandInfo>, force_all: bool) -> CommandBlocking {
        out.push(CommandInfo::new(
            strings::commands::new_note(),
            self.target.is_some(),
            true,
        ));
        CommandBlocking::PassingOn
    }

    fn get_notes_text(&self, height: usize, width: usize) -> Vec<Line<'_>> {
        let mut txt: Vec<Line> = Vec::with_capacity(height);

        if self.target.is_some() && self.loaded_target != self.target {
            txt.push(Line::from("Loading notes..."));
            return txt;
        }

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

        let note_open = strings::commands::new_note();
        let hint = Paragraph::new(Line::from(vec![
            Span::raw(" "),
            Span::styled(note_open.name, self.theme.commit_hash(false)),
            Span::raw(" "),
            Span::raw(note_open.desc),
        ]))
        .block(Block::default().borders(Borders::ALL).title("Note"));
        f.render_widget(hint, input_area);
    }
}
