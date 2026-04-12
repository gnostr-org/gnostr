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

/// The main application state.
struct App {
    branches: Vec<Branch>,
    state: ListState,
    repo: git2::Repository,
}

impl App {
    /// Constructs a new App with branch data loaded from the current git
    /// repository.
    fn new() -> Result<Self> {
        let repo = git2::Repository::open_from_env()?;
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

        // Sort branches: current branch first, then local branches, then remote
        // branches
        branches.sort_by(|a, b| match (a.is_current, b.is_current) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => match (a.is_remote, b.is_remote) {
                (false, true) => std::cmp::Ordering::Less,
                (true, false) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            },
        });

        let mut state = ListState::default();
        if !branches.is_empty() {
            state.select(Some(0));
        }

        Ok(Self {
            branches,
            state,
            repo,
        })
    }

    /// Moves the carousel selection up.
    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.branches.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    /// Moves the carousel selection down.
    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.branches.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    /// Switches to the selected branch (only for local branches).
    fn checkout_branch(&mut self) -> Result<()> {
        if let Some(selected_index) = self.state.selected() {
            if let Some(branch) = self.branches.get(selected_index) {
                if !branch.is_remote {
                    let branch_name = &branch.name;
                    let branch_ref = format!("refs/heads/{}", branch_name);

                    // Checkout the branch
                    let reference = self.repo.find_reference(&branch_ref)?;
                    self.repo
                        .set_head(reference.name().expect("Reference should have a name"))?;

                    // Update the current branch status
                    for b in &mut self.branches {
                        b.is_current = false;
                    }
                    self.branches[selected_index].is_current = true;
                }
            }
        }
        Ok(())
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
                        KeyCode::Down | KeyCode::Char('j') => app.next(),
                        KeyCode::Up | KeyCode::Char('k') => app.previous(),
                        KeyCode::Enter | KeyCode::Char(' ') => {
                            if let Err(_e) = app.checkout_branch() {
                                // Could show error message in UI
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

    // Layout for branch list (left) and details (right)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .split(size);

    // --- Branch List (Carousel) ---
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
                .title("Git Branch Navigator")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(ratatui::style::Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, chunks[0], &mut app.state);

    // --- Branch Details ---
    let block = Block::default()
        .title("Branch Details")
        .borders(Borders::ALL);
    f.render_widget(block, chunks[1]);

    if let Some(selected_index) = app.state.selected() {
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

    // --- Help Text ---
    let help_text = Paragraph::new(
        "Controls: [q/Esc] Quit | [j/Down] Next | [k/Up] Previous | [Enter/Space] Checkout",
    )
    .style(Style::default().bg(Color::Black).fg(Color::White))
    .alignment(ratatui::layout::Alignment::Center)
    .block(Block::default());

    // Position the help text at the bottom
    let help_area = Rect::new(0, size.height.saturating_sub(1), size.width, 1);
    f.render_widget(help_text, help_area);
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
