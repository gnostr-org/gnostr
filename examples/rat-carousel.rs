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
    author: String,
    summary: String,
    committer_date: String,
}

/// The main application state.
struct App {
    commits: Vec<Commit>,
    state: ListState,
}

impl App {
    /// Constructs a new App with commit history loaded from the current git
    /// repository.
    fn new() -> Result<Self> {
        let repo = git2::Repository::open_from_env()?;
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

                let hash = commit.id().to_string();
                let summary = commit.summary().unwrap_or_default().to_string();

                Commit {
                    hash: hash.chars().take(8).collect(),
                    author: author.name().unwrap_or("Unknown").to_string(),
                    summary,
                    committer_date,
                }
            })
            .collect();

        let mut state = ListState::default();
        if !commits.is_empty() {
            state.select(Some(0));
        }

        Ok(Self { commits, state })
    }

    /// Moves the carousel selection up.
    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.commits.len() - 1
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
                if i >= self.commits.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
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

    // Layout for commit list (left) and details (right)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .split(size);

    // --- Commit List (Carousel) ---
    let items: Vec<ListItem> = app
        .commits
        .iter()
        .map(|c| {
            let content = format!("[{}] {} - {}", c.hash, c.author, c.summary);
            ListItem::new(content).style(Style::default().fg(Color::Gray))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title("Commit History Carousel")
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

    // --- Commit Details ---
    let block = Block::default()
        .title("Commit Details (Read-Only)")
        .borders(Borders::ALL);
    f.render_widget(block, chunks[1]);

    if let Some(selected_index) = app.state.selected() {
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
                Paragraph::new(format!("Author: {}", commit.author.clone().green().bold())),
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

    // --- Help Text ---
    let help_text = Paragraph::new("Controls: [q/Esc] Quit | [j/Down] Next | [k/Up] Previous")
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
