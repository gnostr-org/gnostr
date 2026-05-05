use std::{
    collections::BTreeSet,
    error::Error,
    fs,
    io,
    io::stdout,
    path::{Path, PathBuf},
    process::Command,
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use filetreelist::{FileTree, MoveSelection};
use ratatui::{
    backend::CrosstermBackend,
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use crate::{upstream, workflow};

pub fn collect_checkout_paths(dir: impl AsRef<Path>) -> io::Result<Vec<PathBuf>> {
    let output = Command::new("git")
        .args(["ls-files", "--cached", "--others", "--exclude-standard"])
        .current_dir(dir)
        .output()?;

    if !output.status.success() {
        return Err(io::Error::other(format!(
            "git ls-files failed with exit code {:?}\nstdout: {}\nstderr: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    let mut paths: Vec<PathBuf> = String::from_utf8(output.stdout)
        .map_err(io::Error::other)?
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(PathBuf::from)
        .collect();

    paths.sort();
    paths.dedup();
    Ok(paths)
}

fn build_tree(paths: &[PathBuf]) -> io::Result<FileTree> {
    let path_refs: Vec<&Path> = paths.iter().map(PathBuf::as_path).collect();
    let collapsed: BTreeSet<&String> = BTreeSet::new();
    filetreelist::FileTree::new(&path_refs, &collapsed).map_err(|err| io::Error::other(err.to_string()))
}

struct App {
    checkout_dir: PathBuf,
    tree: FileTree,
    content_scroll: u16,
    show_tree: bool,
    status_line: String,
}

impl App {
    fn new(checkout_dir: PathBuf, paths: Vec<PathBuf>) -> io::Result<Self> {
        Ok(Self {
            checkout_dir,
            tree: build_tree(&paths)?,
            content_scroll: 0,
            show_tree: true,
            status_line: String::from("e: edit  p: propose  g: git ui  r: refresh  q: quit"),
        })
    }

    fn reload(&mut self) -> io::Result<()> {
        let selected = self.selected_entry_path();
        let paths = collect_checkout_paths(&self.checkout_dir)?;
        self.tree = build_tree(&paths)?;
        if let Some(selected) = selected {
            let _ = self.tree.select_file(selected.as_path());
        }
        self.content_scroll = 0;
        Ok(())
    }

    fn selected_entry_path(&self) -> Option<PathBuf> {
        let visible = self.tree.visual_selection().map(|v| v.count).unwrap_or(0).max(1);
        self.tree
            .iterate(0, visible)
            .find(|(_, selected)| *selected)
            .map(|(item, _)| item.info().full_path().to_path_buf())
    }

    fn selected_file_path(&self) -> Option<PathBuf> {
        self.tree
            .selected_file()
            .map(|info| self.checkout_dir.join(info.full_path()))
    }

    fn move_selection(&mut self, direction: MoveSelection) {
        if self.tree.move_selection(direction) {
            self.content_scroll = 0;
        }
    }

    fn page_move(&mut self, down: bool) {
        for _ in 0..10 {
            if !self.tree.move_selection(if down {
                MoveSelection::Down
            } else {
                MoveSelection::Up
            }) {
                break;
            }
        }
        self.content_scroll = 0;
    }
}

fn content_for_selection(app: &App) -> (String, String) {
    if let Some(selected_file) = app.selected_file_path() {
        let title = selected_file
            .strip_prefix(&app.checkout_dir)
            .unwrap_or(selected_file.as_path())
            .display()
            .to_string();
        let content = fs::read_to_string(&selected_file)
            .unwrap_or_else(|_| format!("Error reading file: {}", selected_file.display()));
        return (title, content);
    }

    if let Some(selected_entry) = app.selected_entry_path() {
        let title = selected_entry.display().to_string();
        return (title.clone(), format!("Directory selected: {title}"));
    }

    (String::from("Content"), String::from("No file selected."))
}

fn render_tree(frame: &mut Frame, app: &App, area: Rect) {
    let visible = app.tree.visual_selection().map(|v| v.count).unwrap_or(0).max(1);
    let items: Vec<ListItem> = app
        .tree
        .iterate(0, visible)
        .map(|(item, selected)| {
            let indent = "  ".repeat(item.info().indent() as usize);
            let marker = if item.kind().is_path() {
                if item.kind().is_path_collapsed() {
                    "▸ "
                } else {
                    "▾ "
                }
            } else {
                "• "
            };
            let label = format!("{indent}{marker}{}", item.info().path_str());
            let style = if selected {
                Style::default().fg(Color::Black).bg(Color::LightGreen)
            } else if item.kind().is_path() {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };

            ListItem::new(label).style(style)
        })
        .collect();

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title("Files"));
    frame.render_widget(list, area);
}

fn ui(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(1), Constraint::Min(0)])
        .split(frame.area());

    let title = Paragraph::new("Nostr NIPs Browser")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    frame.render_widget(title, chunks[0]);

    let status = Paragraph::new(app.status_line.clone())
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Left);
    frame.render_widget(status, chunks[1]);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(if app.show_tree {
            vec![Constraint::Percentage(35), Constraint::Percentage(65)]
        } else {
            vec![Constraint::Percentage(100)]
        })
        .split(chunks[2]);

    if app.show_tree {
        render_tree(frame, app, body[0]);
    }

    let (content_title, content_text) = content_for_selection(app);
    let content = Paragraph::new(content_text)
        .block(Block::default().borders(Borders::ALL).title(content_title))
        .wrap(Wrap { trim: false })
        .scroll((app.content_scroll, 0));

    frame.render_widget(content, body[if app.show_tree { 1 } else { 0 }]);
}

fn editor_target(app: &App) -> Option<PathBuf> {
    app.selected_file_path()
}

pub fn run_default() -> Result<(), Box<dyn Error>> {
    let checkout_dir = upstream::ensure_checkout()?;
    let mut app = App::new(checkout_dir.clone(), collect_checkout_paths(&checkout_dir)?)?;
    if upstream::worktree_dirty(&checkout_dir)? {
        app.status_line = String::from("dirty checkout: fetch only, local edits preserved");
    }

    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let result = (|| -> Result<(), Box<dyn Error>> {
        loop {
            terminal.draw(|f| ui(f, &mut app))?;

            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Esc | KeyCode::Enter => app.show_tree = !app.show_tree,
                    KeyCode::Left => {
                        app.move_selection(MoveSelection::Left);
                        app.show_tree = false;
                    }
                    KeyCode::Right => {
                        app.move_selection(MoveSelection::Right);
                        app.show_tree = false;
                    }
                    KeyCode::Down | KeyCode::Char('j') => app.move_selection(MoveSelection::Down),
                    KeyCode::Up | KeyCode::Char('k') => app.move_selection(MoveSelection::Up),
                    KeyCode::PageDown => app.page_move(true),
                    KeyCode::PageUp => app.page_move(false),
                    KeyCode::Char('r') => match upstream::ensure_checkout() {
                        Ok(_) => match app.reload() {
                            Ok(_) => app.status_line = String::from("refreshed upstream checkout"),
                            Err(err) => app.status_line = format!("refresh failed: {err}"),
                        },
                        Err(err) => app.status_line = format!("refresh failed: {err}"),
                    },
                    KeyCode::Char('e') => {
                        if let Some(file_path) = editor_target(&app) {
                            disable_raw_mode()?;
                            stdout().execute(LeaveAlternateScreen)?;
                            let editor_result = workflow::launch_editor(&file_path);
                            stdout().execute(EnterAlternateScreen)?;
                            enable_raw_mode()?;
                            app.status_line = match editor_result {
                                Ok(_) => match app.reload() {
                                    Ok(_) => format!("edited {}", file_path.display()),
                                    Err(err) => format!("edited {}; reload failed: {err}", file_path.display()),
                                },
                                Err(err) => format!("editor error: {err}"),
                            };
                        } else {
                            app.status_line = String::from("select a file to edit");
                        }
                    }
                    KeyCode::Char('g') => {
                        disable_raw_mode()?;
                        stdout().execute(LeaveAlternateScreen)?;
                        let git_result = workflow::launch_git_tui(&app.checkout_dir);
                        stdout().execute(EnterAlternateScreen)?;
                        enable_raw_mode()?;
                        app.status_line = match git_result {
                            Ok(_) => String::from("returned from asyncgit TUI"),
                            Err(err) => format!("git ui error: {err}"),
                        };
                        let _ = app.reload();
                    }
                    KeyCode::Char('p') => {
                        if let Some(file_path) = editor_target(&app) {
                            app.status_line = String::from("publishing proposal...");
                            terminal.draw(|f| ui(f, &mut app))?;
                            let result = workflow::submit_proposal(&app.checkout_dir, &file_path);
                            app.status_line = match result {
                                Ok(hash) => format!("submitted proposal {hash}"),
                                Err(err) => format!("proposal failed: {err}"),
                            };
                            let _ = app.reload();
                        } else {
                            app.status_line = String::from("select a file to propose");
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    })();

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    terminal.show_cursor()?;

    result
}
