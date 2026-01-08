use std::{
    collections::HashSet,
    io,
    time::{Duration, Instant},
};

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    prelude::Stylize,
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs},
    Frame, Terminal,
};

// Import gnostr NIP-34 types
use gnostr::types::nip34::{UnsignedEvent, Nip34Kind};

/// Represents a relevant subset of a Git commit's data.
#[derive(Debug, Clone)]
struct Commit {
    hash: String,
    full_hash: String,
    author: String,
    summary: String,
    committer_date: String,
}

/// Represents a Git branch's data.
#[derive(Debug, Clone)]
struct Branch {
    name: String,
    commit_hash: String,
    commit_message: String,
    author: String,
    is_current: bool,
    is_remote: bool,
}

/// NIP-34 event data for display.
#[derive(Debug, Clone)]
struct Nip34Event {
    id: String,
    pubkey: String,
    kind: u16,
    created_at: u64,
    content: String,
    tags: Vec<Vec<String>>,
}

/// Navigation modes for the application.
#[derive(Debug, Clone, PartialEq)]
enum NavigatorMode {
    Commits,
    Branches,
    Nip34Events,
}

/// The main application state.
struct App {
    commits: Vec<Commit>,
    branches: Vec<Branch>,
    nip34_events: Vec<Nip34Event>,
    commit_state: ListState,
    branch_state: ListState,
    nip34_state: ListState,
    current_mode: NavigatorMode,
    repo: git2::Repository,
    selected_commits: HashSet<usize>,
    selected_nip34_events: HashSet<usize>,
    full_commit_details: Option<String>,
    show_full_commit: bool,
}

impl App {
    /// Constructs a new App with git data and NIP-34 support.
    fn new() -> Result<Self> {
        let repo = git2::Repository::open_from_env()?;
        
        // Load commits (same as original)
        let mut revwalk = repo.revwalk()?;
        revwalk.push_head()?;

        let commits: Vec<Commit> = revwalk
            .filter_map(|id| id.ok())
            .filter_map(|oid| repo.find_commit(oid).ok())
            .take(100)
            .map(|commit| {
                let author = commit.author();
                let time = commit.committer().when();
                let date = time.seconds();
                let datetime = chrono::DateTime::from_timestamp(date, 0)
                    .map(|dt| dt.naive_local())
                    .unwrap_or_default();
                let committer_date = datetime.format("%Y-%m-%d %H:%M:%S").to_string();

                let full_hash = commit.id().to_string();
                let hash = full_hash.chars().take(8).collect::<String>();
                let summary = commit.summary().unwrap_or_default().to_string();

                Commit {
                    hash,
                    full_hash,
                    author: author.name().unwrap_or("Unknown").to_string(),
                    summary,
                    committer_date,
                }
            })
            .collect();

        // Load branches (same as original)
        let mut branches = Vec::new();

        for branch_ref in repo.branches(Some(git2::BranchType::Local))? {
            let (branch, _) = branch_ref?;
            if let Some(branch_name) = branch.name()? {
                let branch_ref_name = format!("refs/heads/{}", branch_name);
                if let Ok(reference) = repo.find_reference(&branch_ref_name) {
                    if let Some(commit) = reference.peel_to_commit().ok() {
                        let is_current = if let Ok(head) = repo.head() {
                            head.name() == Some(format!("refs/heads/{}", branch_name).as_str())
                        } else {
                            false
                        };

                        let author = commit.author();
                        branches.push(Branch {
                            name: branch_name.to_string(),
                            commit_hash: commit.id().to_string().chars().take(8).collect(),
                            commit_message: commit.summary().unwrap_or_default().to_string(),
                            author: author.name().unwrap_or("Unknown").to_string(),
                            is_current,
                            is_remote: false,
                        });
                    }
                }
            }
        }

        // For now, create sample NIP-34 events
        // In a real implementation, these would be loaded from Nostr relays
        let nip34_events = vec![
            Nip34Event {
                id: "abc123def456".to_string(),
                pubkey: "npub1abc...".to_string(),
                kind: Nip34Kind::Patch as u16,
                created_at: chrono::Utc::now().timestamp() as u64,
                content: "Fix critical authentication bug in NIP-34 implementation".to_string(),
                tags: vec![
                    vec!["d".to_string(), "fix-auth-bug".to_string()],
                    vec!["repository".to_string(), "gnostr-org/gnostr".to_string()],
                ],
            },
            Nip34Event {
                id: "def456ghi789".to_string(),
                pubkey: "npub1def...".to_string(),
                kind: Nip34Kind::PullRequest as u16,
                created_at: (chrono::Utc::now().timestamp() - 3600) as u64, // 1 hour ago
                content: "Add NIP-34 event creation and signing support".to_string(),
                tags: vec![
                    vec!["r".to_string(), "main".to_string()],
                    vec!["pr".to_string(), "42".to_string()],
                    vec!["repository".to_string(), "gnostr-org/gnostr".to_string()],
                ],
            },
            Nip34Event {
                id: "ghi789jkl012".to_string(),
                pubkey: "npub1ghi...".to_string(),
                kind: Nip34Kind::Issue as u16,
                created_at: (chrono::Utc::now().timestamp() - 7200) as u64, // 2 hours ago
                content: "Implement NIP-34 event serialization for git patches".to_string(),
                tags: vec![
                    vec!["k".to_string(), "issue-123".to_string()],
                    vec!["repository".to_string(), "gnostr-org/gnostr".to_string()],
                    vec!["title".to_string(), "Feature: NIP-34 support".to_string()],
                ],
            },
        ];

        let mut commit_state = ListState::default();
        if !commits.is_empty() {
            commit_state.select(Some(0));
        }

        let mut branch_state = ListState::default();
        if !branches.is_empty() {
            branch_state.select(Some(0));
        }

        let mut nip34_state = ListState::default();
        if !nip34_events.is_empty() {
            nip34_state.select(Some(0));
        }

        Ok(Self {
            commits,
            branches,
            nip34_events,
            commit_state,
            branch_state,
            nip34_state,
            current_mode: NavigatorMode::Commits,
            repo,
            selected_commits: HashSet::new(),
            selected_nip34_events: HashSet::new(),
        })
    }

