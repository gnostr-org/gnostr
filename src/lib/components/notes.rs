#![allow(missing_docs)]
use crate::{
    app::Environment,
    components::{CommandBlocking, CommandInfo},
    queue::Queue,
    spinner::spinner_char,
    strings,
    ui::style::SharedTheme,
};
use anyhow::Result;
use crossterm::event::Event;
use git2::Oid;
use gnostr_asyncgit::{
    sync::{default_notes_ref, NoteInfo, RepoPathRef},
    AsyncGitNotification, AsyncNotes,
};
use ratatui::{
    layout::{Alignment, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::time::Duration;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NotesState {
    Idle,
    Loading,
    Ready,
    Empty,
    Error,
}

/// Reusable notes service for notes data and refresh lifecycle.
pub struct NotesService {
    repo: RepoPathRef,
    queue: Queue,
    theme: SharedTheme,
    // Mirrors the repository snapshot that was last delivered through the
    // async notes notification path.
    async_notes: AsyncNotes,
    target: Option<Oid>,
    notes_ref: Option<String>,
    state: NotesState,
    // Keep the full notes snapshot here so any commit row can query it.
    notes_snapshot: Vec<NoteInfo>,
    // Keep only the selected commit's notes in this view.
    notes: Vec<NoteInfo>,
    spinner_idx: usize,
    snapshot_loaded: bool,
}

impl NotesService {
    pub fn new(env: &Environment) -> Self {
        Self {
            repo: env.repo.clone(),
            queue: env.queue.clone(),
            theme: env.theme.clone(),
            async_notes: AsyncNotes::new(env.repo.borrow().clone(), &env.sender_git),
            target: None,
            notes_ref: None,
            state: NotesState::Idle,
            notes_snapshot: Vec::new(),
            notes: Vec::new(),
            spinner_idx: 0,
            snapshot_loaded: false,
        }
    }

    pub fn clear(&mut self) {
        self.target = None;
        self.state = NotesState::Idle;
        self.notes_snapshot.clear();
        self.notes.clear();
        self.spinner_idx = 0;
        self.snapshot_loaded = false;
    }

    pub fn set_target(&mut self, target: Option<Oid>) {
        if self.target != target {
            self.target = target;
            if self.snapshot_loaded {
                self.sync_target_notes();
            } else if self.target.is_some() {
                self.request_refresh();
            }
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
            self.state = NotesState::Idle;
            self.snapshot_loaded = false;
            self.notes_snapshot.clear();
            self.notes.clear();
            return;
        };

        self.state = NotesState::Loading;
        self.snapshot_loaded = false;
        self.notes_snapshot.clear();
        self.notes.clear();
        self.spinner_idx = 0;

        if let Err(err) =
            self.async_notes
                .request(Duration::from_secs(0), true, self.notes_ref.as_deref())
        {
            log::error!("failed to request notes: {}", err);
            self.queue
                .push(crate::queue::InternalEvent::ShowErrorMsg(format!(
                    "failed to request notes:\n{}",
                    err
                )));
        }
    }

    /// Force a notes reload for the current target.
    pub fn refresh(&mut self) {
        self.request_refresh();
    }

    /// Drain completed async notes work on the normal update cycle.
    pub fn update(&mut self) {
        self.refresh_from_async();
    }

    /// Advance the notes spinner frame on the shared UI tick.
    pub fn update_spinner(&mut self) {
        if matches!(self.state, NotesState::Loading) || self.async_notes.is_pending() {
            self.spinner_idx = self.spinner_idx.wrapping_add(1);
        }
    }

    /// Drain the latest notes snapshot from the async job queue.
    fn refresh_from_async(&mut self) {
        match self.async_notes.refresh() {
            Ok(true) => {
                if let Ok(Some(mut notes)) = self.async_notes.last() {
                    notes.sort_by(|a, b| {
                        a.committer_time
                            .cmp(&b.committer_time)
                            .then_with(|| a.note_id.to_string().cmp(&b.note_id.to_string()))
                    });
                    self.notes_snapshot = notes;
                    self.snapshot_loaded = true;
                    self.sync_target_notes();
                }
            }
            Ok(false) => {}
            Err(err) => {
                log::error!("failed to refresh notes: {}", err);
                self.state = NotesState::Error;
                self.snapshot_loaded = false;
                self.notes_snapshot.clear();
                self.queue
                    .push(crate::queue::InternalEvent::ShowErrorMsg(format!(
                        "failed to refresh notes:\n{}",
                        err
                    )));
                self.notes.clear();
            }
        }
    }

    fn sync_target_notes(&mut self) {
        self.notes.clear();

        if let Some(target) = self.target {
            self.notes.extend(
                self.notes_snapshot
                    .iter()
                    .filter(|note| note.annotated_id == target)
                    .cloned(),
            );
            self.state = if self.notes.is_empty() {
                NotesState::Empty
            } else {
                NotesState::Ready
            };
        } else {
            self.state = NotesState::Idle;
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

            self.queue.push(crate::queue::InternalEvent::OpenGitNote(
                target.into(),
                notes_ref,
            ));
        }
    }

    /// Apply async-git notifications from the app's queue lifecycle.
    pub fn update_git(&mut self, ev: AsyncGitNotification) {
        if matches!(ev, AsyncGitNotification::Notes) {
            self.refresh_from_async();
        }
    }

    pub fn event(&mut self, _ev: &Event) -> Result<bool> {
        Ok(false)
    }

    pub fn is_editing(&self) -> bool {
        self.target.is_some()
    }

    pub fn has_notes(&self) -> bool {
        matches!(self.state, NotesState::Ready) && !self.notes.is_empty()
    }

    pub fn has_notes_for(&self, target: Oid) -> bool {
        self.notes_snapshot
            .iter()
            .any(|note| note.annotated_id == target)
    }

    pub fn state(&self) -> NotesState {
        self.state
    }

    pub fn any_work_pending(&self) -> bool {
        self.async_notes.is_pending() || matches!(self.state, NotesState::Loading)
    }

    pub fn commands(&self, out: &mut Vec<CommandInfo>, _force_all: bool) -> CommandBlocking {
        out.push(CommandInfo::new(
            strings::commands::new_note(),
            self.target.is_some(),
            true,
        ));
        CommandBlocking::PassingOn
    }

    fn get_notes_text(&self, height: usize, width: usize) -> Vec<Line<'_>> {
        let mut txt: Vec<Line> = Vec::with_capacity(height);

        match self.state {
            NotesState::Loading => {
                txt.push(Line::from("Loading notes..."));
                return txt;
            }
            NotesState::Error => {
                txt.push(Line::from("Failed to load notes"));
                return txt;
            }
            NotesState::Idle | NotesState::Empty => {
                txt.push(Line::from("No notes"));
                return txt;
            }
            NotesState::Ready => {}
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
        let title = if matches!(self.state, NotesState::Loading) || self.async_notes.is_pending() {
            format!("{} Notes", spinner_char(self.spinner_idx))
        } else {
            "Notes".to_string()
        };

        f.render_widget(
            Paragraph::new(self.get_notes_text(notes_height, notes_width))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(title)
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

pub type NotesComponent = NotesService;
