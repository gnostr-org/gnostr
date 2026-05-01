use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    fs,
    io,
    path::{Path, PathBuf},
    collections::HashSet,
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
    let mut input_mode = InputMode::Normal;
    let mut search_query = String::new();
    let mut status_message: Option<String> = None;
    selected.sync(tree.selected_path())?;

    loop {
        selected.sync(tree.selected_path())?;
        terminal
            .terminal
            .draw(|frame| draw(frame, &tree, &selected, input_mode, &search_query, status_message.as_deref()))?;

        if event::poll(Duration::from_millis(150))? {
            match event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Char('q') if matches!(input_mode, InputMode::Normal) => break,
                    KeyCode::Char('/') if matches!(input_mode, InputMode::Normal) => {
                        input_mode = InputMode::Search;
                        search_query = tree.active_filter().unwrap_or_default();
                        status_message = None;
                    }
                    _ if matches!(input_mode, InputMode::Search) => match key.code {
                        KeyCode::Esc => {
                            input_mode = InputMode::Normal;
                            status_message = None;
                        }
                        KeyCode::Tab => {
                            if let Some(completion) = tree.best_completion(&search_query) {
                                search_query = completion;
                                tree.apply_filter(Some(&search_query))?;
                                selected.sync(tree.selected_path())?;
                                status_message = Some(format!(
                                    "completed: {} ({} paths)",
                                    search_query,
                                    tree.visible_count()
                                ));
                            }
                        }
                        KeyCode::Enter => {
                            input_mode = InputMode::Normal;
                            status_message = if search_query.trim().is_empty() {
                                Some(String::from("filter cleared"))
                            } else {
                                Some(format!(
                                    "filter: {} ({} paths)",
                                    search_query,
                                    tree.visible_count()
                                ))
                            };
                        }
                        KeyCode::Backspace => {
                            search_query.pop();
                            tree.apply_filter(Some(&search_query))?;
                            selected.sync(tree.selected_path())?;
                            status_message = Some(format!(
                                "filter: {} ({} paths)",
                                search_query,
                                tree.visible_count()
                            ));
                        }
                        KeyCode::Char(c) if !c.is_control() => {
                            search_query.push(c);
                            tree.apply_filter(Some(&search_query))?;
                            selected.sync(tree.selected_path())?;
                            status_message = Some(format!(
                                "filter: {} ({} paths)",
                                search_query,
                                tree.visible_count()
                            ));
                        }
                        _ => {}
                    },
                    KeyCode::Down | KeyCode::Char('j') => {
                        status_message = None;
                        tree.move_selection(MoveSelection::Down);
                    }
                    KeyCode::Char(' ') => {
                        status_message = match tree.toggle_selected_favorite()? {
                            Some(true) => Some(String::from("selected bucket")),
                            Some(false) => Some(String::from("cleared selected bucket")),
                            None => Some(String::from("select a relay file to toggle bucket")),
                        };
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        status_message = None;
                        tree.move_selection(MoveSelection::Up);
                    }
                    KeyCode::Left | KeyCode::Char('h') => {
                        status_message = None;
                        tree.move_selection(MoveSelection::Left);
                    }
                    KeyCode::Right | KeyCode::Char('l') => {
                        status_message = None;
                        tree.move_selection(MoveSelection::Right);
                    }
                    KeyCode::Home => {
                        status_message = None;
                        tree.move_selection(MoveSelection::Top);
                    }
                    KeyCode::End => {
                        status_message = None;
                        tree.move_selection(MoveSelection::End);
                    }
                    KeyCode::PageDown => {
                        status_message = None;
                        tree.move_selection(MoveSelection::PageDown);
                    }
                    KeyCode::PageUp => {
                        status_message = None;
                        tree.move_selection(MoveSelection::PageUp);
                    }
                    KeyCode::Char('r') => {
                        status_message = None;
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
    active_filter: Option<String>,
    favorites: HashSet<String>,
    favorites_path: PathBuf,
}

impl BucketedCrawlerTree {
    fn discover(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        let files = collect_files(&root)?;
        let mut entries = build_entries(&root, &files)?;
        sort_entries_for_tree(&mut entries);
        let buckets = summarize_buckets(&entries);
        let tree = build_tree(&entries)?;
        let virtual_to_real = entries
            .iter()
            .map(|entry| (entry.virtual_path.clone(), entry.real.clone()))
            .collect::<HashMap<_, _>>();
        let favorites_path = root.join("filetree-favorites.txt");
        let favorites = load_favorites(&favorites_path)?;

        Ok(Self {
            root,
            tree,
            entries,
            virtual_to_real,
            buckets,
            active_filter: None,
            favorites,
            favorites_path,
        })
    }

    fn refresh(&mut self) -> Result<()> {
        let filter = self.active_filter.clone();
        let next = Self::discover(&self.root)?;
        self.entries = next.entries;
        self.apply_filter(filter.as_deref())
    }

    fn move_selection(&mut self, dir: MoveSelection) -> bool {
        self.tree.move_selection(dir)
    }

    fn apply_filter(&mut self, query: Option<&str>) -> Result<()> {
        self.active_filter = query
            .map(str::trim)
            .filter(|query| !query.is_empty())
            .map(ToOwned::to_owned);

        let filtered_entries = self.filtered_entries();
        self.buckets = summarize_buckets(&filtered_entries);
        self.tree = build_tree(&filtered_entries)?;
        self.virtual_to_real = filtered_entries
            .iter()
            .map(|entry| (entry.virtual_path.clone(), entry.real.clone()))
            .collect::<HashMap<_, _>>();
        Ok(())
    }

    fn active_filter(&self) -> Option<String> {
        self.active_filter.clone()
    }

    fn visible_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|entry| self.entry_matches_filter(entry))
            .count()
    }

    fn selected_path(&self) -> Option<&Path> {
        self.tree
            .selected_file()
            .and_then(|info| self.virtual_to_real.get(info.full_path()))
            .map(PathBuf::as_path)
    }

    fn favorite_items(&self) -> Vec<String> {
        let mut buckets: BTreeMap<String, Vec<(usize, String)>> = BTreeMap::new();

        for path in &self.favorites {
            let path = Path::new(path);
            let relative = path.strip_prefix(&self.root).unwrap_or(path);
            let bucket = relative
                .components()
                .next()
                .and_then(|component| component.as_os_str().to_str())
                .unwrap_or("(root)")
                .to_string();
            let name = path
                .file_stem()
                .and_then(|name| name.to_str())
                .unwrap_or(path.to_str().unwrap_or_default())
                .to_string();
            buckets
                .entry(bucket)
                .or_default()
                .push((favorite_rank(&name), name));
        }

        let mut items = Vec::new();
        for (bucket, mut entries) in buckets {
            entries.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
            if let Some((_, name)) = entries.into_iter().next() {
                items.push(format!("{name} ({bucket})"));
            }
        }
        items
    }

    fn toggle_selected_favorite(&mut self) -> Result<Option<bool>> {
        let Some(path) = self.selected_path() else {
            return Ok(None);
        };

        if let Some(bucket) = self.selected_relay_bucket(path) {
            let bucket_keys = self.bucket_keys(bucket);
            let favored = bucket_keys.iter().all(|key| self.favorites.contains(key));

            if favored {
                for key in bucket_keys {
                    self.favorites.remove(&key);
                }
            } else {
                for key in bucket_keys {
                    self.favorites.insert(key);
                }
            }

            save_favorites(&self.favorites_path, &self.favorites)?;
            return Ok(Some(!favored));
        }

        let key = path.display().to_string();
        let favored = if self.favorites.contains(&key) {
            self.favorites.remove(&key);
            false
        } else {
            self.favorites.insert(key);
            true
        };
        save_favorites(&self.favorites_path, &self.favorites)?;
        Ok(Some(favored))
    }

    fn sorted_buckets(&self) -> Vec<BucketSummary> {
        let mut buckets = self.buckets.clone();
        buckets.sort_by(|a, b| a.name.cmp(&b.name));
        buckets
    }

    fn bucket_for_real<'a>(&'a self, path: &'a Path) -> Option<&'a str> {
        let relative = path.strip_prefix(&self.root).ok()?;
        relative
            .components()
            .next()
            .and_then(|component| component.as_os_str().to_str())
    }

    fn bucket_is_favorited(&self, bucket: &str) -> bool {
        self.favorites.iter().any(|path| {
            let path = Path::new(path);
            let relative = path.strip_prefix(&self.root).unwrap_or(path);
            relative
                .components()
                .next()
                .and_then(|component| component.as_os_str().to_str())
                == Some(bucket)
        })
    }

    fn selected_relay_bucket<'a>(&'a self, path: &'a Path) -> Option<&'a str> {
        let name = path.file_name().and_then(|name| name.to_str())?;
        if !matches!(
            name.to_ascii_lowercase().as_str(),
            "relay.json" | "relays.json" | "relay.txt" | "relays.txt" | "relay.yaml" | "relays.yaml" | "relay.yml" | "relays.yml"
        ) {
            return None;
        }

        self.bucket_for_real(path)
    }

    fn bucket_keys(&self, bucket: &str) -> Vec<String> {
        self.entries
            .iter()
            .filter(|entry| entry.bucket == bucket)
            .map(|entry| entry.real.display().to_string())
            .collect()
    }

    fn filtered_entries(&self) -> Vec<FileEntry> {
        self.entries
            .iter()
            .filter(|entry| self.entry_matches_filter(entry))
            .cloned()
            .collect()
    }

    fn entry_matches_filter(&self, entry: &FileEntry) -> bool {
        self.active_filter
            .as_ref()
            .is_none_or(|filter| {
                fuzzy_score(&entry.virtual_path.to_string_lossy(), filter).is_some()
            })
    }

    fn best_completion(&self, query: &str) -> Option<String> {
        let needle = query.trim();
        if needle.is_empty() {
            return None;
        }

        self.filtered_entries()
            .into_iter()
            .filter_map(|entry| {
                let path = entry.virtual_path.to_string_lossy().into_owned();
                fuzzy_score(&path, needle).map(|score| (score, path))
            })
            .min_by(|a, b| a.0.cmp(&b.0).then(a.1.len().cmp(&b.1.len())))
            .map(|(_, path)| path)
    }

    fn render_items(&self, start: usize, max: usize) -> Vec<ListItem<'static>> {
        let total = self.tree.iterate(start, max);
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
                let favorite = if item.kind().is_path() {
                    " "
                } else {
                    let bucket = self
                        .virtual_to_real
                        .get(item.info().full_path())
                        .and_then(|path| self.bucket_for_real(path));
                    if bucket.is_some_and(|bucket| self.bucket_is_favorited(bucket)) {
                        "♥"
                    } else {
                        " "
                    }
                };
                let label = format!("{indent}{icon} {favorite} {}", item.info().path_str());
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

fn draw(
    frame: &mut Frame,
    tree: &BucketedCrawlerTree,
    selected: &SelectedFileView,
    input_mode: InputMode,
    search_query: &str,
    status_message: Option<&str>,
) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(5)])
        .split(frame.area());

    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(26)])
        .split(root[0]);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(48), Constraint::Percentage(52)])
        .split(root[1]);

    frame.render_widget(header(tree), top[0]);
    frame.render_widget(
        search_box(input_mode, search_query, tree.best_completion(search_query).as_deref()),
        top[1],
    );

    let tree_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(5), Constraint::Min(0)])
        .split(body[0]);
    frame.render_widget(search_box(input_mode, search_query, tree.best_completion(search_query).as_deref()).block(
        Block::default().borders(Borders::ALL).title("tree search"),
    ), tree_area[0]);
    frame.render_widget(favorites_panel(tree), tree_area[1]);
    frame.render_widget(tree_panel_list(tree, tree_area[2]), tree_area[2]);
    render_selected(frame, body[1], selected);
    frame.render_widget(footer(tree, status_message), root[2]);
}