    /// Switches between navigation modes.
    fn switch_mode(&mut self) {
        self.current_mode = match self.current_mode {
            NavigatorMode::Commits => NavigatorMode::Branches,
            NavigatorMode::Branches => NavigatorMode::Nip34Events,
            NavigatorMode::Nip34Events => NavigatorMode::Commits,
        };
    }

    /// Moves the selection up in the current mode.
    fn previous(&mut self) {
        let state = match self.current_mode {
            NavigatorMode::Commits => &mut self.commit_state,
            NavigatorMode::Branches => &mut self.branch_state,
            NavigatorMode::Nip34Events => &mut self.nip34_state,
        };

        let items = match self.current_mode {
            NavigatorMode::Commits => self.commits.len(),
            NavigatorMode::Branches => self.branches.len(),
            NavigatorMode::Nip34Events => self.nip34_events.len(),
        };

        if items == 0 {
            return;
        }

        let i = match state.selected() {
            Some(i) => {
                if i == 0 {
                    items - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        state.select(Some(i));
    }

    /// Moves the selection down in the current mode.
    fn next(&mut self) {
        let state = match self.current_mode {
            NavigatorMode::Commits => &mut self.commit_state,
            NavigatorMode::Branches => &mut self.branch_state,
            NavigatorMode::Nip34Events => &mut self.nip34_state,
        };

        let items = match self.current_mode {
            NavigatorMode::Commits => self.commits.len(),
            NavigatorMode::Branches => self.branches.len(),
            NavigatorMode::Nip34Events => self.nip34_events.len(),
        };

        if items == 0 {
            return;
        }

        let i = match state.selected() {
            Some(i) => {
                if i >= items - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        state.select(Some(i));
    }

    /// Toggles selection of current commit (max 2 commits).
    /// Automatically shows diff when exactly 2 commits are selected.
    fn toggle_commit_selection(&mut self) {
        if let Some(selected_index) = self.commit_state.selected() {
            if self.selected_commits.contains(&selected_index) {
                self.selected_commits.remove(&selected_index);
            } else if self.selected_commits.len() < 2 {
                self.selected_commits.insert(selected_index);
                
                // Auto-show diff when exactly 2 commits are selected
                if self.selected_commits.len() == 2 {
                    let _ = self.load_full_commit();
                }
            }
            // If we already have 2 selected and trying to add a third,
            // replace the oldest selection and auto-show new diff
            else if self.selected_commits.len() >= 2 {
                let mut indices: Vec<_> = self.selected_commits.iter().cloned().collect();
                indices.sort();
                self.selected_commits.remove(&indices[0]); // Remove oldest
                self.selected_commits.insert(selected_index);
                
                // Auto-show diff for new selection
                let _ = self.load_full_commit();
            }
        }
    }

    /// Toggles selection of NIP-34 events.
    fn toggle_nip34_selection(&mut self) {
        if let Some(selected_index) = self.nip34_state.selected() {
            if self.selected_nip34_events.contains(&selected_index) {
                self.selected_nip34_events.remove(&selected_index);
            } else {
                self.selected_nip34_events.insert(selected_index);
            }
        }
    }

    /// Clears all selected commits/events.
    fn clear_selection(&mut self) {
        self.selected_commits.clear();
        self.selected_nip34_events.clear();
    }

    /// Returns number of selected items in current mode.
    fn selected_count(&self) -> usize {
        match self.current_mode {
            NavigatorMode::Commits => self.selected_commits.len(),
            NavigatorMode::Branches => 0,
            NavigatorMode::Nip34Events => self.selected_nip34_events.len(),
        }
    }

    /// Creates NIP-34 event from selected commits.
    fn create_nip34_patch_event(&self) -> Result<Nip34Event> {
        if self.selected_commits.len() != 2 {
            return Err(anyhow::anyhow!("Need exactly 2 commits selected to create patch"));
        }

        let mut indices: Vec<_> = self.selected_commits.iter().cloned().collect();
        indices.sort();
        
        if let (Some(from_index), Some(to_index)) = (indices.get(0), indices.get(1)) {
            if let (Some(from_commit), Some(to_commit)) = (self.commits.get(*from_index), self.commits.get(*to_index)) {
                // Create a patch between two commits
                let content = format!(
                    "Patch from {} to {}\n\nFrom: {} ({})\nTo: {} ({})\n\nThis would create a NIP-34 patch event.",
                    from_commit.hash, to_commit.hash,
                    from_commit.author, from_commit.summary,
                    to_commit.author, to_commit.summary
                );

                Ok(Nip34Event {
                    id: "new-patch-event".to_string(),
                    pubkey: "your-pubkey-here".to_string(), // User would need to provide this
                    kind: Nip34Kind::Patch as u16,
                    created_at: chrono::Utc::now().timestamp() as u64,
                    content,
                    tags: vec![
                        vec!["d".to_string(), format!("{}..{}", from_commit.full_hash, to_commit.full_hash)],
                        vec!["repository".to_string(), "gnostr-org/gnostr".to_string()],
                    ],
                })
            } else {
                Err(anyhow::anyhow!("Invalid commit selection"))
            }
        } else {
            Err(anyhow::anyhow!("No commits selected"))
        }
    }

    /// Loads git diff for selected commits (max 2 for range diff).
    fn load_full_commit(&mut self) -> Result<()> {
        let mut diff_content = String::new();

        if !self.selected_commits.is_empty() || self.show_full_commit {
            let mut selected_indices: Vec<_> = self.selected_commits.iter().cloned().collect();
            selected_indices.sort();

            if self.selected_commits.len() == 2 {
                // Show diff range between two commits
                if let (Some(from_index), Some(to_index)) = (selected_indices.get(0), selected_indices.get(1)) {
                    if let (Some(from_commit), Some(to_commit)) = (self.commits.get(*from_index), self.commits.get(*to_index)) {
                        diff_content.push_str("╭─────────────────────────────────────────────────╮\n");
                        diff_content.push_str("│                   Diff Range                        │\n");
                        diff_content.push_str("╰─────────────────────────────────────────────────╯\n\n");

                        diff_content.push_str(&format!("From: [{}] {} - {}\n", from_commit.hash, from_commit.author, from_commit.summary));
                        diff_content.push_str(&format!("To:   [{}] {} - {}\n\n", to_commit.hash, to_commit.author, to_commit.summary));

                        diff_content.push_str("╭─────────────────────────────────────────────────╮\n");
                        diff_content.push_str("│                      Git Diff                       │\n");
                        diff_content.push_str("╰─────────────────────────────────────────────────╯\n\n");

                        diff_content.push_str("📝 Changes between commits:\n\n");

                        // Get diff between two commits
                        let from_oid = git2::Oid::from_str(&from_commit.full_hash)?;
                        let to_oid = git2::Oid::from_str(&to_commit.full_hash)?;
                        
                        if let (Ok(from_commit_obj), Ok(to_commit_obj)) = (self.repo.find_commit(from_oid), self.repo.find_commit(to_oid)) {
                            let from_tree = from_commit_obj.tree()?;
                            let to_tree = to_commit_obj.tree()?;
                            let diff = self.repo.diff_tree_to_tree(Some(&from_tree), Some(&to_tree), None)?;
                            self.format_diff(&diff, &mut diff_content)?;
                        }
                    }
                }
            } else {
                // Show selection summary (1 or 3+ commits selected)
                diff_content.push_str("╭─────────────────────────────────────────────────╮\n");
                diff_content.push_str(&format!(
                    "│              Selected {} Commit(s)                │\n",
                    self.selected_count()
                ));
                diff_content.push_str("╰─────────────────────────────────────────────────╯\n\n");

                for (count, &index) in selected_indices.iter().enumerate() {
                    if let Some(commit) = self.commits.get(*index) {
                        diff_content.push_str(&format!(
                            "{}. [{}] {} - {}\n",
                            count + 1,
                            commit.hash,
                            commit.author,
                            commit.summary
                        ));
                    }
                }

                diff_content.push_str("\n");
                if self.selected_commits.len() == 1 {
                    diff_content.push_str("💡 Tip: Select another commit to view diff range\n");
                } else {
                    diff_content.push_str("💡 Tip: Only 2 commits allowed for diff range. Press 'c' to clear.\n");
                }
            }
        } else if let Some(selected_index) = self.commit_state.selected() {
            // No commits selected, load current focused commit
            if let Some(commit) = self.commits.get(selected_index) {
                let commit_oid = git2::Oid::from_str(&commit.full_hash)?;
                if let Ok(git_commit) = self.repo.find_commit(commit_oid) {
                    diff_content.push_str("╭─────────────────────────────────────────────────╮\n");
                    diff_content.push_str("│                    Commit Details                    │\n");
                    diff_content.push_str("╰─────────────────────────────────────────────────╯\n");
                    diff_content.push_str(&format!("Commit: {}\n", git_commit.id()));
                    diff_content.push_str(&format!(
                        "Author: {} <{}>\n",
                        git_commit.author().name().unwrap_or("Unknown"),
                        git_commit.author().email().unwrap_or("unknown@example.com")
                    ));

                    // Add committer if different from author
                    if git_commit.author() != git_commit.committer() {
                        diff_content.push_str(&format!(
                            "Committer: {} <{}>\n",
                            git_commit.committer().name().unwrap_or("Unknown"),
                            git_commit.committer().email().unwrap_or("unknown@example.com")
                        ));
                        diff_content.push_str(&format!(
                            "Commit Date: {}\n",
                            chrono::DateTime::from_timestamp(git_commit.committer().when().seconds(), 0)
                                .map(|dt| dt.naive_local())
                                .unwrap_or_default()
                                .format("%Y-%m-%d %H:%M:%S")
                        ));
                    } else {
                        diff_content.push_str(&format!(
                            "Date: {}\n",
                            chrono::DateTime::from_timestamp(git_commit.author().when().seconds(), 0)
                                .map(|dt| dt.naive_local())
                                .unwrap_or_default()
                                .format("%Y-%m-%d %H:%M:%S")
                        ));
                    }

                    diff_content.push_str("\n");
                    diff_content.push_str("Message:\n");
                    diff_content.push_str(&format!("    {}\n", git_commit.summary().unwrap_or("")));

                    // Add full commit message body if it exists
                    if let Some(message) = git_commit.message() {
                        let lines: Vec<&str> = message.lines().collect();
                        if lines.len() > 1 {
                            for line in lines.iter().skip(1) {
                                if !line.trim().is_empty() {
                                    diff_content.push_str(&format!("    {}\n", line));
                                }
                            }
                        }
                    }

                    diff_content.push_str("\n");
                    diff_content.push_str("╭─────────────────────────────────────────────────╮\n");
                    diff_content.push_str("│                      Git Diff                       │\n");
                    diff_content.push_str("╰─────────────────────────────────────────────────╯\n\n");

                    // Get diff against parent(s)
                    let parent_count = git_commit.parent_count();

                    if parent_count == 0 {
                        // Initial commit - show all files
                        diff_content.push_str("🌟 Initial Commit - All files:\n\n");

                        let tree = git_commit.tree()?;
                        let diff = self.repo.diff_tree_to_tree(None, Some(&tree), None)?;
                        self.format_diff(&diff, &mut diff_content)?;
                    } else {
                        // Show diff against first parent
                        if parent_count > 1 {
                            diff_content.push_str("🔀 Merge Commit - Diff against first parent:\n\n");
                        } else {
                            diff_content.push_str("📝 Changes - Diff against parent:\n\n");
                        }

                        if let Ok(parent) = git_commit.parent(0) {
                            let parent_tree = parent.tree()?;
                            let current_tree = git_commit.tree()?;
                            let diff = self.repo.diff_tree_to_tree(
                                Some(&parent_tree),
                                Some(&current_tree),
                                None,
                            )?;
                            self.format_diff(&diff, &mut diff_content)?;
                        }
                    }
                }
            }
        }

        self.full_commit_details = Some(diff_content);
        self.show_full_commit = true;
        Ok(())
    }

    /// Formats a git diff object into a readable string.
    fn format_diff(&self, diff: &git2::Diff, output: &mut String) -> Result<()> {
        let mut patch = String::new();
        diff.print(git2::DiffFormat::Patch, |delta, _hunk, line| {
            let origin_char = line.origin();
            let content_str = std::str::from_utf8(line.content()).unwrap_or("");

            match origin_char {
                '+' => {
                    patch.push('+');
                    patch.push_str(content_str);
                }
                '-' => {
                    patch.push('-');
                    patch.push_str(content_str);
                }
                ' ' => {
                    patch.push(' ');
                    patch.push_str(content_str);
                }
                'F' => {
                    // File header
                    if let Some(new_path) = delta.new_file().path() {
                        if let Some(old_path) = delta.old_file().path() {
                            patch.push_str(&format!("diff --git a/{} b/{}\n", 
                                old_path.display(), new_path.display()));
                        }
                    }
                }
                '>' => {
                    // Add/delete/rename operations
                    patch.push_str(content_str);
                }
                '<' => {
                    // Add/delete/rename operations
                    patch.push_str(content_str);
                }
                _ => {
                    // Other line types including file headers
                    patch.push(origin_char);
                    patch.push_str(content_str);
                }
            }
            true
        })?;

        output.push_str(&patch);
        Ok(())
    }

    // Storage for full commit details
    full_commit_details: Option<String>,
    show_full_commit: bool,
}

/// Runs the TUI application loop.
fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> Result<()> {
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Tab => app.switch_mode(),
                        KeyCode::Down | KeyCode::Char('j') => app.next(),
                        KeyCode::Up | KeyCode::Char('k') => app.previous(),
                        KeyCode::Enter | KeyCode::Char(' ') => {
                            match app.current_mode {
                                NavigatorMode::Commits => app.toggle_commit_selection(),
                                NavigatorMode::Branches => {
                                    // Could add branch checkout here
                                }
                                NavigatorMode::Nip34Events => app.toggle_nip34_selection(),
                            }
                        }
                        KeyCode::Right => {
                            if app.current_mode == NavigatorMode::Commits {
                                if let Err(_e) = app.load_full_commit() {
                                    // Could show error message in UI
                                }
                            }
                        }
                        KeyCode::Left => {
                            if app.current_mode == NavigatorMode::Commits && app.show_full_commit {
                                app.show_full_commit = false;
                            }
                        }
                        KeyCode::Char('c') => {
                            app.clear_selection();
                            app.show_full_commit = false;
                        }
                        KeyCode::Char('n') => {
                            // Create NIP-34 patch from selected commits
                            if app.current_mode == NavigatorMode::Commits {
                                if let Err(_e) = app.create_nip34_patch_event() {
                                    // Could show error message
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}

/// Draws the UI to the terminal frame.
fn ui(f: &mut Frame, app: &mut App) {
    let size = f.area();

    // Top navigation tabs
    let titles = vec!["Commits", "Branches", "NIP-34 Events"];
    let selected_index = match app.current_mode {
        NavigatorMode::Commits => 0,
        NavigatorMode::Branches => 1,
        NavigatorMode::Nip34Events => 2,
    };

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("NIP-34 Git Navigator"),
        )
        .style(Style::default().fg(Color::Cyan))
        .highlight_style(Style::default().fg(Color::White).bg(Color::Blue))
        .divider(" | ")
        .select(selected_index);

    f.render_widget(tabs, Rect::new(0, 0, size.width, 3));

    // Main content area below tabs
    let content_area = Rect::new(0, 3, size.width, size.height - 4);

    match app.current_mode {
        NavigatorMode::Commits => render_commits_view(f, app, content_area),
        NavigatorMode::Branches => render_branches_view(f, app, content_area),
        NavigatorMode::Nip34Events => render_nip34_view(f, app, content_area),
    }

    // Help text at bottom
    let help_text = get_help_text(app);
    let help_widget = Paragraph::new(help_text)
        .style(Style::default().bg(Color::Black).fg(Color::White))
        .alignment(ratatui::layout::Alignment::Center)
        .block(Block::default());
    
    let help_area = Rect::new(0, size.height.saturating_sub(1), size.width, 1);
    f.render_widget(help_widget, help_area);
}

/// Renders the commits view.
fn render_commits_view(f: &mut Frame, app: &mut App, area: Rect) {
    // Layout for list (left) and details (right)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .split(area);

    // --- Commit List ---
    let items: Vec<ListItem> = app
        .commits
        .iter()
        .enumerate()
        .map(|(index, c)| {
            let selected_indicator = if app.selected_commits.contains(&index) { "✓ " } else { "  " };
            let content = format!("{}[{}] {} - {}", selected_indicator, c.hash, c.author, c.summary);
            let style = if app.selected_commits.contains(&index) {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Gray)
            };
            ListItem::new(content).style(style)
        })
        .collect();

    let title = if app.selected_commits.len() > 0 {
        format!("Commit History ({} selected)", app.selected_commits.len())
    } else {
        "Commit History".to_string()
    };
    
    let list = List::new(items)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(if app.selected_commits.len() > 0 { 
                    Style::default().fg(Color::Yellow) 
                } else { 
                    Style::default().fg(Color::Green) 
                }),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(ratatui::style::Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, chunks[0], &mut app.commit_state);

    // --- Details Panel ---
    let details_block = Block::default()
        .title("Git & NIP-34 Operations")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));
    f.render_widget(details_block, chunks[1]);

    if app.show_full_commit {
        if let Some(ref details) = app.full_commit_details {
            let details_chunk = chunks[1].inner(ratatui::layout::Margin {
                horizontal: 1,
                vertical: 1,
            });
            f.render_widget(
                Paragraph::new(details.clone())
                    .style(Style::default().fg(Color::White)),
                details_chunk,
            );
        }
    } else {
        let details_chunk = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(1), // Instructions
                Constraint::Length(1), // Selection count
                Constraint::Min(0),    // Tips
            ])
            .split(chunks[1].inner(ratatui::layout::Margin {
                horizontal: 1,
                vertical: 1,
            }));

        f.render_widget(
            Paragraph::new("Use ↑↓ to navigate, Space to select (max 2)")
                .style(Style::default().fg(Color::Cyan)),
            details_chunk[0],
        );

        f.render_widget(
            Paragraph::new(format!("Selected: {} commits", app.selected_commits.len()))
                .style(Style::default().fg(Color::Yellow)),
            details_chunk[1],
        );

        f.render_widget(
            Paragraph::new(format!("Press 'n' to create NIP-34 patch from selected commits"))
                .style(Style::default().fg(Color::Green)),
            details_chunk[2],
        );
    }
}

/// Renders the branches view.
fn render_branches_view(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .split(area);

    // --- Branch List ---
    let items: Vec<ListItem> = app
        .branches
        .iter()
        .map(|b| {
            let prefix = if b.is_current {
                "* "
            } else if b.is_remote {
                "R "
            } else {
                "  "
            };
            let content = format!("{}{} - {}", prefix, b.name, b.commit_message);
            let style = if b.is_current {
                Style::default().fg(Color::Green)
            } else if b.is_remote {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::Gray)
            };
            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title("Git Branches")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(ratatui::style::Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, chunks[0], &mut app.branch_state);

    // --- Details Panel ---
    let details_block = Block::default()
        .title("Branch Operations")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));
    f.render_widget(details_block, chunks[1]);
}

/// Renders the NIP-34 events view.
fn render_nip34_view(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .split(area);

    // --- NIP-34 Events List ---
    let items: Vec<ListItem> = app
        .nip34_events
        .iter()
        .enumerate()
        .map(|(index, event)| {
            let selected_indicator = if app.selected_nip34_events.contains(&index) { "✓ " } else { "  " };
            let kind_name = match event.kind {
                1617 => "Repo Announcement",
                1618 => "Repo State",
                1617 => "Patch",
                1618 => "Pull Request",
                1619 => "PR Update",
                1621 => "Issue",
                _ => "Unknown",
            };
            
            let content_preview = if event.content.len() > 50 {
                format!("{}...", &event.content[..47])
            } else {
                event.content.clone()
            };

            let content = format!("{}[{}] {} - {}", 
                selected_indicator, 
                &event.id[..8], 
                kind_name,
                content_preview
            );
            let style = if app.selected_nip34_events.contains(&index) {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Gray)
            };
            ListItem::new(content).style(style)
        })
        .collect();

