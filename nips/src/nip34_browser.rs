use std::{
    cmp::Reverse,
    sync::mpsc::{self, Receiver},
    thread,
    time::Duration,
};

use futures::{future::join_all, SinkExt, StreamExt};
use crossterm::event::{KeyCode, KeyEvent};
use gnostr_asyncgit::{
    default_gnostr_private_key,
    types::{
        local_relay_urls, Client, ClientMessage, Event, EventKind, Filter, KeySecurity, Keys,
        Options, PrivateKey, RelayMessage, SubscriptionId, Unixtime,
    },
};
use ratatui::{
    prelude::*,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Tabs, Wrap},
};
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Nip34Tab {
    Patches,
    PullRequest,
    PullRequestUpdate,
    Issue,
    Reply,
    StatusOpen,
    StatusApplied,
    StatusClosed,
    StatusDraft,
    RepoAnnouncement,
    RepoState,
    GraspList,
}

impl Nip34Tab {
    fn title(self) -> &'static str {
        match self {
            Self::Patches => "patches",
            Self::PullRequest => "pr",
            Self::PullRequestUpdate => "pr-upd",
            Self::Issue => "issue",
            Self::Reply => "reply",
            Self::StatusOpen => "open",
            Self::StatusApplied => "applied",
            Self::StatusClosed => "closed",
            Self::StatusDraft => "draft",
            Self::RepoAnnouncement => "repo",
            Self::RepoState => "state",
            Self::GraspList => "grasp",
        }
    }

    fn kind(self) -> EventKind {
        match self {
            Self::Patches => EventKind::Patches,
            Self::PullRequest => EventKind::from(gnostr_asyncgit::types::nip34::PULL_REQUEST_KIND),
            Self::PullRequestUpdate => {
                EventKind::from(gnostr_asyncgit::types::nip34::PULL_REQUEST_UPDATE_KIND)
            }
            Self::Issue => EventKind::from(gnostr_asyncgit::types::nip34::GIT_ISSUE_KIND),
            Self::Reply => EventKind::from(gnostr_asyncgit::types::nip34::GIT_REPLY_KIND),
            Self::StatusOpen => EventKind::GitStatusOpen,
            Self::StatusApplied => EventKind::GitStatusApplied,
            Self::StatusClosed => EventKind::GitStatusClosed,
            Self::StatusDraft => EventKind::GitStatusDraft,
            Self::RepoAnnouncement => {
                EventKind::from(gnostr_asyncgit::types::nip34::REPO_ANNOUNCEMENT_KIND)
            }
            Self::RepoState => EventKind::from(gnostr_asyncgit::types::nip34::REPO_STATE_KIND),
            Self::GraspList => EventKind::from(gnostr_asyncgit::types::nip34::USER_GRASP_LIST_KIND),
        }
    }
}

const ALL_TABS: &[Nip34Tab] = &[
    Nip34Tab::Patches,
    Nip34Tab::PullRequest,
    Nip34Tab::PullRequestUpdate,
    Nip34Tab::Issue,
    Nip34Tab::Reply,
    Nip34Tab::StatusOpen,
    Nip34Tab::StatusApplied,
    Nip34Tab::StatusClosed,
    Nip34Tab::StatusDraft,
    Nip34Tab::RepoAnnouncement,
    Nip34Tab::RepoState,
    Nip34Tab::GraspList,
];

const INITIAL_POLL_INTERVAL: Duration = Duration::from_millis(750);
const RECONNECT_INTERVAL: Duration = Duration::from_secs(2);

#[derive(Clone)]
struct Nip34TabData {
    tab: Nip34Tab,
    events: Vec<Event>,
    selected: usize,
}

impl Nip34TabData {
    fn new(tab: Nip34Tab, events: Vec<Event>) -> Self {
        Self {
            tab,
            events,
            selected: 0,
        }
    }

    fn selected_event(&self) -> Option<&Event> {
        self.events.get(self.selected)
    }

    fn upsert_event(&mut self, event: Event) {
        if self.events.iter().any(|existing| existing.id == event.id) {
            return;
        }

        let selected_id = self.selected_event().map(|event| event.id);
        self.events.push(event);
        self.events.sort_by(|a, b| {
            b.created_at
                .cmp(&a.created_at)
                .then_with(|| b.id.cmp(&a.id))
        });
        self.events.truncate(20);

        if let Some(selected_id) = selected_id {
            self.selected = self
                .events
                .iter()
                .position(|event| event.id == selected_id)
                .unwrap_or(0);
        } else {
            self.selected = 0;
        }
    }
}

enum Nip34Update {
    Loaded(Result<Vec<Nip34TabData>, String>),
    Event(Event),
}

pub struct Nip34Browser {
    receiver: Receiver<Nip34Update>,
    tabs: Vec<Nip34TabData>,
    active_tab: usize,
    loading: bool,
    error: Option<String>,
}