fn header(tree: &BucketedCrawlerTree) -> Paragraph<'static> {
    let bucket_line = tree
        .sorted_buckets()
        .into_iter()
        .map(|bucket| format!("{}:{}", bucket.name, bucket.files))
        .collect::<Vec<_>>()
        .join("  ");
    let filter_label = tree
        .active_filter()
        .map(|filter| format!("filter: {filter}"))
        .unwrap_or_else(|| String::from("filter: all"));

    Paragraph::new(Line::from(vec![
        Span::styled("crawler file buckets", Style::default().fg(Color::Cyan)),
        Span::raw("  "),
        Span::styled(filter_label, Style::default().fg(Color::Magenta)),
        Span::raw("  "),
        Span::styled(tree.root.display().to_string(), Style::default().fg(Color::Yellow)),
        Span::raw("  "),
        Span::styled(bucket_line, Style::default().fg(Color::DarkGray)),
    ]))
    .block(Block::default().borders(Borders::ALL).title("gnostr"))
}

fn search_box(
    input_mode: InputMode,
    search_query: &str,
    completion: Option<&str>,
) -> Paragraph<'static> {
    let label = if matches!(input_mode, InputMode::Search) {
        format!("/{}", search_query)
    } else {
        String::from("/ search")
    };

    let hint = match (input_mode, completion) {
        (InputMode::Search, Some(completion)) if completion != search_query => {
            format!("tab → {}", completion)
        }
        (InputMode::Search, _) => String::from("type to filter"),
        (InputMode::Normal, Some(_)) => String::from("tab fuzzy complete"),
        (InputMode::Normal, None) => String::from("type / to search"),
    };

    Paragraph::new(vec![
        Line::from(vec![Span::styled(
            label,
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(hint, Style::default().fg(Color::DarkGray))]),
    ])
    .block(Block::default().borders(Borders::ALL).title("search"))
}

