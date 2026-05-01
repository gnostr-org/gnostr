use std::{
    collections::HashSet,
    io,
    sync::mpsc::{self, Receiver, Sender},
    time::Duration,
};

use anyhow::{anyhow, Context, Result};
use crawler::{
    build_gnostr_query, fetch_relay_texts, load_relays_or_bootstrap, parse_relay_metadata, send,
    Relay,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use ratatui::crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use serde_json::Value;

const CORE_KINDS: &[u32] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 16, 25];
const CHAT_KINDS: &[u32] = &[4, 9, 10, 11, 12, 13, 14, 16, 40, 41, 42, 43, 44, 1311, 22242, 24133];
const LIST_KINDS: &[u32] = &[
    10000, 10001, 10002, 10003, 10004, 10005, 10006, 10007, 10009, 10015, 10030, 10050, 10096,
    30000, 30001, 30002, 30003, 30004, 30008, 30009, 30015, 30017, 30018, 30019, 30020, 30023,
    30024, 30030, 30063, 30078,
];
const GIT_KINDS: &[u32] = &[1617, 1621, 1622, 1630, 1631, 1632, 1633, 30617, 30618];
const MARKET_KINDS: &[u32] = &[
    8, 1021, 1022, 1040, 1059, 1063, 1984, 1985, 7000, 7001, 9041, 9734, 9735, 9802, 13194, 21000,
    23194, 23195, 30402, 30403, 30818, 31922, 31923, 31924, 31925,
];

#[tokio::main]
async fn main() -> Result<()> {
    run().await
}