impl Nip34Browser {
    pub fn spawn() -> Self {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let initial = load_nip34_tabs(local_relay_urls(), INITIAL_POLL_INTERVAL);
            if tx.send(Nip34Update::Loaded(initial)).is_err() {
                return;
            }

            let relay_urls = local_relay_urls();
            let runtime = match tokio::runtime::Runtime::new() {
                Ok(runtime) => runtime,
                Err(err) => {
                    let _ = tx.send(Nip34Update::Loaded(Err(err.to_string())));
                    return;
                }
            };

            runtime.block_on(async move {
                let since = Unixtime::now();
                let filters = nip34_subscription_filters(since);
                let mut handles = Vec::new();

                for relay_url in relay_urls {
                    let tx = tx.clone();
                    let filters = filters.clone();
                    let subscription_id = SubscriptionId(format!("nips-nip34-{relay_url}"));
                    handles.push(tokio::spawn(async move {
                        subscribe_nip34_updates(relay_url, subscription_id, filters, tx).await;
                    }));
                }

                let _ = join_all(handles).await;
            });
        });

        Self {
            receiver: rx,
            tabs: ALL_TABS.iter().copied().map(|tab| Nip34TabData::new(tab, Vec::new())).collect(),
            active_tab: 0,
            loading: false,
            error: None,
        }
    }

    pub fn drain(&mut self) {
        while let Ok(update) = self.receiver.try_recv() {
            match update {
                Nip34Update::Loaded(Ok(tabs)) => {
                    self.tabs = tabs;
                    self.loading = false;
                    self.error = None;
                    self.active_tab = self.active_tab.min(self.tabs.len().saturating_sub(1));
                }
                Nip34Update::Loaded(Err(err)) => {
                    self.loading = false;
                    self.error = Some(err);
                }
                Nip34Update::Event(event) => {
                    self.loading = false;
                    self.error = None;
                    for tab in &mut self.tabs {
                        if tab.tab.kind() == event.kind {
                            tab.upsert_event(event.clone());
                        }
                    }
                }
            }
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => return true,
            KeyCode::Left | KeyCode::Char('h') | KeyCode::BackTab => self.previous_tab(),
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Tab => self.next_tab(),
            KeyCode::Up | KeyCode::Char('k') => self.move_selection(false),
            KeyCode::Down | KeyCode::Char('j') => self.move_selection(true),
            KeyCode::Home => self.select_first(),
            KeyCode::End => self.select_last(),
            KeyCode::Char('r') => {
                *self = Self::spawn();
            }
            _ => {}
        }

        false
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(Clear, area);
        frame.render_widget(Block::default().borders(Borders::ALL).title("nip34 browser"), area);

        let inner = area.inner(Margin { vertical: 1, horizontal: 1 });
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(inner);

        if let Some(err) = &self.error {
            let err = Paragraph::new(err.clone())
                .block(Block::default().borders(Borders::ALL).title("Error"))
                .wrap(Wrap { trim: false });
            frame.render_widget(err, chunks[0]);
        } else {
            self.render_events(frame, chunks[0]);
        }

        let bottom = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Length(2)])
            .split(chunks[1]);

        let titles = self
            .tabs
            .iter()
            .map(|tab| Line::from(Span::styled(tab.tab.title(), Style::default())))
            .collect::<Vec<_>>();
        let tabs = Tabs::new(titles)
            .select(self.active_tab)
            .highlight_style(Style::default().fg(Color::Black).bg(Color::LightGreen))
            .style(Style::default().fg(Color::Gray));
        frame.render_widget(tabs, bottom[0]);

        let footer = Paragraph::new(Text::from(vec![
            Line::from(vec![
                Span::styled("Esc", Style::default().fg(Color::Yellow)),
                Span::raw(" close  "),
                Span::styled("←/→", Style::default().fg(Color::Yellow)),
                Span::raw(" tabs  "),
                Span::styled("j/k", Style::default().fg(Color::Yellow)),
                Span::raw(" move  "),
                Span::styled("r", Style::default().fg(Color::Yellow)),
                Span::raw(" reload"),
            ]),
        ]));
        frame.render_widget(footer, bottom[1]);
    }

    fn render_events(&self, frame: &mut Frame, area: Rect) {
        let Some(tab) = self.tabs.get(self.active_tab) else {
            let empty = Paragraph::new("No tab selected");
            frame.render_widget(empty, area);
            return;
        };

        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(38), Constraint::Percentage(62)])
            .split(area);

        let items: Vec<ListItem> = tab
            .events
            .iter()
            .enumerate()
            .map(|(idx, event)| {
                let id = event.id.as_hex_string();
                let short_id = &id[..id.len().min(8)];
                let content = event.content.lines().next().unwrap_or_default().trim();
                let title = format!("{short_id}  {content}");
                let style = if idx == tab.selected {
                    Style::default().fg(Color::Black).bg(Color::LightGreen)
                } else {
                    Style::default()
                };
                ListItem::new(title).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(tab.tab.title()))
            .highlight_symbol(">");
        frame.render_widget(list, body[0]);

        let detail = tab
            .selected_event()
            .map(render_event_detail)
            .unwrap_or_else(|| String::from("No event selected"));
        let detail = Paragraph::new(detail)
            .block(Block::default().borders(Borders::ALL).title("Event"))
            .wrap(Wrap { trim: false });
        frame.render_widget(detail, body[1]);
    }

    fn previous_tab(&mut self) {
        if self.tabs.is_empty() {
            return;
        }
        self.active_tab = if self.active_tab == 0 {
            self.tabs.len().saturating_sub(1)
        } else {
            self.active_tab - 1
        };
    }

    fn next_tab(&mut self) {
        if self.tabs.is_empty() {
            return;
        }
        self.active_tab = (self.active_tab + 1) % self.tabs.len();
    }

    fn move_selection(&mut self, down: bool) {
        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            if tab.events.is_empty() {
                return;
            }
            if down {
                tab.selected = (tab.selected + 1).min(tab.events.len().saturating_sub(1));
            } else {
                tab.selected = tab.selected.saturating_sub(1);
            }
        }
    }

    fn select_first(&mut self) {
        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            tab.selected = 0;
        }
    }

    fn select_last(&mut self) {
        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            if !tab.events.is_empty() {
                tab.selected = tab.events.len() - 1;
            }
        }
    }
}

