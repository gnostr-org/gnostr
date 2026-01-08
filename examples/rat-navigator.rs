use std::{
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
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};

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

/// Navigation modes for the application.
#[derive(Debug, Clone, PartialEq)]
enum NavigatorMode {
    Commits,
    Branches,
}

/// The main application state.
struct App {
    commits: Vec<Commit>,
    branches: Vec<Branch>,
    commit_state: ListState,
    branch_state: ListState,
    current_mode: NavigatorMode,
    repo: git2::Repository,
    full_commit_details: Option<String>,
    show_full_commit: bool,
    selected_commits: std::collections::HashSet<usize>,
}

impl App {
    /// Constructs a new App with commit and branch data loaded from the current git
    /// repository.
    fn new() -> Result<Self> {
        let repo = git2::Repository::open_from_env()?;

        // Load commits
        let mut revwalk = repo.revwalk()?;
        revwalk.push_head()?;

        let commits: Vec<Commit> = revwalk
            .filter_map(|id| id.ok())
            .filter_map(|oid| repo.find_commit(oid).ok())
            .take(100) // Limit to 100 commits for performance
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

        // Load branches
        let mut branches = Vec::new();

        // Get local branches
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
                        let time = author.when();
                        let _datetime = chrono::DateTime::from_timestamp(time.seconds(), 0)
                            .map(|dt| dt.naive_local())
                            .unwrap_or_default();

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

        // Get remote branches
        for branch_ref in repo.branches(Some(git2::BranchType::Remote))? {
            let (branch, _) = branch_ref?;
            if let Some(branch_name) = branch.name()? {
                let branch_ref_name = format!("refs/remotes/{}", branch_name);
                if let Ok(reference) = repo.find_reference(&branch_ref_name) {
                    if let Some(commit) = reference.peel_to_commit().ok() {
                        let author = commit.author();
                        let time = author.when();
                        let _datetime = chrono::DateTime::from_timestamp(time.seconds(), 0)
                            .map(|dt| dt.naive_local())
                            .unwrap_or_default();

                        branches.push(Branch {
                            name: branch_name.to_string(),
                            commit_hash: commit.id().to_string().chars().take(8).collect(),
                            commit_message: commit.summary().unwrap_or_default().to_string(),
                            author: author.name().unwrap_or("Unknown").to_string(),
                            is_current: false,
                            is_remote: true,
                        });
                    }
                }
            }
        }

        // Sort branches: current branch first, then local branches, then remote branches
        branches.sort_by(|a, b| match (a.is_current, b.is_current) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => match (a.is_remote, b.is_remote) {
                (false, true) => std::cmp::Ordering::Less,
                (true, false) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            },
        });

        let mut commit_state = ListState::default();
        if !commits.is_empty() {
            commit_state.select(Some(0));
        }

        let mut branch_state = ListState::default();
        if !branches.is_empty() {
            branch_state.select(Some(0));
        }

