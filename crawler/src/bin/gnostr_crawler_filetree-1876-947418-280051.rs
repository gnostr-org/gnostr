use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    fs,
    io,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{Context, Result};
use gnostr_crawler::{relays::get_config_dir_path, tui::JsonPanel};
use gnostr_filetreelist::{FileTree, MoveSelection};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
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
    let mut tree = BucketedCrawlerTree::discover(root)?;
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

#[derive(Clone, Debug, PartialEq, Eq)]
struct BucketSummary {
    name: String,
    files: usize,
}

#[derive(Clone, Debug)]
struct FileEntry {
    real: PathBuf,
    virtual_path: PathBuf,
    bucket: String,
    format: String,
}

struct BucketedCrawlerTree {
    root: PathBuf,
    tree: FileTree,
    entries: Vec<FileEntry>,
    virtual_to_real: HashMap<PathBuf, PathBuf>,
    buckets: Vec<BucketSummary>,
}

impl BucketedCrawlerTree {
    fn discover(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        let files = collect_files(&root)?;
        let entries = build_entries(&root, &files)?;
        let buckets = summarize_buckets(&entries);
        let tree = build_tree(&entries)?;
        let virtual_to_real = entries
            .iter()
            .map(|entry| (entry.virtual_path.clone(), entry.real.clone()))
            .collect::<HashMap<_, _>>();

        Ok(Self {
            root,
            tree,
            entries,
            virtual_to_real,
            buckets,
        })
    }

    fn refresh(&mut self) -> Result<()> {
        let next = Self::discover(&self.root)?;
        self.tree = next.tree;
        self.entries = next.entries;
        self.virtual_to_real = next.virtual_to_real;
        self.buckets = next.buckets;
        Ok(())
    }

    fn move_selection(&mut self, dir: MoveSelection) -> bool {
        self.tree.move_selection(dir)
    }

    fn selected_path(&self) -> Option<&Path> {
        self.tree
            .selected_file()
            .and_then(|info| self.virtual_to_real.get(info.full_path()))
            .map(PathBuf::as_path)
    }

    fn sorted_buckets(&self) -> Vec<BucketSummary> {
        let mut buckets = self.buckets.clone();
        buckets.sort_by(|a, b| a.name.cmp(&b.name));
        buckets
    }

    fn render_items(&self) -> Vec<ListItem<'static>> {
        let total = self.tree.iterate(0, self.tree.iterate(0, usize::MAX).count());
        total
            .map(|(item, selected)| {
                let indent = "  ".repeat(item.info().indent() as usize);
                let icon = if item.kind().is_path() {
                    if item.kind().is_path_collapsed() {
                        "▸"
                    } else {
                        "▾"
                    }
                } else {
                    " "
                };
                let label = format!("{indent}{icon} {}", item.info().path_str());
                let style = if selected {
                    Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(Line::from(vec![
                    Span::styled(label, style),
                    Span::raw("  "),
                    Span::styled(
                        if item.kind().is_path() { "bucket" } else { "file" },
                        Style::default().fg(Color::DarkGray),
                    ),
                ]))
            })
            .collect()
    }
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
    fn sync(&mut self, path: Option<&Path>) -> Result<()> {
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

fn draw(frame: &mut Frame, tree: &BucketedCrawlerTree, selected: &SelectedFileView) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(2)])
        .split(frame.area());

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(48), Constraint::Percentage(52)])
        .split(root[1]);

    frame.render_widget(header(tree), root[0]);
    frame.render_widget(tree_panel(tree), body[0]);
    render_selected(frame, body[1], selected);
    frame.render_widget(footer(tree), root[2]);
}

fn header(tree: &BucketedCrawlerTree) -> Paragraph<'static> {
    let bucket_line = tree
        .sorted_buckets()
        .into_iter()
        .map(|bucket| format!("{}:{}", bucket.name, bucket.files))
        .collect::<Vec<_>>()
        .join("  ");

    Paragraph::new(Line::from(vec![
        Span::styled("crawler file buckets", Style::default().fg(Color::Cyan)),
        Span::raw("  "),
        Span::styled(tree.root.display().to_string(), Style::default().fg(Color::Yellow)),
        Span::raw("  "),
        Span::styled(bucket_line, Style::default().fg(Color::DarkGray)),
    ]))
    .block(Block::default().borders(Borders::ALL).title("gnostr"))
}