async fn run() -> Result<()> {
    let mut terminal = TerminalGuard::enter()?;
    let mut app = App::new().await?;

    loop {
        app.drain_updates();
        terminal.terminal.draw(|frame| app.draw(frame))?;

        if event::poll(Duration::from_millis(150))? {
            match event::read()? {
                Event::Key(key) => {
                    if app.handle_key(key)? {
                        break;
                    }
                }
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
        let backend = CrosstermBackend::new(stdout);
        Ok(Self {
            terminal: Terminal::new(backend)?,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Focus {
    Types,
    Events,
    Detail,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FeedKind {
    RelayMetadata,
    Query { label: &'static str, kinds: &'static [u32] },
}

impl FeedKind {
    fn label(self) -> &'static str {
        match self {
            FeedKind::RelayMetadata => "crawler relay metadata",
            FeedKind::Query { label, .. } => label,
        }
    }
}

#[derive(Clone, Debug)]
struct TypeItem {
    name: String,
    feed: FeedKind,
    summary: String,
}

#[derive(Clone, Debug)]
struct EventItem {
    title: String,
    subtitle: String,
    source: String,
    kind_label: String,
    raw: String,
    detail: String,
}

#[derive(Debug)]
enum LoadMessage {
    Ready {
        request_id: u64,
        feed: FeedKind,
        events: Vec<EventItem>,
    },
    Failed {
        request_id: u64,
        feed: FeedKind,
        error: String,
    },
}

struct App {
    types: Vec<TypeItem>,
    selected_type: usize,
    selected_event: usize,
    detail_scroll: u16,
    focus: Focus,
    help_open: bool,
    loading: bool,
    status: String,
    request_id: u64,
    active_request: u64,
    events: Vec<EventItem>,
    tx: Sender<LoadMessage>,
    rx: Receiver<LoadMessage>,
}

impl App {
    async fn new() -> Result<Self> {
        let types = build_type_items();
        let (tx, rx) = mpsc::channel();
        let mut app = Self {
            types,
            selected_type: 0,
            selected_event: 0,
            detail_scroll: 0,
            focus: Focus::Types,
            help_open: false,
            loading: false,
            status: String::from("select a type to load real crawler data"),
            request_id: 0,
            active_request: 0,
            events: Vec::new(),
            tx,
            rx,
        };
        app.refresh();
        Ok(app)
    }

    fn current_type(&self) -> &TypeItem {
        &self.types[self.selected_type]
    }

    fn current_event(&self) -> Option<&EventItem> {
        self.events.get(self.selected_event)
    }

    fn refresh(&mut self) {
        self.request_id = self.request_id.saturating_add(1);
        self.active_request = self.request_id;
        self.loading = true;
        self.detail_scroll = 0;
        let feed = self.current_type().feed;
        let type_name = self.current_type().name.clone();
        let tx = self.tx.clone();
        let request_id = self.request_id;
        self.status = format!("loading {} from real crawler data", type_name);

        tokio::spawn(async move {
            let result = load_feed(feed).await;
            let message = match result {
                Ok(events) => LoadMessage::Ready {
                    request_id,
                    feed,
                    events,
                },
                Err(error) => LoadMessage::Failed {
                    request_id,
                    feed,
                    error: error.to_string(),
                },
            };
            let _ = tx.send(message);
        });
    }

    fn drain_updates(&mut self) {
        while let Ok(message) = self.rx.try_recv() {
            match message {
                LoadMessage::Ready {
                    request_id,
                    feed,
                    events,
                } if request_id == self.active_request && feed == self.current_type().feed => {
                    self.loading = false;
                    self.events = events;
                    self.selected_event = 0;
                    self.detail_scroll = 0;
                    self.status = format!(
                        "loaded {} real events for {}",
                        self.events.len(),
                        self.current_type().name
                    );
                }
                LoadMessage::Failed {
                    request_id,
                    feed,
                    error,
                } if request_id == self.active_request && feed == self.current_type().feed => {
                    self.loading = false;
                    self.events.clear();
                    self.selected_event = 0;
                    self.detail_scroll = 0;
                    self.status = format!("failed to load {}: {}", self.current_type().name, error);
                }
                _ => {}
            }
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        if self.help_open {
            match key.code {
                KeyCode::Esc | KeyCode::Char('?') => self.help_open = false,
                KeyCode::Char('q') => return Ok(true),
                _ => {}
            }
            return Ok(false);
        }

        match key.code {
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Char('?') => self.help_open = true,
            KeyCode::Tab => self.focus = next_focus(self.focus),
            KeyCode::BackTab => self.focus = prev_focus(self.focus),
            KeyCode::Enter => match self.focus {
                Focus::Types => self.refresh(),
                Focus::Events => self.focus = Focus::Detail,
                Focus::Detail => {}
            },
            KeyCode::Char('r') => self.refresh(),
            KeyCode::Char('g') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.selected_type = 0;
                self.selected_event = 0;
                self.refresh();
            }
            KeyCode::Home => match self.focus {
                Focus::Types => {
                    self.selected_type = 0;
                    self.selected_event = 0;
                    self.refresh();
                }
                Focus::Events => {
                    self.selected_event = 0;
                    self.detail_scroll = 0;
                }
                Focus::Detail => self.detail_scroll = 0,
            },
            KeyCode::End => match self.focus {
                Focus::Types => {
                    if !self.types.is_empty() {
                        self.selected_type = self.types.len() - 1;
                        self.refresh();
                    }
                }
                Focus::Events => {
                    if !self.events.is_empty() {
                        self.selected_event = self.events.len() - 1;
                        self.detail_scroll = 0;
                    }
                }
                Focus::Detail => self.detail_scroll = u16::MAX / 2,
            },
            KeyCode::PageUp => {
                if matches!(self.focus, Focus::Detail) {
                    self.detail_scroll = self.detail_scroll.saturating_sub(8);
                }
            }
            KeyCode::PageDown => {
                if matches!(self.focus, Focus::Detail) {
                    self.detail_scroll = self.detail_scroll.saturating_add(8);
                }
            }
            KeyCode::Up | KeyCode::Char('k') => self.move_selection(-1),
            KeyCode::Down | KeyCode::Char('j') => self.move_selection(1),
            KeyCode::Left | KeyCode::Char('h') => self.focus = prev_focus(self.focus),
            KeyCode::Right | KeyCode::Char('l') => self.focus = next_focus(self.focus),
            _ => {}
        }

        Ok(false)
    }

    fn move_selection(&mut self, delta: i32) {
        match self.focus {
            Focus::Types => {
                self.selected_type = clamp_index(self.selected_type, self.types.len(), delta);
                self.selected_event = 0;
                self.refresh();
            }
            Focus::Events => {
                self.selected_event = clamp_index(self.selected_event, self.events.len(), delta);
                self.detail_scroll = 0;
            }
            Focus::Detail => {
                if delta < 0 {
                    self.detail_scroll = self.detail_scroll.saturating_sub(delta.unsigned_abs() as u16);
                } else {
                    self.detail_scroll = self.detail_scroll.saturating_add(delta as u16);
                }
            }
        }
    }

    fn draw(&self, frame: &mut Frame) {
        let root = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(2)])
            .split(frame.area());

        frame.render_widget(self.header(), root[0]);

        let main = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(32),
                Constraint::Percentage(34),
                Constraint::Percentage(34),
            ])
            .split(root[1]);

        let left = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(10), Constraint::Length(9)])
            .split(main[0]);
        frame.render_stateful_widget(self.types_list(), left[0], &mut self.type_state());
        frame.render_widget(self.left_detail(), left[1]);

        frame.render_stateful_widget(self.events_list(), main[1], &mut self.event_state());
        frame.render_widget(self.event_detail(frame.area(), main[2]), main[2]);

        frame.render_widget(self.status_bar(), root[2]);

        if self.help_open {
            self.draw_help(frame);
        }
    }

    fn header(&self) -> Paragraph<'_> {
        let focus = match self.focus {
            Focus::Types => "types",
            Focus::Events => "events",
            Focus::Detail => "detail",
        };
        let mut title = vec![
            Span::styled("master-detail", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            Span::raw(self.current_type().name.clone()),
            Span::raw("  "),
            Span::styled(
                format!("[{}]", self.current_type().feed.label()),
                Style::default().fg(Color::Yellow),
            ),
            Span::raw("  "),
            Span::styled(
                format!("focus: {}", focus),
                Style::default().fg(Color::Green),
            ),
        ];
        if self.loading {
            title.push(Span::raw("  loading..."));
        }
        Paragraph::new(Line::from(title)).block(Block::default().borders(Borders::ALL).title("gnostr"))
    }

    fn types_list(&self) -> List<'_> {
        let items = self
            .types
            .iter()
            .map(|item| {
                ListItem::new(Line::from(vec![
                    Span::styled(item.name.clone(), Style::default().fg(Color::White)),
                    Span::raw("  "),
                    Span::styled(
                        item.feed.label(),
                        Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
                    ),
                ]))
            })
            .collect::<Vec<_>>();

        List::new(items)
            .block(Block::default().borders(Borders::ALL).title("asyncgit types"))
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ")
    }

    fn events_list(&self) -> List<'_> {
        let items = self
            .events
            .iter()
            .map(|item| {
                ListItem::new(Line::from(vec![
                    Span::styled(item.title.clone(), Style::default().fg(Color::White)),
                    Span::raw("  "),
                    Span::styled(
                        item.subtitle.clone(),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]))
            })
            .collect::<Vec<_>>();

        List::new(items)
            .block(Block::default().borders(Borders::ALL).title("real crawler events"))
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ")
    }

    fn left_detail(&self) -> Paragraph<'_> {
        let selected_type = self.current_type();
        let selected_event = self.current_event();
        let mut lines = vec![
            Line::from(vec![
                Span::styled("type: ", Style::default().fg(Color::Green)),
                Span::raw(selected_type.name.clone()),
            ]),
            Line::from(vec![
                Span::styled("feed: ", Style::default().fg(Color::Green)),
                Span::raw(selected_type.feed.label()),
            ]),
            Line::from(vec![
                Span::styled("events: ", Style::default().fg(Color::Green)),
                Span::raw(self.events.len().to_string()),
            ]),
            Line::from(""),
        ];

        if let Some(event) = selected_event {
            lines.extend_from_slice(&[
                Line::from(vec![
                    Span::styled("selected: ", Style::default().fg(Color::Green)),
                    Span::raw(event.title.clone()),
                ]),
                Line::from(vec![
                    Span::styled("source: ", Style::default().fg(Color::Green)),
                    Span::raw(shorten(&event.source, 48)),
                ]),
                Line::from(vec![
                    Span::styled("summary: ", Style::default().fg(Color::Green)),
                    Span::raw(shorten(&event.subtitle, 52)),
                ]),
                Line::from(vec![
                    Span::styled("kind: ", Style::default().fg(Color::Green)),
                    Span::raw(event.kind_label.clone()),
                ]),
            ]);
        } else {
            lines.push(Line::from("no event selected"));
        }

        Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title("selection detail"))
            .wrap(Wrap { trim: false })
    }

    fn event_detail(&self, _area: Rect, detail_area: Rect) -> Paragraph<'_> {
        let event = self.current_event();
        let mut lines = vec![
            Line::from(vec![
                Span::styled("detail: ", Style::default().fg(Color::Green)),
                Span::raw(self.current_type().name.clone()),
            ]),
            Line::from(vec![
                Span::styled("feed: ", Style::default().fg(Color::Green)),
                Span::raw(self.current_type().feed.label()),
            ]),
        ];

        if let Some(event) = event {
            lines.extend_from_slice(&[
                Line::from(""),
                Line::from(vec![
                    Span::styled("title: ", Style::default().fg(Color::Green)),
                    Span::raw(event.title.clone()),
                ]),
                Line::from(vec![
                    Span::styled("source: ", Style::default().fg(Color::Green)),
                    Span::raw(event.source.clone()),
                ]),
                Line::from(vec![
                    Span::styled("kind: ", Style::default().fg(Color::Green)),
                    Span::raw(event.kind_label.clone()),
                ]),
                Line::from(""),
                Line::from(Span::styled(
                    "raw event data",
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                )),
            ]);
            let raw = event.raw.clone();
            return Paragraph::new({
                let mut full = lines;
                full.push(Line::from(raw));
                full
            })
            .block(Block::default().borders(Borders::ALL).title("event data"))
            .wrap(Wrap { trim: false })
            .scroll((self.detail_scroll, 0));
        }

        Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title("event data"))
            .wrap(Wrap { trim: false })
            .scroll((self.detail_scroll, 0))
    }

    fn status_bar(&self) -> Paragraph<'_> {
        let status = if self.loading { "loading" } else { "ready" };
        Paragraph::new(Line::from(vec![
            Span::styled("status: ", Style::default().fg(Color::Green)),
            Span::raw(status),
            Span::raw("  "),
            Span::styled("message: ", Style::default().fg(Color::Green)),
            Span::raw(self.status.clone()),
            Span::raw("  "),
            Span::styled(
                "keys: q quit  tab cycle  enter reload  ? help  j/k move",
                Style::default().fg(Color::DarkGray),
            ),
        ]))
        .block(Block::default().borders(Borders::ALL))
    }

    fn draw_help(&self, frame: &mut Frame) {
        let area = centered_rect(72, 60, frame.area());
        frame.render_widget(Clear, area);

        let lines = vec![
            Line::from("q  quit"),
            Line::from("tab / shift-tab  move focus"),
            Line::from("j/k or arrows  move selection"),
            Line::from("enter  reload selected type"),
            Line::from("r  refresh current feed"),
            Line::from("page up/down  scroll detail"),
            Line::from("?  toggle help"),
        ];

        frame.render_widget(
            Paragraph::new(lines)
                .block(Block::default().borders(Borders::ALL).title("help"))
                .wrap(Wrap { trim: false }),
            area,
        );
    }

    fn type_state(&self) -> ListState {
        let mut state = ListState::default();
        state.select(Some(self.selected_type));
        state
    }

    fn event_state(&self) -> ListState {
        let mut state = ListState::default();
        state.select(Some(self.selected_event));
        state
    }
}

