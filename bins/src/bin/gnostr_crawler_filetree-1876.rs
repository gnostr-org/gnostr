#![allow(dead_code)]

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

#[tokio::main]
async fn main() -> Result<()> {
    run()
}

pub fn run() -> Result<()> {
    let mut terminal = TerminalGuard::enter()?;
    let root = get_config_dir_path();
    fs::create_dir_all(&root)?;
    let mut tree = CrawlerDiskTree::discover(root)?;
    let mut selected = SelectedFileView::default();
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

enum SelectedFileView {
    Empty,
    RawText { text: String },
    Json { value: Value },
}

impl Default for SelectedFileView {
    fn default() -> Self {
        Self::Empty
    }
}

impl SelectedFileView {
    fn sync(&mut self, path: Option<&std::path::Path>) -> Result<()> {
        let Some(path) = path else {
            *self = Self::Empty;
            return Ok(());
        };

        let content = fs::read_to_string(path)?;
        let ext = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_ascii_lowercase());

        match ext.as_deref() {
            Some("txt") => {
                *self = Self::RawText { text: content };
            }
            Some("yaml") | Some("yml") => {
                let parsed: serde_yaml::Value = serde_yaml::from_str(&content)?;
                let value = serde_json::to_value(parsed)?;
                *self = Self::Json { value };
            }
            Some("json") => {
                let value = serde_json::from_str::<Value>(&content)?;
                *self = Self::Json { value };
            }
            _ => {
                if let Ok(value) = serde_json::from_str::<Value>(&content) {
                    *self = Self::Json { value };
                } else if let Ok(parsed) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                    let value = serde_json::to_value(parsed)?;
                    *self = Self::Json { value };
                } else {
                    *self = Self::RawText { text: content };
                }
            }
        }

        Ok(())
    }
}

fn render_selected(frame: &mut Frame, area: ratatui::layout::Rect, selected: &SelectedFileView) {
    match selected {
        SelectedFileView::Empty => {
            frame.render_widget(
                JsonPanel {
                    title: "selected data",
                    value: None,
                    error: None,
                    empty_message: "select a file to inspect its contents",
                    scroll: (0, 0),
                },
                area,
            );
        }
        SelectedFileView::RawText { text } => {
            let lines = text.lines().map(Line::from).collect::<Vec<_>>();
            frame.render_widget(
                Paragraph::new(lines)
                    .block(Block::default().borders(Borders::ALL).title("selected text")),
                area,
            );
        }
        SelectedFileView::Json { value } => {
            frame.render_widget(
                JsonPanel {
                    title: "selected data",
                    value: Some(value),
                    error: None,
                    empty_message: "",
                    scroll: (0, 0),
                },
                area,
            );
        }
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

fn draw(frame: &mut Frame, tree: &CrawlerDiskTree, selected: &SelectedFileView) {
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
    render_selected(frame, body[1], selected);
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