fn tree_panel(tree: &BucketedCrawlerTree) -> List<'static> {
    let mut items = vec![ListItem::new(Line::from(vec![
        Span::styled("crawler", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled("bucket/json|text|yaml", Style::default().fg(Color::DarkGray)),
    ]))];
    items.extend(tree.render_items());

    List::new(items).block(Block::default().borders(Borders::ALL).title("crawler tree"))
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

fn footer(tree: &BucketedCrawlerTree) -> Paragraph<'static> {
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
    .block(Block::default().borders(Borders::ALL))
}

fn collect_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    walk(root, &mut files)?;
    files.sort();
    Ok(files)
}

fn walk(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir).with_context(|| format!("reading {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        if file_name == "target" || file_name == ".git" || file_name.starts_with('.') && path.is_dir() {
            continue;
        }
        let meta = entry.metadata()?;
        if meta.is_dir() {
            walk(&path, files)?;
        } else if meta.is_file() {
            files.push(path);
        }
    }
    Ok(())
}

fn build_entries(root: &Path, files: &[PathBuf]) -> Result<Vec<FileEntry>> {
    files.iter().map(|file| build_entry(root, file)).collect()
}

fn build_entry(root: &Path, real: &Path) -> Result<FileEntry> {
    let relative = real
        .strip_prefix(root)
        .with_context(|| format!("path {} is outside {}", real.display(), root.display()))?;
    let bucket = bucket_name(relative, real);
    let format = file_format(real);
    let virtual_path = virtual_path(&bucket, &format, relative, real)?;

    Ok(FileEntry {
        real: real.to_path_buf(),
        virtual_path,
        bucket,
        format,
    })
}

fn bucket_name(relative: &Path, real: &Path) -> String {
    relative
        .components()
        .next()
        .and_then(|component| component.as_os_str().to_str())
        .map(String::from)
        .unwrap_or_else(|| {
            real.file_stem()
                .and_then(|stem| stem.to_str())
                .map(String::from)
                .unwrap_or_else(|| String::from("root"))
        })
}

fn file_format(real: &Path) -> String {
    match real
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
        .as_deref()
    {
        Some("json") => String::from("json"),
        Some("yaml") | Some("yml") => String::from("yaml"),
        Some("txt") => String::from("text"),
        Some(other) => other.to_string(),
        None => String::from("text"),
    }
}

fn virtual_path(bucket: &str, format: &str, relative: &Path, real: &Path) -> Result<PathBuf> {
    let mut virtual_path = PathBuf::from("crawler");
    virtual_path.push(bucket);
    virtual_path.push(format);

    let mut components = relative.components();
    let _ = components.next();
    let remaining = components.collect::<Vec<_>>();

    if remaining.is_empty() {
        virtual_path.push(
            real.file_name()
                .and_then(|name| name.to_str())
                .context("missing file name")?,
        );
    } else {
        for component in remaining {
            virtual_path.push(component.as_os_str());
        }
    }

    Ok(virtual_path)
}

fn summarize_buckets(entries: &[FileEntry]) -> Vec<BucketSummary> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for entry in entries {
        *counts.entry(entry.bucket.clone()).or_insert(0) += 1;
    }

    counts
        .into_iter()
        .map(|(name, files)| BucketSummary { name, files })
        .collect()
}

fn build_tree(entries: &[FileEntry]) -> Result<FileTree> {
    let refs = entries
        .iter()
        .map(|entry| entry.virtual_path.as_path())
        .collect::<Vec<_>>();
    let collapsed = BTreeSet::new();
    let mut tree = FileTree::new(&refs, &collapsed)?;
    tree.collapse_but_root();
    Ok(tree)
}