        Ok(Self {
            commits,
            branches,
            commit_state,
            branch_state,
            current_mode: NavigatorMode::Branches,
            repo,
            full_commit_details: None,
            show_full_commit: false,
            selected_commits: std::collections::HashSet::new(),
        })
    }

    /// Switches between navigation modes.
    fn switch_mode(&mut self) {
        self.current_mode = match self.current_mode {
            NavigatorMode::Commits => NavigatorMode::Branches,
            NavigatorMode::Branches => NavigatorMode::Commits,
        };
    }

    /// Moves the selection up in the current mode.
    fn previous(&mut self) {
        match self.current_mode {
            NavigatorMode::Commits => {
                let i = match self.commit_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            self.commits.len() - 1
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                self.commit_state.select(Some(i));
            }
            NavigatorMode::Branches => {
                let i = match self.branch_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            self.branches.len() - 1
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                self.branch_state.select(Some(i));
            }
        }
    }

    /// Moves the selection down in the current mode.
    fn next(&mut self) {
        match self.current_mode {
            NavigatorMode::Commits => {
                let i = match self.commit_state.selected() {
                    Some(i) => {
                        if i >= self.commits.len() - 1 {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                self.commit_state.select(Some(i));
            }
            NavigatorMode::Branches => {
                let i = match self.branch_state.selected() {
                    Some(i) => {
                        if i >= self.branches.len() - 1 {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                self.branch_state.select(Some(i));
            }
        }
    }

    /// Performs mode-specific action (checkout branch in branches mode).
    fn action(&mut self) -> Result<()> {
        match self.current_mode {
            NavigatorMode::Commits => {
                // No action for commits mode (read-only)
            }
            NavigatorMode::Branches => {
                if let Some(selected_index) = self.branch_state.selected() {
                    if let Some(branch) = self.branches.get(selected_index) {
                        if !branch.is_remote {
                            let branch_name = &branch.name;
                            let branch_ref = format!("refs/heads/{}", branch_name);

                            // Checkout the branch
                            let reference = self.repo.find_reference(&branch_ref)?;
                            self.repo.set_head(
                                reference.name().expect("Reference should have a name"),
                            )?;

                            // Update the current branch status
                            for b in &mut self.branches {
                                b.is_current = false;
                            }
                            self.branches[selected_index].is_current = true;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Loads git diff for selected commits.
    fn load_full_commit(&mut self) -> Result<()> {
        let mut diff_content = String::new();

        if !self.selected_commits.is_empty() {
            // Multiple commits selected - show summary
            diff_content.push_str("╭─────────────────────────────────────────────────────────╮\n");
            diff_content.push_str(&format!(
                "│              Selected {} Commit(s)                │\n",
                self.selected_count()
            ));
            diff_content
                .push_str("╰─────────────────────────────────────────────────────────╯\n\n");

            let mut selected_indices: Vec<_> = self.selected_commits.iter().cloned().collect();
            selected_indices.sort();

            for (count, &index) in selected_indices.iter().enumerate() {
                if let Some(commit) = self.commits.get(index) {
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
            diff_content.push_str(
                "💡 Tip: Press 'c' to clear selection and view individual commit diffs\n",
            );
        } else if let Some(selected_index) = self.commit_state.selected() {
            // No commits selected, load current focused commit
            if let Some(commit) = self.commits.get(selected_index) {
                // Find the full commit object using the hash
                let commit_oid = git2::Oid::from_str(&commit.full_hash)?;
                if let Ok(git_commit) = self.repo.find_commit(commit_oid) {
                    // Reuse existing diff_content variable declared above

                    // Add commit details header widget
                    diff_content
                        .push_str("╭─────────────────────────────────────────────────────────╮\n");
                    diff_content
                        .push_str("│                    Commit Details                    │\n");
                    diff_content
                        .push_str("╰─────────────────────────────────────────────────────────╯\n");
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
                            git_commit
                                .committer()
                                .email()
                                .unwrap_or("unknown@example.com")
                        ));
                        diff_content.push_str(&format!(
                            "Commit Date: {}\n",
                            chrono::DateTime::from_timestamp(
                                git_commit.committer().when().seconds(),
                                0
                            )
                            .map(|dt| dt.naive_local())
                            .unwrap_or_default()
                            .format("%Y-%m-%d %H:%M:%S")
                        ));
                    } else {
                        diff_content.push_str(&format!(
                            "Date: {}\n",
                            chrono::DateTime::from_timestamp(
                                git_commit.author().when().seconds(),
                                0
                            )
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
                    diff_content
                        .push_str("╭─────────────────────────────────────────────────────────╮\n");
                    diff_content
                        .push_str("│                      Git Diff                       │\n");
                    diff_content.push_str(
                        "╰─────────────────────────────────────────────────────────╯\n\n",
                    );

                    // Get the diff against parent(s)
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
                            diff_content
                                .push_str("🔀 Merge Commit - Diff against first parent:\n\n");
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

                    self.full_commit_details = Some(diff_content);
                    self.show_full_commit = true;
                }
            }
        }
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
                            patch.push_str(&format!(
                                "diff --git a/{} b/{}\n",
                                old_path.display(),
                                new_path.display()
                            ));
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

    /// Toggles full commit view.
    fn toggle_full_commit(&mut self) {
        self.show_full_commit = !self.show_full_commit;
    }

    /// Toggles selection of the current commit.
    fn toggle_commit_selection(&mut self) {
        if let Some(selected_index) = self.commit_state.selected() {
            if self.selected_commits.contains(&selected_index) {
                self.selected_commits.remove(&selected_index);
            } else {
                self.selected_commits.insert(selected_index);
            }
        }
    }

    /// Clears all selected commits.
    fn clear_selection(&mut self) {
        self.selected_commits.clear();
    }

    /// Returns the number of selected commits.
    fn selected_count(&self) -> usize {
        self.selected_commits.len()
    }
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
                        KeyCode::Enter => {
                            if let Err(_e) = app.action() {
                                // Could show error message in UI
                            }
                        }
                        KeyCode::Char(' ') => {
                            match app.current_mode {
                                NavigatorMode::Commits => app.toggle_commit_selection(),
                                NavigatorMode::Branches => {
                                    if let Err(_e) = app.action() {
                                        // Could show error message in UI
                                    }
                                }
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
                                app.toggle_full_commit();
                            }
                        }
                        KeyCode::Char('c') => {
                            if app.current_mode == NavigatorMode::Commits {
                                app.clear_selection();
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

    // Layout for list (left) and details (right)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .split(size);

    match app.current_mode {
        NavigatorMode::Commits => {
            // --- Commit List ---
            let items: Vec<ListItem> = app
                .commits
                .iter()
                .enumerate()
                .map(|(index, c)| {
                    let selected_indicator = if app.selected_commits.contains(&index) {
                        "✓ "
                    } else {
                        "  "
                    };
                    let content = format!(
                        "{}[{}] {} - {}",
                        selected_indicator, c.hash, c.author, c.summary
                    );
                    let style = if app.selected_commits.contains(&index) {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::Gray)
                    };
                    ListItem::new(content).style(style)
                })
                .collect();

            let title = if app.selected_count() > 0 {
                format!("Commit History ({} selected)", app.selected_count())
            } else {
                "Commit History".to_string()
            };

            let list = List::new(items)
                .block(
                    Block::default()
                        .title(title)
                        .borders(Borders::ALL)
                        .border_style(if app.selected_count() > 0 {
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

            // --- Commit Details ---
            let title = if app.show_full_commit {
                "Git Diff (Read-Only) - [Left] to return"
            } else {
                "Commit Details (Read-Only) - [Right] for git diff"
            };
            let block = Block::default().title(title).borders(Borders::ALL);
            f.render_widget(block, chunks[1]);

            if let Some(selected_index) = app.commit_state.selected() {
                if app.show_full_commit {
                    // Show full commit details
                    if let Some(ref full_details) = app.full_commit_details {
                        let details_chunk = chunks[1].inner(ratatui::layout::Margin {
                            horizontal: 1,
                            vertical: 1,
                        });
                        f.render_widget(
                            Paragraph::new(full_details.clone())
                                .style(Style::default().fg(Color::White)),
                            details_chunk,
                        );
                    }
                } else {
                    // Show summary details
                    if let Some(commit) = app.commits.get(selected_index) {
                        let details_chunks = Layout::default()
                            .direction(Direction::Vertical)
                            .margin(1)
                            .constraints([
                                Constraint::Length(1), // Hash
                                Constraint::Length(1), // Author
                                Constraint::Length(1), // Date
                                Constraint::Min(0),    // Summary
                            ])
                            .split(chunks[1].inner(ratatui::layout::Margin {
                                horizontal: 1,
                                vertical: 1,
                            }));

                        f.render_widget(
                            Paragraph::new(format!("Hash: {}", commit.hash.clone().bold()))
                                .style(Style::default().fg(Color::Yellow)),
                            details_chunks[0],
                        );
                        f.render_widget(
                            Paragraph::new(format!(
                                "Author: {}",
                                commit.author.clone().green().bold()
                            )),
                            details_chunks[1],
                        );
                        f.render_widget(
                            Paragraph::new(format!(
                                "Date: {}",
                                commit.committer_date.to_string().cyan()
                            )),
                            details_chunks[2],
                        );
                        f.render_widget(
                            Paragraph::new(format!("Summary: \n{}", commit.summary)),
                            details_chunks[3],
                        );
                    }
                }
            }
        }
        NavigatorMode::Branches => {
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

            // --- Branch Details ---
            let block = Block::default()
                .title("Branch Details")
                .borders(Borders::ALL);
            f.render_widget(block, chunks[1]);

            if let Some(selected_index) = app.branch_state.selected() {
                if let Some(branch) = app.branches.get(selected_index) {
                    let details_chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(1)
                        .constraints([
                            Constraint::Length(1), // Name
                            Constraint::Length(1), // Type
                            Constraint::Length(1), // Commit
                            Constraint::Length(1), // Author
                            Constraint::Min(0),    // Message
                        ])
                        .split(chunks[1].inner(ratatui::layout::Margin {
                            horizontal: 1,
                            vertical: 1,
                        }));

                    let branch_type = if branch.is_current {
                        "Current Branch".to_string().green().bold()
                    } else if branch.is_remote {
                        "Remote Branch".to_string().cyan().bold()
                    } else {
                        "Local Branch".to_string().yellow()
                    };

                    f.render_widget(
                        Paragraph::new(format!("Name: {}", branch.name.clone().bold()))
                            .style(Style::default().fg(Color::Yellow)),
                        details_chunks[0],
                    );
                    f.render_widget(
                        Paragraph::new(format!("Type: {}", branch_type)),
                        details_chunks[1],
                    );
                    f.render_widget(
                        Paragraph::new(format!("Commit: {}", branch.commit_hash.clone().bold()))
                            .style(Style::default().fg(Color::Magenta)),
                        details_chunks[2],
                    );
                    f.render_widget(
                        Paragraph::new(format!("Author: {}", branch.author.clone().green().bold())),
                        details_chunks[3],
                    );
                    f.render_widget(
                        Paragraph::new(format!("Message:\n{}", branch.commit_message)),
                        details_chunks[4],
                    );
                }
            }
        }
    }

    // --- Mode Indicator and Help Text ---
    let mode_text = match app.current_mode {
        NavigatorMode::Commits => "COMMITS MODE".green().bold(),
        NavigatorMode::Branches => "BRANCHES MODE".blue().bold(),
    };

    let help_text = match app.current_mode {
        NavigatorMode::Commits => {
            let base_help = "Controls: [Tab] Switch Mode | [q/Esc] Quit | [j/Down] Next | [k/Up] Previous";
            let selection_help = if app.selected_count() > 0 { 
                " | [c] Clear selection" 
            } else { 
                "" 
            };
            
            if app.show_full_commit {
                format!("{} | [Left] Summary{}", base_help, selection_help)
            } else {
                format!("{} | [Space] Select | [Right] Git Diff{}", base_help, selection_help)
            }
        }
        NavigatorMode::Branches => {
            "Controls: [Tab] Switch Mode | [q/Esc] Quit | [j/Down] Next | [k/Up] Previous | [Enter/Space] Checkout".to_string()
        }
    };

    let mode_indicator = Paragraph::new(format!("{} | {}", mode_text, help_text))
        .style(Style::default().bg(Color::Black).fg(Color::White))
        .alignment(ratatui::layout::Alignment::Center)
        .block(Block::default());

    // Position the help text at the bottom
    let help_area = Rect::new(0, size.height.saturating_sub(1), size.width, 1);
    f.render_widget(mode_indicator, help_area);
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
    execute!(terminal.backend_mut(), LeaveAlternateScreen,)?;
    terminal.show_cursor()?;

    if let Err(e) = res {
        println!("{e}");
    }

    Ok(())
}