fn build_type_items() -> Vec<TypeItem> {
    collect_asyncgit_exports()
        .into_iter()
        .map(|name| {
            let feed = feed_for_name(&name);
            TypeItem {
                summary: feed.label().to_string(),
                name,
                feed,
            }
        })
        .collect()
}

fn collect_asyncgit_exports() -> Vec<String> {
    let mut names = Vec::new();
    let mut seen = HashSet::new();
    for source in [ASYNCGIT_TYPES_MOD, ASYNCGIT_VERSIONED_MOD] {
        for statement in source.split(';') {
            let statement = statement.trim();
            if !statement.starts_with("pub use ") {
                continue;
            }
            let rest = statement.trim_start_matches("pub use ").trim();
            for name in parse_pub_use_statement(rest) {
                if seen.insert(name.clone()) {
                    names.push(name);
                }
            }
        }
    }
    names
}

fn parse_pub_use_statement(rest: &str) -> Vec<String> {
    let mut names = Vec::new();
    if rest.ends_with("::*") {
        let module = rest.trim_end_matches("::*").rsplit("::").next().unwrap_or(rest);
        let label = title_case(module);
        names.push(format!("{}::*", label));
        return names;
    }

    if let (Some(start), Some(end)) = (rest.find('{'), rest.rfind('}')) {
        let items = &rest[start + 1..end];
        for item in items.split(',') {
            let item = item.trim();
            if let Some(name) = extract_export_name(item) {
                names.push(name);
            }
        }
        return names;
    }

    if let Some(name) = extract_export_name(rest) {
        names.push(name);
    }
    names
}