fn render_event_detail(event: &Event) -> String {
    let tags = if event.tags.is_empty() {
        String::from("(no tags)")
    } else {
        event
            .tags
            .iter()
            .map(|tag| tag.0.join(":"))
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        "kind: {:?}\nid: {}\ncreated_at: {}\npubkey: {}\n\ncontent:\n{}\n\ntags:\n{}",
        event.kind,
        event.id.as_hex_string(),
        event.created_at,
        event.pubkey.as_hex_string(),
        event.content,
        tags
    )
}

fn load_nip34_tabs(relay_urls: Vec<String>, timeout: Duration) -> Result<Vec<Nip34TabData>, String> {
    let runtime = tokio::runtime::Runtime::new().map_err(|err| err.to_string())?;
    runtime.block_on(async {
        if relay_urls.is_empty() {
            return Err(String::from("no local relay endpoint configured"));
        }

        let keys = Keys::new(PrivateKey(
            default_gnostr_private_key(),
            KeySecurity::Weak,
        ));
        let mut client = Client::new(&keys, Options::new());
        client
            .add_relays(relay_urls)
            .await
            .map_err(|err| err.to_string())?;
        client.connect().await;

        let tabs = join_all(ALL_TABS.iter().copied().map(|tab| {
            let client = client.clone();
            async move {
                let mut filter = Filter::new();
                filter.add_event_kind(tab.kind());
                filter.limit = Some(20);
                let mut events = client
                    .get_events_of(vec![filter], Some(timeout))
                    .await
                    .map_err(|err| err.to_string())?;
                events.sort_by_key(|event| Reverse(event.created_at));
                Ok::<Nip34TabData, String>(Nip34TabData::new(tab, events))
            }
        }))
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;

        Ok(tabs)
    })
}

fn nip34_subscription_filters(since: Unixtime) -> Vec<Filter> {
    let mut filter = Filter::new();
    for tab in ALL_TABS {
        filter.add_event_kind(tab.kind());
    }
    filter.since = Some(since);
    vec![filter]
}

async fn subscribe_nip34_updates(
    relay_url: String,
    subscription_id: SubscriptionId,
    filters: Vec<Filter>,
    tx: std::sync::mpsc::Sender<Nip34Update>,
) {
    loop {
        match connect_async(&relay_url).await {
            Ok((stream, _response)) => {
                let (mut write, mut read) = stream.split();
                let request = ClientMessage::Req(subscription_id.clone(), filters.clone());
                let request = match serde_json::to_string(&request) {
                    Ok(request) => request,
                    Err(_) => return,
                };

                if let Err(_err) = write.send(WsMessage::Text(request.into())).await {
                    tokio::time::sleep(RECONNECT_INTERVAL).await;
                    continue;
                }

                while let Some(message) = read.next().await {
                    match message {
                        Ok(WsMessage::Text(text)) => match serde_json::from_str::<RelayMessage>(&text)
                        {
                            Ok(RelayMessage::Event(_, event)) => {
                                if tx.send(Nip34Update::Event(*event)).is_err() {
                                    return;
                                }
                            }
                            Ok(RelayMessage::Notice(_))
                            | Ok(RelayMessage::Notify(_))
                            | Ok(RelayMessage::Eose(_))
                            | Ok(RelayMessage::Ok(_, _, _))
                            | Ok(RelayMessage::Auth(_))
                            | Ok(RelayMessage::Closed(_, _)) => {}
                            Err(_) => {}
                        },
                        Ok(WsMessage::Ping(payload)) => {
                            if write.send(WsMessage::Pong(payload)).await.is_err() {
                                break;
                            }
                        }
                        Ok(WsMessage::Close(_)) => break,
                        Ok(WsMessage::Binary(_))
                        | Ok(WsMessage::Pong(_))
                        | Ok(WsMessage::Frame(_)) => {}
                        Err(_) => break,
                    }
                }
            }
            Err(_) => {}
        }

        tokio::time::sleep(RECONNECT_INTERVAL).await;
    }
}
