use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    str::FromStr,
};

use anyhow::Result;
use crossterm::event::{Event, KeyCode};
use gnostr_asyncgit::{
    sync::{utils::repo_work_dir, RepoPathRef},
    types::{
        nip34::{RepoRef, RepoState},
        EventKind, EventV3, PrivateKey, PublicKey, UncheckedUrl,
    },
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Tabs, Wrap},
    Frame,
};
use unicode_truncate::UnicodeTruncateStr;

use crate::{
    app::Environment,
    components::{
        visibility_blocking, CommandBlocking, CommandInfo, Component, DrawableComponent,
        EventState, ScrollType,
    },
    keys::{key_match, SharedKeyConfig},
    queue::Queue,
    strings,
    ui::{self, style::SharedTheme},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Nip34Mode {
    Commits,
    Branches,
    Events,
}

#[derive(Debug, Clone)]
struct CommitRow {
    hash: String,
    full_hash: String,
    author: String,
    summary: String,
    date: String,
}

#[derive(Debug, Clone)]
struct BranchRow {
    name: String,
    full_hash: String,
    commit_message: String,
    author: String,
    is_current: bool,
    is_remote: bool,
}

#[derive(Debug, Clone)]
struct EventRow {
    label: String,
    event: EventV3,
}

pub struct Nip34Popup {
    repo: RepoPathRef,
    queue: Queue,
    theme: SharedTheme,
    key_config: SharedKeyConfig,
    visible: bool,
    mode: Nip34Mode,
    commits: Vec<CommitRow>,
    branches: Vec<BranchRow>,
    events: Vec<EventRow>,
    commit_state: RefCell<ListState>,
    branch_state: RefCell<ListState>,
    event_state: RefCell<ListState>,
    current_height: Cell<usize>,
    private_key: PrivateKey,
    public_key: PublicKey,
    repo_name: String,
}

impl Nip34Popup {
    pub fn new(env: &Environment) -> Self {
        let private_key = PrivateKey::generate();
        let public_key = private_key.public_key();

        Self {
            repo: env.repo.clone(),
            queue: env.queue.clone(),
            theme: env.theme.clone(),
            key_config: env.key_config.clone(),
            visible: false,
            mode: Nip34Mode::Commits,
            commits: Vec::new(),
            branches: Vec::new(),
            events: Vec::new(),
            commit_state: RefCell::new(ListState::default()),
            branch_state: RefCell::new(ListState::default()),
            event_state: RefCell::new(ListState::default()),
            current_height: Cell::new(0),
            private_key,
            public_key,
            repo_name: String::new(),
        }
    }

    fn short_hash(hash: &str) -> String {
        hash.chars().take(8).collect()
    }

    fn repo_name(work_dir: &str) -> String {
        std::path::Path::new(work_dir)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("repository")
            .to_string()
    }

    fn selected_index(&self) -> usize {
        match self.mode {
            Nip34Mode::Commits => self.commit_state.borrow().selected().unwrap_or(0),
            Nip34Mode::Branches => self.branch_state.borrow().selected().unwrap_or(0),
            Nip34Mode::Events => self.event_state.borrow().selected().unwrap_or(0),
        }
    }

    fn select_first(&self) {
        self.commit_state
            .borrow_mut()
            .select((!self.commits.is_empty()).then_some(0));
        self.branch_state
            .borrow_mut()
            .select((!self.branches.is_empty()).then_some(0));
        self.event_state
            .borrow_mut()
            .select((!self.events.is_empty()).then_some(0));
    }

    fn switch_mode(&mut self) {
        self.mode = match self.mode {
            Nip34Mode::Commits => Nip34Mode::Branches,
            Nip34Mode::Branches => Nip34Mode::Events,
            Nip34Mode::Events => Nip34Mode::Commits,
        };
    }

    fn move_selection(&self, scroll_type: ScrollType) -> bool {
        let (state, len) = match self.mode {
            Nip34Mode::Commits => (&self.commit_state, self.commits.len()),
            Nip34Mode::Branches => (&self.branch_state, self.branches.len()),
            Nip34Mode::Events => (&self.event_state, self.events.len()),
        };

        if len == 0 {
            return false;
        }

        let mut state = state.borrow_mut();
        let old = state.selected().unwrap_or(0);
        let max = len.saturating_sub(1);
        let next = match scroll_type {
            ScrollType::Up => old.saturating_sub(1),
            ScrollType::Down => old.saturating_add(1).min(max),
            ScrollType::Home => 0,
            ScrollType::End => max,
            ScrollType::PageUp => old.saturating_sub(self.current_height.get().saturating_sub(1)),
            ScrollType::PageDown => old
                .saturating_add(self.current_height.get().saturating_sub(1))
                .min(max),
        };
        state.select(Some(next));
        next != old
    }

    fn load_commits(repo: &git2::Repository) -> Result<Vec<CommitRow>> {
        let mut revwalk = repo.revwalk()?;
        revwalk.push_head()?;

        let commits = revwalk
            .filter_map(|id| id.ok())
            .filter_map(|oid| repo.find_commit(oid).ok())
            .take(100)
            .map(|commit| {
                let author = commit.author();
                let date = chrono::DateTime::from_timestamp(commit.committer().when().seconds(), 0)
                    .map(|dt| dt.naive_local())
                    .unwrap_or_default()
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string();
                let full_hash = commit.id().to_string();

                CommitRow {
                    hash: Self::short_hash(&full_hash),
                    full_hash,
                    author: author.name().unwrap_or("Unknown").to_string(),
                    summary: commit.summary().unwrap_or_default().to_string(),
                    date,
                }
            })
            .collect();

        Ok(commits)
    }

    fn load_branches(repo: &git2::Repository) -> Result<Vec<BranchRow>> {
        let head = repo.head().ok().and_then(|head| head.name().map(str::to_string));
        let mut branches = Vec::new();

        for branch_type in [git2::BranchType::Local, git2::BranchType::Remote] {
            for branch_ref in repo.branches(Some(branch_type))? {
                let (branch, _) = branch_ref?;
                if let Some(branch_name) = branch.name()? {
                    let ref_name = match branch_type {
                        git2::BranchType::Local => format!("refs/heads/{branch_name}"),
                        git2::BranchType::Remote => format!("refs/remotes/{branch_name}"),
                    };
                    if let Ok(reference) = repo.find_reference(&ref_name) {
                        if let Ok(commit) = reference.peel_to_commit() {
                            let author = commit.author();
                            branches.push(BranchRow {
                                name: branch_name.to_string(),
                                full_hash: commit.id().to_string(),
                                commit_message: commit.summary().unwrap_or_default().to_string(),
                                author: author.name().unwrap_or("Unknown").to_string(),
                                is_current: head.as_deref() == Some(ref_name.as_str()),
                                is_remote: matches!(branch_type, git2::BranchType::Remote),
                            });
                        }
                    }
                }
            }
        }

        Ok(branches)
    }

    fn build_events(
        repo_name: &str,
        commits: &[CommitRow],
        branches: &[BranchRow],
        private_key: &PrivateKey,
        public_key: PublicKey,
    ) -> Result<Vec<EventRow>> {
        let mut events = Vec::new();

        if let Some(root_commit) = commits.first().map(|commit| commit.full_hash.clone()) {
            if !root_commit.is_empty() {
                let repo_ref = RepoRef {
                    name: repo_name.to_string(),
                    description: format!("NIP-34 snapshot for {repo_name}"),
                    identifier: repo_name.to_string(),
                    root_commit,
                    git_server: Vec::new(),
                    web: Vec::new(),
                    relays: vec![UncheckedUrl::from_str("wss://relay.damus.io")],
                    hashtags: vec![repo_name.to_string()],
                    maintainers: vec![public_key],
                    trusted_maintainer: public_key,
                    events: HashMap::new(),
                };

                if let Ok(event) = repo_ref.to_event(private_key) {
                    events.push(EventRow {
                        label: "Repo announcement".to_string(),
                        event,
                    });
                }
            }
        }

        let mut state = HashMap::new();
        for branch in branches {
            let key = if branch.is_remote {
                format!("refs/remotes/{}", branch.name)
            } else {
                format!("refs/heads/{}", branch.name)
            };
            state.insert(key, branch.full_hash.clone());
        }

        if let Ok(repo_state) = RepoState::build(repo_name.to_string(), state, private_key) {
            events.push(EventRow {
                label: "Repo state".to_string(),
                event: repo_state.event,
            });
        }

        Ok(events)
    }

    fn reload(&mut self) -> Result<()> {
        let work_dir = repo_work_dir(&self.repo.borrow())?;
        let repo = git2::Repository::open(&work_dir)?;
        let repo_name = Self::repo_name(&work_dir);
        let commits = Self::load_commits(&repo)?;
        let branches = Self::load_branches(&repo)?;
        let events = Self::build_events(
            &repo_name,
            &commits,
            &branches,
            &self.private_key,
            self.public_key,
        )?;

        self.repo_name = repo_name;
        self.commits = commits;
        self.branches = branches;
        self.events = events;
        self.select_first();

        Ok(())
    }

    pub fn any_work_pending(&self) -> bool {
        false
    }

    fn open_detail_lines(&self, width: usize) -> Vec<Line<'_>> {
        let mut out = Vec::new();
        match self.mode {
            Nip34Mode::Commits => {
                if let Some(commit) = self.commits.get(self.selected_index()) {
                    out.push(Line::from(vec![Span::raw(format!("Commit: {}", commit.full_hash))]));
                    out.push(Line::from(vec![Span::raw(format!("Author: {}", commit.author))]));
                    out.push(Line::from(vec![Span::raw(format!("Date: {}", commit.date))]));
                    out.push(Line::from(vec![Span::raw(String::new())]));
                    out.push(Line::from(vec![Span::raw("Message:")]));
                    out.push(Line::from(vec![Span::raw(
                        commit.summary.unicode_truncate(width).0.to_string(),
                    )]));
                }
            }
            Nip34Mode::Branches => {
                if let Some(branch) = self.branches.get(self.selected_index()) {
                    out.push(Line::from(vec![Span::raw(format!("Branch: {}", branch.name))]));
                    out.push(Line::from(vec![Span::raw(format!("Commit: {}", branch.full_hash))]));
                    out.push(Line::from(vec![Span::raw(format!("Author: {}", branch.author))]));
                    out.push(Line::from(vec![Span::raw(format!(
                        "Current: {}",
                        if branch.is_current { "yes" } else { "no" }
                    ))]));
                    out.push(Line::from(vec![Span::raw(String::new())]));
                    out.push(Line::from(vec![Span::raw("Message:")]));
                    out.push(Line::from(vec![Span::raw(
                        branch.commit_message.unicode_truncate(width).0.to_string(),
                    )]));
                }
            }
            Nip34Mode::Events => {
                if let Some(row) = self.events.get(self.selected_index()) {
                    let event = &row.event;
                    let kind = u32::from(event.kind);
                    out.push(Line::from(vec![Span::raw(format!("Event: {}", row.label))]));
                    out.push(Line::from(vec![Span::raw(format!("Kind: {} ({})", kind, kind_label(event.kind)))]));
                    out.push(Line::from(vec![Span::raw(format!("Id: {}", event.id))]));
                    out.push(Line::from(vec![Span::raw(format!("Author: {}", event.pubkey))]));
                    out.push(Line::from(vec![Span::raw(format!("Created: {}", event.created_at))]));
                    out.push(Line::from(vec![Span::raw(String::new())]));
                    out.push(Line::from(vec![Span::raw("Tags:")]));
                    let tag_text = event
                        .tags
                        .iter()
                        .map(|tag| format!("{tag:?}"))
                        .collect::<Vec<_>>()
                        .join(" | ");
                    out.push(Line::from(vec![Span::raw(tag_text.unicode_truncate(width).0.to_string())]));
                    out.push(Line::from(vec![Span::raw(String::new())]));
                    out.push(Line::from(vec![Span::raw("Content:")]));
                    out.push(Line::from(vec![Span::raw(
                        event.content.unicode_truncate(width).0.to_string(),
                    )]));
                }
            }
        }

        out
    }

    fn items(&self) -> Vec<ListItem<'_>> {
        match self.mode {
            Nip34Mode::Commits => self
                .commits
                .iter()
                .map(|commit| {
                    ListItem::new(Line::from(vec![Span::raw(format!(
                        "[{}] {} - {}",
                        commit.hash,
                        commit.author,
                        commit.summary
                    ))]))
                })
                .collect(),
            Nip34Mode::Branches => self
                .branches
                .iter()
                .map(|branch| {
                    let marker = if branch.is_current { "*" } else { " " };
                    ListItem::new(Line::from(vec![Span::raw(format!(
                        "{marker} [{}] {} - {}",
                        Self::short_hash(&branch.full_hash),
                        branch.name,
                        branch.commit_message
                    ))]))
                })
                .collect(),
            Nip34Mode::Events => self
                .events
                .iter()
                .map(|event| ListItem::new(Line::from(vec![Span::raw(event.label.clone())])))
                .collect(),
        }
    }

    fn state(&self) -> std::cell::RefMut<'_, ListState> {
        match self.mode {
            Nip34Mode::Commits => self.commit_state.borrow_mut(),
            Nip34Mode::Branches => self.branch_state.borrow_mut(),
            Nip34Mode::Events => self.event_state.borrow_mut(),
        }
    }
}

