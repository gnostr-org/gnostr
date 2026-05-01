use std::{fs, io, time::Duration};

use anyhow::Result;
use gnostr_crawler::{
    relays::get_config_dir_path,
    tui::{CrawlerDiskTree, CrawlerFileTreeWidget, JsonPanel, MoveSelection},
};
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
use serde_json::Value;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    run()
}

pub fn run() -> Result<()> {
    let mut terminal = TerminalGuard::enter()?;
    let root = get_config_dir_path();
    fs::create_dir_all(&root)?;
    let mut tree = CrawlerDiskTree::discover(root)?;
    let mut selected = SelectedJson::default();
    selected.sync(tree.selected_path())?;

    loop {
        selected.sync(tree.selected_path())?;
        terminal.terminal.draw(|frame| draw(frame, &tree, &selected))?;

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
                        selected.sync(tree.selected_path())?;
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

#[derive(Default)]
struct SelectedJson {
    path: Option<PathBuf>,
    value: Option<Value>,
    error: Option<String>,
}

impl SelectedJson {
    fn sync(&mut self, path: Option<&std::path::Path>) -> Result<()> {
        match path {
            Some(path) if self.path.as_deref() != Some(path) => {
                self.path = Some(path.to_path_buf());
                let content = fs::read_to_string(path)?;
                match serde_json::from_str::<Value>(&content) {
                    Ok(value) => {
                        self.value = Some(value);
                        self.error = None;
                    }
                    Err(error) => {
                        self.value = None;
                        self.error = Some(error.to_string());
                    }
                }
            }
            None => {
                self.path = None;
                self.value = None;
                self.error = None;
            }
            _ => {}
        }
        Ok(())
    }
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

fn draw(frame: &mut Frame, tree: &CrawlerDiskTree, selected: &SelectedJson) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(2)])
        .split(frame.area());

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(48), Constraint::Percentage(52)])
        .split(root[1]);

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
        body[0],
    );
    frame.render_widget(
        JsonPanel {
            title: "selected json",
            value: selected.value.as_ref(),
            error: selected.error.as_deref(),
            empty_message: "select a file to parse json",
            scroll: (0, 0),
        },
        body[1],
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
                format!(
                    "selected: {}",
                    tree.selected_path()
                        .map(|p| p.display().to_string())
                        .unwrap_or_default()
                ),
                Style::default().fg(Color::Green),
            ),
        ]))
        .block(Block::default().borders(Borders::ALL)),
        root[2],
    );
}