fn extract_export_name(item: &str) -> Option<String> {
    let item = item.trim();
    if item.is_empty() {
        return None;
    }
    let exported = if let Some(alias) = item.split(" as ").nth(1) {
        alias.trim().to_string()
    } else {
        item.rsplit("::").next().unwrap_or(item).trim().to_string()
    };
    if exported
        .chars()
        .next()
        .map(|c| c.is_uppercase())
        .unwrap_or(false)
    {
        Some(exported)
    } else {
        None
    }
}

fn feed_for_name(name: &str) -> FeedKind {
    let lower = name.to_ascii_lowercase();
    if lower.contains("relay") {
        return FeedKind::RelayMetadata;
    }
    if lower.contains("nip34")
        || lower.contains("repo")
        || lower.contains("issue")
        || lower.contains("status")
        || lower.contains("patch")
    {
        return FeedKind::Query {
            label: "git collaboration",
            kinds: GIT_KINDS,
        };
    }
    if lower.contains("channel")
        || lower.contains("chat")
        || lower.contains("dm")
        || lower.contains("nip44")
        || lower.contains("giftwrap")
        || lower.contains("privatekey")
        || lower.contains("signer")
    {
        return FeedKind::Query {
            label: "messages and auth",
            kinds: CHAT_KINDS,
        };
    }
    if lower.contains("list")
        || lower.contains("set")
        || lower.contains("bookmark")
        || lower.contains("mute")
        || lower.contains("pin")
    {
        return FeedKind::Query {
            label: "lists and sets",
            kinds: LIST_KINDS,
        };
    }
    if lower.contains("zap")
        || lower.contains("wallet")
        || lower.contains("market")
        || lower.contains("product")
        || lower.contains("bid")
        || lower.contains("calendar")
        || lower.contains("file")
        || lower.contains("badge")
        || lower.contains("liveevent")
    {
        return FeedKind::Query {
            label: "market and calendar",
            kinds: MARKET_KINDS,
        };
    }
    FeedKind::Query {
        label: "core social",
        kinds: CORE_KINDS,
    }
}