fn kind_label(kind: EventKind) -> &'static str {
    match kind {
        EventKind::RepositoryAnnouncement => "Repo Announcement",
        EventKind::GitRepoAnnouncement => "Repo State",
        EventKind::Patches => "Patch",
        EventKind::GitIssue => "Issue",
        EventKind::GitReply => "Reply",
        EventKind::GitStatusOpen => "Status Open",
        EventKind::GitStatusApplied => "Status Applied",
        EventKind::GitStatusClosed => "Status Closed",
        EventKind::GitStatusDraft => "Status Draft",
        _ => "Unknown",
    }
}

impl DrawableComponent for Nip34Popup {
    fn draw(&self, f: &mut Frame, rect: Rect) -> Result<()> {
        if self.visible {
            let area = ui::rect_inside(
                crate::ui::Size::new(70, 22),
                crate::ui::Size::from(f.area()),
                ui::centered_rect(90, 80, f.area()),
            )
            .intersection(rect);

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(8),
                    Constraint::Length(9),
                ])
                .split(area);

            let tabs = Tabs::new(vec![
                Line::from("Commits"),
                Line::from("Branches"),
                Line::from("NIP-34"),
            ])
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(Span::styled(
                        format!("NIP-34: {}", self.repo_name),
                        self.theme.title(true),
                    ))
                    .border_style(self.theme.block(true)),
            )
            .style(self.theme.tab(false))
            .highlight_style(self.theme.tab(true))
            .select(match self.mode {
                Nip34Mode::Commits => 0,
                Nip34Mode::Branches => 1,
                Nip34Mode::Events => 2,
            });
            f.render_widget(Clear, area);
            f.render_widget(tabs, chunks[0]);

            let items = self.items();
            let mut state = self.state();
            let list = List::new(items)
                .highlight_style(self.theme.text(true, true))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(Span::styled("Items", self.theme.title(true)))
                        .border_style(self.theme.block(true)),
                );
            f.render_stateful_widget(list, chunks[1], &mut *state);

            let detail_area = chunks[2].inner(Margin {
                vertical: 0,
                horizontal: 1,
            });
            let details = Paragraph::new(self.open_detail_lines(detail_area.width as usize))
                .wrap(Wrap { trim: true })
                .alignment(Alignment::Left)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(Span::styled("Details", self.theme.title(true)))
                        .border_style(self.theme.block(true)),
            );
            f.render_widget(details, chunks[2]);

            self.current_height.set(chunks[1].height.into());
        }

        Ok(())
    }
}