    let title = if app.selected_nip34_events.len() > 0 {
        format!("NIP-34 Events ({} selected)", app.selected_nip34_events.len())
    } else {
        "NIP-34 Events".to_string()
    };

    let list = List::new(items)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(if app.selected_nip34_events.len() > 0 { 
                    Style::default().fg(Color::Yellow) 
                } else { 
                    Style::default().fg(Color::Green) 
                }),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(ratatui::style::Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, chunks[0], &mut app.nip34_state);

    // --- Event Details Panel ---
    let details_block = Block::default()
        .title("NIP-34 Event Details")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));
    f.render_widget(details_block, chunks[1]);

    if let Some(selected_index) = app.nip34_state.selected() {
        if let Some(event) = app.nip34_events.get(selected_index) {
            let details_chunk = chunks[1].inner(ratatui::layout::Margin {
                horizontal: 1,
                vertical: 1,
            });
            
            let event_details = format!(
                "Event ID: {}\n\
                Public Key: {}\n\
                Kind: {} ({})\n\
                Created: {}\n\
                Content: {}\n\n\
                Tags:\n{}",
                event.id,
                event.pubkey,
                event.kind,
                match event.kind {
                    1617 => "Repo Announcement",
                    1618 => "Repo State", 
                    1617 => "Patch",
                    1618 => "Pull Request",
                    1619 => "PR Update",
                    1621 => "Issue",
                    _ => "Unknown",
                },
                chrono::DateTime::from_timestamp(event.created_at as i64, 0)
                    .map(|dt| dt.naive_local())
                    .unwrap_or_default()
                    .format("%Y-%m-%d %H:%M:%S"),
                event.content,
                event.tags.iter()
                    .map(|tag| format!("  {}: {}", tag.get(0).unwrap_or(&"".to_string()), 
                        tag.get(1).unwrap_or(&"".to_string())))
                    .collect::<Vec<_>>()
                    .join("\n")
            );

            f.render_widget(
                Paragraph::new(event_details)
                    .style(Style::default().fg(Color::White)),
                details_chunk,
            );
        }
    }
}