async fn load_feed(feed: FeedKind) -> Result<Vec<EventItem>> {
    match feed {
        FeedKind::RelayMetadata => load_relay_metadata().await,
        FeedKind::Query { kinds, label } => load_query_events(label, kinds).await,
    }
}

async fn load_query_events(label: &'static str, kinds: &'static [u32]) -> Result<Vec<EventItem>> {
    let relays = load_relays_or_bootstrap();
    let urls = relays
        .into_iter()
        .map(|relay| {
            reqwest::Url::parse(&relay)
                .with_context(|| format!("invalid relay url: {}", relay))
        })
        .collect::<Result<Vec<_>>>()?;
    if urls.is_empty() {
        return Err(anyhow!("no bootstrap relays available"));
    }

    let kinds_string = kinds
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let query = build_gnostr_query(
        None,
        None,
        Some(24),
        None,
        None,
        None,
        None,
        Some(kinds_string.as_str()),
        None,
    )?;

    let messages = send(query, urls, Some(24)).await?;
    let mut seen = HashSet::new();
    let mut events = Vec::new();

    for message in messages {
        if let Some(event) = parse_event_message(&message, label) {
            if seen.insert(event.title.clone()) {
                events.push(event);
            }
        }
    }

    events.sort_by(|a, b| a.subtitle.cmp(&b.subtitle));
    Ok(events)
}