impl Component for Nip34Popup {
    fn commands(&self, out: &mut Vec<CommandInfo>, force_all: bool) -> CommandBlocking {
        if self.visible || force_all {
            if !force_all {
                out.clear();
            }

            out.push(CommandInfo::new(
                strings::commands::scroll(&self.key_config),
                true,
                true,
            ));
            out.push(CommandInfo::new(
                strings::commands::nip34_switch_mode(),
                true,
                true,
            ));
            out.push(CommandInfo::new(
                strings::commands::close_popup(&self.key_config),
                true,
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
                } else if key_match(key, self.key_config.keys.home)
                    || key_match(key, self.key_config.keys.shift_up)
                {
                    self.move_selection(ScrollType::Home);
                } else if key_match(key, self.key_config.keys.end)
                    || key_match(key, self.key_config.keys.shift_down)
                {
                    self.move_selection(ScrollType::End);
                } else if key_match(key, self.key_config.keys.page_up) {
                    self.move_selection(ScrollType::PageUp);
                } else if key_match(key, self.key_config.keys.page_down) {
                    self.move_selection(ScrollType::PageDown);
                } else if key.code == KeyCode::Tab {
                    self.switch_mode();
                }
            }
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
        self.commits.clear();
        self.branches.clear();
        self.events.clear();
        self.commit_state.borrow_mut().select(None);
        self.branch_state.borrow_mut().select(None);
        self.event_state.borrow_mut().select(None);

        if let Err(err) = self.reload() {
            self.queue.push(crate::queue::InternalEvent::ShowErrorMsg(format!(
                "failed to load NIP-34 snapshot:\n{err}"
            )));
        }
        self.visible = true;
        Ok(())
    }
}