/// Gets help text based on current mode and state.
fn get_help_text(app: &App) -> String {
    let base_help = "Controls: [Tab] Switch | [q/Esc] Quit | [j/Down] Next | [k/Up] Previous";
    
    match app.current_mode {
        NavigatorMode::Commits => {
            let selection_help = if app.selected_commits.len() > 0 { 
                if app.selected_commits.len() == 1 {
                    " | [Space] Select 1 more | [n] Create Patch"
                } else {
                    " | [c] Clear | [n] Create Patch"
                }
            } else { 
                "" 
            };
            
            if app.show_full_commit {
                format!("{} | [Left] Summary{}", base_help, selection_help)
            } else {
                format!("{} | [Space] Select | [Right] Diff{}", base_help, selection_help)
            }
        }
        NavigatorMode::Branches => {
            format!("{} | [Enter] Checkout", base_help)
        }
        NavigatorMode::Nip34Events => {
            let selection_help = if app.selected_nip34_events.len() > 0 { 
                " | [c] Clear" 
            } else { 
                "" 
            };
            format!("{} | [Space] Select{}", base_help, selection_help)
        }
    }
}

/// Initializes the terminal and runs the application.
fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let app = App::new()?;
    let tick_rate = Duration::from_millis(250);
    let res = run_app(&mut terminal, app, tick_rate);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(e) = res {
        println!("{e}");
    }

    Ok(())
}