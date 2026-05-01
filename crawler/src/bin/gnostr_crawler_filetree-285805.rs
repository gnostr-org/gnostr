use std::{io, path::PathBuf, time::Duration};

use anyhow::Result;
use gnostr_crawler::tui::{CrawlerDiskTree, CrawlerFileTreeWidget, MoveSelection};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use ratatui::crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

#[tokio::main]
async fn main() -> Result<()> {
    run()
}

fn run() -> Result<()> {
    let mut terminal = TerminalGuard::enter()?;
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut tree = CrawlerDiskTree::discover(root)?;

    loop {
        terminal.terminal.draw(|frame| draw(frame, &tree))?;

        if event::poll(Duration::from_millis(150))? {
            match event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Down | KeyCode::Char('j') => {
                        tree.move_selection(MoveSelection::Down);
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        tree.move_selection(MoveSelection::Up);
                    }
                    KeyCode::Left | KeyCode::Char('h') => {
                        tree.move_selection(MoveSelection::Left);
                    }
                    KeyCode::Right | KeyCode::Char('l') => {
                        tree.move_selection(MoveSelection::Right);
                    }
                    KeyCode::Home => {
                        tree.move_selection(MoveSelection::Top);
                    }
                    KeyCode::End => {
                        tree.move_selection(MoveSelection::End);
                    }
                    KeyCode::PageDown => {
                        tree.move_selection(MoveSelection::PageDown);
                    }
                    KeyCode::PageUp => {
                        tree.move_selection(MoveSelection::PageUp);
                    }
                    KeyCode::Char('r') => {
                        tree.refresh()?;
                    }
                    _ => {}
                },
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
    }

    Ok(())
}

struct TerminalGuard {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl TerminalGuard {
    fn enter() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        Ok(Self {
            terminal: Terminal::new(CrosstermBackend::new(stdout))?,
        })
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}

fn draw(frame: &mut Frame, tree: &CrawlerDiskTree) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(2)])
        .split(frame.area());

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                "crawler file buckets",
                Style::default().fg(Color::Cyan),
            ),
            Span::raw("  "),
            Span::styled(
                tree.root().display().to_string(),
                Style::default().fg(Color::Yellow),
            ),
        ]))
        .block(Block::default().borders(Borders::ALL).title("gnostr")),
        root[0],
    );
    frame.render_widget(
        CrawlerFileTreeWidget {
            tree,
            title: "crawler on disk",
        },
        root[1],
    );
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("q quit", Style::default().fg(Color::DarkGray)),
            Span::raw("  "),
            Span::styled("hjkl/arrows move", Style::default().fg(Color::DarkGray)),
            Span::raw("  "),
            Span::styled("r refresh", Style::default().fg(Color::DarkGray)),
            Span::raw("  "),
            Span::styled(
                format!("selected: {}", tree.selected_path().map(|p| p.display().to_string()).unwrap_or_default()),
                Style::default().fg(Color::Green),
            ),
        ]))
        .block(Block::default().borders(Borders::ALL)),
        root[2],
    );
}