async fn load_relay_metadata() -> Result<Vec<EventItem>> {
    let relays = load_relays_or_bootstrap();
    if relays.is_empty() {
        return Err(anyhow!("no bootstrap relays available"));
    }

    let client = reqwest::Client::new();
    let responses = fetch_relay_texts(relays, &client, "master-detail").await;
    let mut events = Vec::new();
    let mut seen = HashSet::new();

    for response in responses {
        let (url, body, ping_ms) = response.context("failed to fetch relay metadata")?;
        if body.trim().is_empty() {
            continue;
        }
        let relay = parse_relay_metadata(&body)
            .with_context(|| format!("failed to parse relay metadata from {}", url))?;
        let item = relay_to_event_item(url, ping_ms, relay, body)?;
        if seen.insert(item.title.clone()) {
            events.push(item);
        }
    }

    events.sort_by(|a, b| a.subtitle.cmp(&b.subtitle));
    Ok(events)
}

fn parse_event_message(message: &str, label: &str) -> Option<EventItem> {
    let value: Value = serde_json::from_str(message).ok()?;
    let array = value.as_array()?;
    if array.first()?.as_str()? != "EVENT" {
        return None;
    }
    let event = array.get(2)?.as_object()?;
    let id = event.get("id")?.as_str()?.to_string();
    let kind = event.get("kind").and_then(|value| value.as_u64()).unwrap_or_default();
    let content = event
        .get("content")
        .and_then(|value| value.as_str())
        .unwrap_or("");
    let pubkey = event
        .get("pubkey")
        .and_then(|value| value.as_str())
        .unwrap_or("");
    let created_at = event
        .get("created_at")
        .and_then(|value| value.as_u64())
        .unwrap_or_default();
    let tags = event
        .get("tags")
        .and_then(|value| value.as_array())
        .map(|value| value.len())
        .unwrap_or(0);
    let raw = serde_json::to_string_pretty(&value).ok()?;
    let subtitle = format!(
        "{} · {} · {} tags",
        shorten(pubkey, 12),
        created_at,
        tags
    );
    Some(EventItem {
        title: format!("{} {}", label, id),
        subtitle,
        source: String::from("nostr relay event"),
        kind_label: format!("kind {}", kind),
        raw,
        detail: shorten(content, 160),
    })
}

fn relay_to_event_item(url: String, ping_ms: u64, relay: Relay, body: String) -> Result<EventItem> {
    let title = relay
        .name
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| url.clone());
    let nips = relay
        .supported_nips
        .clone()
        .unwrap_or_default()
        .into_iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let subtitle = format!("{} ms · nips [{}]", ping_ms, nips);
    let raw = serde_json::to_string_pretty(&relay)
        .or_else(|_| serde_json::to_string_pretty(&serde_json::from_str::<Value>(&body)?))?;
    Ok(EventItem {
        title,
        subtitle,
        source: url,
        kind_label: String::from("relay metadata"),
        raw,
        detail: relay.description.unwrap_or_default(),
    })
}

fn next_focus(focus: Focus) -> Focus {
    match focus {
        Focus::Types => Focus::Events,
        Focus::Events => Focus::Detail,
        Focus::Detail => Focus::Types,
    }
}

fn prev_focus(focus: Focus) -> Focus {
    match focus {
        Focus::Types => Focus::Detail,
        Focus::Events => Focus::Types,
        Focus::Detail => Focus::Events,
    }
}

fn clamp_index(current: usize, len: usize, delta: i32) -> usize {
    if len == 0 {
        return 0;
    }
    let next = if delta.is_negative() {
        current.saturating_sub(delta.unsigned_abs() as usize)
    } else {
        current.saturating_add(delta as usize)
    };
    next.min(len - 1)
}

fn shorten(value: &str, max: usize) -> String {
    let value = value.trim();
    let mut chars = value.chars();
    let prefix = chars.by_ref().take(max).collect::<String>();
    if chars.next().is_some() {
        format!("{}…", prefix)
    } else {
        prefix
    }
}

fn title_case(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}

const ASYNCGIT_TYPES_MOD: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../asyncgit/src/lib/types/mod.rs"
));
const ASYNCGIT_VERSIONED_MOD: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../asyncgit/src/lib/types/versioned/mod.rs"
));