fn favorites_panel(tree: &BucketedCrawlerTree) -> Paragraph<'static> {
    let favorites = tree.favorite_items();
    let lines = if favorites.is_empty() {
        vec![Line::from(vec![Span::styled(
            "space favorite selected row",
            Style::default().fg(Color::DarkGray),
        )])]
    } else {
        let mut lines = vec![Line::from(vec![Span::styled(
            format!("favorites ({})", favorites.len()),
            Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
        )])];
        lines.extend(favorites.into_iter().map(|item| Line::from(vec![Span::raw(item)])));
        lines
    };

    Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("favorites"))
}

fn tree_panel_list(tree: &BucketedCrawlerTree, area: ratatui::layout::Rect) -> List<'static> {
    let mut items = vec![ListItem::new(Line::from(vec![
        Span::styled("crawler", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled("bucket/json|text|yaml", Style::default().fg(Color::DarkGray)),
    ]))];
    let view_height = area.height.saturating_sub(2).max(1) as usize;
    let (start, count) = tree
        .tree
        .visual_selection()
        .map(|selection| {
            let half = view_height / 2;
            let start = selection.index.saturating_sub(half);
            let start = start.min(selection.count.saturating_sub(view_height).max(0));
            (start, selection.count)
        })
        .unwrap_or((0, 0));
    let visible = if count == 0 {
        vec![ListItem::new(Line::from("no files found"))]
    } else {
        tree.render_items(start, view_height)
    };
    items.extend(visible);

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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum InputMode {
    Normal,
    Search,
}

fn footer(
    tree: &BucketedCrawlerTree,
    status_message: Option<&str>,
) -> Paragraph<'static> {
    let status_line = status_message.unwrap_or_default().to_string();

    Paragraph::new(vec![
        Line::from(vec![
            Span::styled("q quit", Style::default().fg(Color::DarkGray)),
            Span::raw("  "),
            Span::styled("hjkl/arrows move", Style::default().fg(Color::DarkGray)),
            Span::raw("  "),
            Span::styled("space favorite", Style::default().fg(Color::DarkGray)),
            Span::raw("  "),
            Span::styled("r refresh", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled("type filters live", Style::default().fg(Color::Cyan)),
            Span::raw("  "),
            Span::styled("tab fuzzy complete", Style::default().fg(Color::Cyan)),
            Span::raw("  "),
            Span::styled("esc cancel", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::raw(""),
        ]),
        Line::from(vec![
            Span::styled(
                format!(
                    "selected: {}",
                    tree.selected_path()
                        .map(|p| p.display().to_string())
                        .unwrap_or_default()
                ),
                Style::default().fg(Color::Green),
            ),
        ]),
        Line::from(vec![
            Span::styled(status_line, Style::default().fg(Color::Red)),
        ]),
    ])
    .block(Block::default().borders(Borders::ALL).title("help"))
}

fn collect_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    walk(root, &mut files)?;
    files.sort();
    Ok(files)
}

fn fuzzy_score(candidate: &str, query: &str) -> Option<usize> {
    let candidate = candidate.to_ascii_lowercase();
    let query = query.trim().to_ascii_lowercase();
    if query.is_empty() {
        return Some(0);
    }

    let mut search_from = 0usize;
    let mut score = 0usize;

    for needle in query.chars() {
        let mut found = None;
        for (offset, ch) in candidate[search_from..].char_indices() {
            if ch == needle {
                found = Some(search_from + offset);
                break;
            }
        }

        let index = found?;
        score += index.saturating_sub(search_from);
        search_from = index + needle.len_utf8();
    }

    score += candidate.len().saturating_sub(search_from);
    Some(score)
}

fn load_favorites(path: &Path) -> Result<HashSet<String>> {
    if !path.exists() {
        return Ok(HashSet::new());
    }

    let content = fs::read_to_string(path)?;
    Ok(content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect())
}

fn save_favorites(path: &Path, favorites: &HashSet<String>) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut values = favorites.iter().cloned().collect::<Vec<_>>();
    values.sort();
    fs::write(path, values.join("\n"))?;
    Ok(())
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

fn favorite_rank(name: &str) -> usize {
    match name.to_ascii_lowercase().as_str() {
        "relay.json" | "relays.json" => 0,
        "relay.txt" | "relays.txt" => 1,
        "relay.yaml" | "relays.yaml" | "relay.yml" | "relays.yml" => 2,
        _ => 3,
    }
}

fn sort_entries_for_tree(entries: &mut [FileEntry]) {
    entries.sort_by(|a, b| {
        let a_parent = a.virtual_path.parent().unwrap_or_else(|| Path::new(""));
        let b_parent = b.virtual_path.parent().unwrap_or_else(|| Path::new(""));
        let a_name = a
            .virtual_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        let b_name = b
            .virtual_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();

        a_parent
            .cmp(b_parent)
            .then_with(|| favorite_rank(a_name).cmp(&favorite_rank(b_name)))
            .then_with(|| a.virtual_path.cmp(&b.virtual_path))
    });
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
