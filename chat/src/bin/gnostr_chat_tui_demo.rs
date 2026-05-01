//! Chat TUI demo for the asyncgit Nostr widgets.
//!
//! The screen layout mirrors the JS app's main sections: home, relays, settings,
//! NIP explorer, NIP-34, and search.

use std::{
    collections::{BTreeMap, HashMap},
    fs,
    io,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
    time::Duration,
};

use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use gnostr_asyncgit::{
    tui::nostr::widgets::*,
    types::{
        ChannelCreationEvent, ChannelMessageEvent, EventKind, EventReference, EventV3, Filter,
        Id, IdHex, Metadata, NAddr, Nip19, Nip19Profile, Profile, PublicKey, PublicKeyHex,
        RelayInformationDocument, RelayList, RelayListUsage, RelayMessage, RelayUrl, RepoRef,
        RepoState, Signature, Tag, Unixtime, UncheckedUrl, Url, RelayUsageSet,
    },
};
use gnostr_crawler::Relay as CrawlerRelay;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};
use sha2::{Digest, Sha256};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut terminal = TerminalGuard::enter()?;
    let mut app = App::new().await?;

    loop {
        terminal.terminal.draw(|frame| app.draw(frame))?;

        if event::poll(Duration::from_millis(250))? {
            match event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Down | KeyCode::Char('j') => app.next(),
                    KeyCode::Up | KeyCode::Char('k') => app.previous(),
                    KeyCode::Left | KeyCode::Char('h') => app.previous(),
                    KeyCode::Right | KeyCode::Char('l') => app.next(),
                    KeyCode::Home => app.select(0),
                    KeyCode::End => app.select(app.pages.len().saturating_sub(1)),
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
    fn enter() -> anyhow::Result<Self> {
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

struct App {
    pages: Vec<Page>,
    selected: usize,
    data: Arc<RwLock<DemoData>>,
}

impl App {
    async fn new() -> anyhow::Result<Self> {
        let data = Arc::new(RwLock::new(DemoData::load_real()?));
        DemoData::spawn_refresh(Arc::clone(&data));
        Ok(Self {
            pages: vec![
                Page::Overview,
                Page::Relays,
                Page::Settings,
                Page::Explorer,
                Page::Nip34,
                Page::Search,
            ],
            selected: 0,
            data,
        })
    }

    fn next(&mut self) {
        self.selected = (self.selected + 1) % self.pages.len();
    }

    fn previous(&mut self) {
        self.selected = if self.selected == 0 {
            self.pages.len() - 1
        } else {
            self.selected - 1
        };
    }

    fn select(&mut self, idx: usize) {
        self.selected = idx.min(self.pages.len().saturating_sub(1));
    }

    fn draw(&self, frame: &mut Frame) {
        let data = self.data.read().expect("demo state poisoned");
        let root = frame.area();
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(2)])
            .split(root);

        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("gnostr ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                Span::raw("chat TUI demo "),
                Span::styled("q", Style::default().fg(Color::Yellow)),
                Span::raw(" quit  "),
                Span::styled("j/k", Style::default().fg(Color::Yellow)),
                Span::raw(" move  "),
                Span::raw("mirror of the JS app"),
            ]))
            .block(Block::default().title("gnostr").borders(Borders::ALL)),
            layout[0],
        );

        let content = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(28), Constraint::Min(0)])
            .split(layout[1]);

        self.draw_nav(frame, content[0]);
        self.draw_page(frame, content[1], self.pages[self.selected], &data);

        frame.render_widget(
            Paragraph::new(self.status_line(&data))
                .wrap(Wrap { trim: true })
                .block(Block::default().title("status").borders(Borders::ALL)),
            layout[2],
        );
    }

    fn draw_nav(&self, frame: &mut Frame, area: Rect) {
        let items = self
            .pages
            .iter()
            .enumerate()
            .map(|(idx, page)| {
                let style = if idx == self.selected {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Magenta)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(page.title()).style(style)
            })
            .collect::<Vec<_>>();

        frame.render_widget(
            List::new(items).block(Block::default().title("views").borders(Borders::ALL)),
            area,
        );
    }

    fn draw_page(&self, frame: &mut Frame, area: Rect, page: Page, data: &DemoData) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(9), Constraint::Min(0)])
            .split(area);

        frame.render_widget(
            Paragraph::new(page.summary(data))
                .wrap(Wrap { trim: true })
                .block(Block::default().title(page.title()).borders(Borders::ALL)),
            chunks[0],
        );

        page.render(frame, chunks[1], data);
    }

    fn status_line(&self, data: &DemoData) -> String {
        let page = self.pages[self.selected].title();
        format!(
            "selected view: {page} | crawler relays: {} | relay entries: {} | updated: {}",
            data.crawler_relays.len(),
            data.relay_list.0.len(),
            data.updated_at
        )
    }
}

#[derive(Clone, Copy)]
enum Page {
    Overview,
    Relays,
    Settings,
    Explorer,
    Nip34,
    Search,
}

impl Page {
    fn title(self) -> &'static str {
        match self {
            Page::Overview => "home",
            Page::Relays => "relays",
            Page::Settings => "settings",
            Page::Explorer => "nip explorer",
            Page::Nip34 => "nip34",
            Page::Search => "search",
        }
    }

    fn summary(self, data: &DemoData) -> String {
        match self {
            Page::Overview => format!(
                "The home view shows the core profile/event shapes that the JS app keeps in its timeline and header panels.\n\nProfile: {}\nEvent id: {}",
                data.profile.pubkey, data.event.id
            ),
            Page::Relays => format!(
                "Relay views combine crawler discovery data and relay metadata.\n\nCrawler discovery source: {} entries\nRelay list entries: {}",
                data.crawler_relays.len(),
                data.relay_list.0.len()
            ),
            Page::Settings => format!(
                "Settings mirrors the JS profile and relay configuration panes.\n\nDisplay name: {}\nRelay usage bits: {}",
                data.metadata.name.as_deref().unwrap_or("unknown"),
                data.relay_usage.bits()
            ),
            Page::Explorer => format!(
                "NIP explorer surfaces bech32 and tag/reference shapes used across the app.\n\nNIP-19 forms: profile, event, address\nReference marker: {:?}",
                data.event_reference
            ),
            Page::Nip34 => format!(
                "NIP-34 mirrors the repository and state panels used by the web app.\n\nRepository identifier: {}\nState refs: {}",
                data.repo_ref.identifier,
                data.repo_state.state.len()
            ),
            Page::Search => format!(
                "Search mirrors the filtered query view in the JS app.\n\nKinds: {:?}\nTags: {}",
                data.filter.kinds,
                data.filter.tags.len()
            ),
        }
    }

    fn render(self, frame: &mut Frame, area: Rect, data: &DemoData) {
        match self {
            Page::Overview => {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(area);
                frame.render_widget(
                    EventV3Widget::new(&data.event)
                        .block(Block::default().title("event").borders(Borders::ALL)),
                    chunks[0],
                );
                frame.render_widget(
                    MetadataWidget::new(&data.metadata)
                        .block(Block::default().title("metadata").borders(Borders::ALL)),
                    chunks[1],
                );
            }
            Page::Relays => {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(area);
                frame.render_widget(
                    RelayInformationDocumentWidget::new(&data.relay_document)
                        .block(Block::default().title("relay document").borders(Borders::ALL)),
                    chunks[0],
                );
                let right = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(chunks[1]);
                frame.render_widget(
                    RelayListWidget::new(&data.relay_list)
                        .block(Block::default().title("relay list").borders(Borders::ALL)),
                    right[0],
                );
                frame.render_widget(
                    Paragraph::new(data.crawler_relays_text())
                        .wrap(Wrap { trim: true })
                        .block(Block::default().title("crawler relay discovery").borders(Borders::ALL)),
                    right[1],
                );
            }
            Page::Settings => {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(area);
                frame.render_widget(
                    ProfileWidget::new(&data.profile)
                        .block(Block::default().title("profile").borders(Borders::ALL)),
                    chunks[0],
                );
                let right = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(chunks[1]);
                frame.render_widget(
                    RelayUsageSetWidget::new(&data.relay_usage)
                        .block(Block::default().title("relay usage").borders(Borders::ALL)),
                    right[0],
                );
                frame.render_widget(
                    PublicKeyWidget::new(&data.public_key)
                        .block(Block::default().title("pubkey").borders(Borders::ALL)),
                    right[1],
                );
            }
            Page::Explorer => {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(area);
                frame.render_widget(
                    Nip19Widget::new(&data.nip19)
                        .block(Block::default().title("nip19").borders(Borders::ALL)),
                    chunks[0],
                );
                let right = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Percentage(33),
                        Constraint::Percentage(33),
                        Constraint::Percentage(34),
                    ])
                    .split(chunks[1]);
                frame.render_widget(
                    NAddrWidget::new(&data.naddr)
                        .block(Block::default().title("naddr").borders(Borders::ALL)),
                    right[0],
                );
                frame.render_widget(
                    EventReferenceWidget::new(&data.event_reference)
                        .block(Block::default().title("event reference").borders(Borders::ALL)),
                    right[1],
                );
                frame.render_widget(
                    TagWidget::new(&data.tag)
                        .block(Block::default().title("tag").borders(Borders::ALL)),
                    right[2],
                );
            }
            Page::Nip34 => {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(area);
                frame.render_widget(
                    RepoRefWidget::new(&data.repo_ref)
                        .block(Block::default().title("repo ref").borders(Borders::ALL)),
                    chunks[0],
                );
                let right = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Percentage(34),
                        Constraint::Percentage(33),
                        Constraint::Percentage(33),
                    ])
                    .split(chunks[1]);
                frame.render_widget(
                    RepoStateWidget::new(&data.repo_state)
                        .block(Block::default().title("repo state").borders(Borders::ALL)),
                    right[0],
                );
                frame.render_widget(
                    ChannelCreationEventWidget::new(&data.channel_creation)
                        .block(Block::default().title("channel creation").borders(Borders::ALL)),
                    right[1],
                );
                frame.render_widget(
                    ChannelMessageEventWidget::new(&data.channel_message)
                        .block(Block::default().title("channel message").borders(Borders::ALL)),
                    right[2],
                );
            }
            Page::Search => {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(area);
                frame.render_widget(
                    FilterWidget::new(&data.filter)
                        .block(Block::default().title("filter").borders(Borders::ALL)),
                    chunks[0],
                );
                let right = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Percentage(34),
                        Constraint::Percentage(33),
                        Constraint::Percentage(33),
                    ])
                    .split(chunks[1]);
                frame.render_widget(
                    Paragraph::new(data.crawler_relays_text())
                        .wrap(Wrap { trim: true })
                        .block(Block::default().title("crawler relay").borders(Borders::ALL)),
                    right[0],
                );
                frame.render_widget(
                    RelayMessageWidget::new(&data.relay_message)
                        .block(Block::default().title("relay message").borders(Borders::ALL)),
                    right[1],
                );
                frame.render_widget(
                    TagWidget::new(&data.tag)
                        .block(Block::default().title("tag").borders(Borders::ALL)),
                    right[2],
                );
            }
        }
    }
}

struct DemoData {
    public_key: PublicKey,
    profile: Profile,
    metadata: Metadata,
    event: EventV3,
    relay_document: RelayInformationDocument,
    relay_list: RelayList,
    relay_usage: RelayUsageSet,
    nip19: Nip19,
    naddr: NAddr,
    event_reference: EventReference,
    tag: Tag,
    repo_ref: RepoRef,
    repo_state: RepoState,
    channel_creation: ChannelCreationEvent,
    channel_message: ChannelMessageEvent,
    relay_message: RelayMessage,
    crawler_relays: Vec<LoadedRelay>,
    filter: Filter,
    updated_at: Unixtime,
}

#[derive(Debug)]
struct LoadedRelay {
    url: String,
    relay: CrawlerRelay,
    pubkey_hex: Option<String>,
}

impl DemoData {
    fn load_real() -> Result<Self> {
        let config_dir = gnostr_crawler::relays::get_config_dir_path();
        let crawler_relays = load_crawler_relays(&config_dir)?;
        anyhow::ensure!(
            !crawler_relays.is_empty(),
            "no crawler relay metadata found in {}",
            config_dir.display()
        );

        let relay_urls = load_relay_urls(&config_dir, &crawler_relays)?;
        anyhow::ensure!(!relay_urls.is_empty(), "no relay URLs available for demo");

        let primary = crawler_relays
            .first()
            .context("missing primary crawler relay")?;
        let primary_pubkey = relay_pubkey(&primary)?;
        let primary_url = relay_urls
            .first()
            .cloned()
            .context("missing primary relay URL")?;
        let relay_url = RelayUrl::try_from_str(&primary_url)?;
        let unchecked_relay = UncheckedUrl::from_str(&primary_url);
        let relay_list = relay_list_from_urls(&relay_urls)?;
        let relay_document = relay_document_from_relay(primary, primary_pubkey)?;
        let profile = Profile {
            pubkey: primary_pubkey,
            relays: relay_urls
                .iter()
                .map(|url| UncheckedUrl::from_str(url))
                .collect(),
        };
        let metadata = metadata_from_relay(primary);
        let relay_usage = RelayUsageSet::new_empty();
        let nip19 = Nip19::Profile(Nip19Profile {
            public_key: primary_pubkey,
            relays: vec![relay_url.clone()],
        });

        let git = load_git_snapshot(primary_pubkey, &relay_url, &unchecked_relay)?;
        let naddr = NAddr {
            d: git.identifier.clone(),
            relays: vec![unchecked_relay.clone()],
            kind: EventKind::LongFormContent,
            author: primary_pubkey,
        };
        let event_reference = EventReference::Id {
            id: git.state_id,
            author: Some(primary_pubkey),
            relays: vec![relay_url.clone()],
            marker: Some("root".to_string()),
        };
        let tag = Tag::new_event(git.state_id, Some(unchecked_relay.clone()), Some("root".to_string()));

        let mut repo_events = HashMap::new();
        repo_events.insert(git.coordinate.clone(), git.state_event.clone());
        let repo_ref = RepoRef {
            name: git.repo_name.clone(),
            description: git.description.clone(),
            identifier: git.identifier.clone(),
            root_commit: git.root_commit.clone(),
            git_server: git.git_servers.clone(),
            web: git.web.clone(),
            relays: vec![unchecked_relay.clone()],
            hashtags: vec![git.repo_name.clone(), "crawler".to_string()],
            maintainers: vec![primary_pubkey],
            trusted_maintainer: primary_pubkey,
            events: repo_events,
        };
        let repo_state = RepoState {
            identifier: git.identifier.clone(),
            state: git.state.clone(),
            event: git.state_event.clone(),
        };
        let channel_creation = ChannelCreationEvent {
            channel_id: git.identifier.clone(),
            channel_name: Some(git.repo_name.clone()),
            channel_description: Some(git.description.clone()),
            channel_picture: None,
            relay_url: Some(unchecked_relay.clone()),
            pubkey: primary_pubkey,
        };
        let channel_message = ChannelMessageEvent {
            channel_id: git.identifier.clone(),
            message: format!(
                "{} crawler relays loaded from {}",
                crawler_relays.len(),
                config_dir.display()
            ),
            reply_to: Some(git.state_id),
            root_message: Some(git.state_id),
            pubkey: primary_pubkey,
            relay_url: Some(unchecked_relay.clone()),
        };
        let relay_message = RelayMessage::Notice(format!(
            "loaded {} relay records from {}",
            crawler_relays.len(),
            config_dir.display()
        ));
        let mut filter = Filter::new();
        let id_hex = IdHex::from(git.state_id);
        filter.add_id(&id_hex);
        filter.add_author(&PublicKeyHex::from(primary_pubkey));
        filter.add_event_kind(EventKind::GitRepoAnnouncement);
        filter.add_event_kind(EventKind::TextNote);
        filter.add_tag_value('e', git.state_id.as_hex_string());
        filter.add_tag_value('p', primary_pubkey.as_hex_string());
        filter.since = Some(Unixtime::now());
        filter.until = Some(Unixtime::now());
        filter.limit = Some(20);

        Ok(Self {
            public_key: primary_pubkey,
            profile,
            metadata,
            event: git.state_event.clone(),
            relay_document,
            relay_list,
            relay_usage,
            nip19,
            naddr,
            event_reference,
            tag,
            repo_ref,
            repo_state,
            channel_creation,
            channel_message,
            relay_message,
            crawler_relays,
            filter,
            updated_at: Unixtime::now(),
        })
    }

    fn spawn_refresh(data: Arc<RwLock<Self>>) {
        tokio::spawn(async move {
            let client = reqwest::Client::new();
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                if let Err(err) = Self::refresh_from_network(&client, &data).await {
                    if let Ok(mut state) = data.write() {
                        state.relay_message = RelayMessage::Notice(format!(
                            "network refresh failed: {err}"
                        ));
                        state.updated_at = Unixtime::now();
                    }
                }
            }
        });
    }

    async fn refresh_from_network(
        client: &reqwest::Client,
        data: &Arc<RwLock<Self>>,
    ) -> Result<()> {
        let relay_urls = {
            let state = data.read().expect("demo state poisoned");
            state
                .relay_list
                .0
                .keys()
                .map(|relay| relay.to_string())
                .collect::<Vec<_>>()
        };

        if relay_urls.is_empty() {
            return Ok(());
        }

        let bodies = gnostr_crawler::fetch_relay_texts(relay_urls, client, "chat demo").await;
        let mut refreshed_relays: Vec<LoadedRelay> = Vec::new();
        for item in bodies {
            let (url, json_string, ping_ms) = match item {
                Ok(tuple) => tuple,
                Err(err) => {
                    if let Ok(mut state) = data.write() {
                        state.relay_message = RelayMessage::Notice(format!(
                            "relay refresh request failed: {err}"
                        ));
                    }
                    continue;
                }
            };

            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&json_string) {
                let pubkey_hex = value
                    .get("pubkey")
                    .and_then(|value| value.as_str())
                    .map(ToString::to_string);
                if let Ok(mut relay) = serde_json::from_value::<CrawlerRelay>(value) {
                    relay.ping_ms = Some(ping_ms);
                    refreshed_relays.push(LoadedRelay {
                        url: url.clone(),
                        relay,
                        pubkey_hex,
                    });
                }
            } else if let Ok(mut relay) = gnostr_crawler::parse_relay_metadata(&json_string) {
                relay.ping_ms = Some(ping_ms);
                refreshed_relays.push(LoadedRelay {
                    url: url.clone(),
                    relay,
                    pubkey_hex: None,
                });
            }
            if let Ok(mut state) = data.write() {
                state.relay_message = RelayMessage::Notice(format!(
                    "refreshed {} from the network",
                    url
                ));
                state.updated_at = Unixtime::now();
            }
        }

        refreshed_relays.sort_by(|a, b| {
            b.relay
                .supported_nips
                .as_ref()
                .map(|v| v.len())
                .unwrap_or(0)
                .cmp(&a.relay.supported_nips.as_ref().map(|v| v.len()).unwrap_or(0))
                .then_with(|| a.relay.name.cmp(&b.relay.name))
        });

        if let Some(primary) = refreshed_relays.first() {
            let primary_pubkey = relay_pubkey(primary)?;
            let relay_urls = {
                let state = data.read().expect("demo state poisoned");
                state
                    .relay_list
                    .0
                    .keys()
                    .map(|relay| relay.to_string())
                    .collect::<Vec<_>>()
            };
            let primary_url = relay_urls
                .first()
                .cloned()
                .context("missing primary relay URL")?;
            let relay_url = RelayUrl::try_from_str(&primary_url)?;
            let unchecked_relay = UncheckedUrl::from_str(&primary_url);
            let relay_document = relay_document_from_relay(primary, primary_pubkey)?;
            let profile = Profile {
                pubkey: primary_pubkey,
                relays: relay_urls
                    .iter()
                    .map(|url| UncheckedUrl::from_str(url))
                    .collect(),
            };
            let metadata = metadata_from_relay(primary);

            let git = load_git_snapshot(primary_pubkey, &relay_url, &unchecked_relay)?;
            let nip19 = Nip19::Profile(Nip19Profile {
                public_key: primary_pubkey,
                relays: vec![relay_url.clone()],
            });
            let naddr = NAddr {
                d: git.identifier.clone(),
                relays: vec![unchecked_relay.clone()],
                kind: EventKind::LongFormContent,
                author: primary_pubkey,
            };
            let event_reference = EventReference::Id {
                id: git.state_id,
                author: Some(primary_pubkey),
                relays: vec![relay_url.clone()],
                marker: Some("root".to_string()),
            };
            let tag = Tag::new_event(git.state_id, Some(unchecked_relay.clone()), Some("root".to_string()));
            let mut repo_events = HashMap::new();
            repo_events.insert(git.coordinate.clone(), git.state_event.clone());
            let repo_ref = RepoRef {
                name: git.repo_name.clone(),
                description: git.description.clone(),
                identifier: git.identifier.clone(),
                root_commit: git.root_commit.clone(),
                git_server: git.git_servers.clone(),
                web: git.web.clone(),
                relays: vec![unchecked_relay.clone()],
                hashtags: vec![git.repo_name.clone(), "crawler".to_string()],
                maintainers: vec![primary_pubkey],
                trusted_maintainer: primary_pubkey,
                events: repo_events,
            };
            let repo_state = RepoState {
                identifier: git.identifier.clone(),
                state: git.state.clone(),
                event: git.state_event.clone(),
            };
            let channel_creation = ChannelCreationEvent {
                channel_id: git.identifier.clone(),
                channel_name: Some(git.repo_name.clone()),
                channel_description: Some(git.description.clone()),
                channel_picture: None,
                relay_url: Some(unchecked_relay.clone()),
                pubkey: primary_pubkey,
            };
            let channel_message = ChannelMessageEvent {
                channel_id: git.identifier.clone(),
                message: format!(
                    "{} crawler relays refreshed from the network",
                    refreshed_relays.len()
                ),
                reply_to: Some(git.state_id),
                root_message: Some(git.state_id),
                pubkey: primary_pubkey,
                relay_url: Some(unchecked_relay.clone()),
            };
            let mut filter = Filter::new();
            let id_hex = IdHex::from(git.state_id);
            filter.add_id(&id_hex);
            filter.add_author(&PublicKeyHex::from(primary_pubkey));
            filter.add_event_kind(EventKind::GitRepoAnnouncement);
            filter.add_event_kind(EventKind::TextNote);
            filter.add_tag_value('e', git.state_id.as_hex_string());
            filter.add_tag_value('p', primary_pubkey.as_hex_string());
            filter.since = Some(Unixtime::now());
            filter.until = Some(Unixtime::now());
            filter.limit = Some(20);

            let mut state = data.write().expect("demo state poisoned");
            state.public_key = primary_pubkey;
            state.profile = profile;
            state.metadata = metadata;
            state.event = git.state_event.clone();
            state.relay_document = relay_document;
            state.nip19 = nip19;
            state.naddr = naddr;
            state.event_reference = event_reference;
            state.tag = tag;
            state.repo_ref = repo_ref;
            state.repo_state = repo_state;
            state.channel_creation = channel_creation;
            state.channel_message = channel_message;
            state.relay_message = RelayMessage::Notice(format!(
                "refreshed {} relay records from the network",
                refreshed_relays.len()
            ));
            state.crawler_relays = refreshed_relays;
            state.filter = filter;
            state.updated_at = Unixtime::now();
        }

        Ok(())
    }

    fn crawler_relays_text(&self) -> String {
        self.crawler_relays
            .iter()
            .enumerate()
            .map(|(idx, relay)| {
                format!(
                    "{}. {} | ping={}ms | nips={:?} | {}",
                    idx + 1,
                    relay.relay.name.as_deref().unwrap_or("unknown"),
                    relay.relay.ping_ms.unwrap_or_default(),
                    relay.relay.supported_nips.clone().unwrap_or_default(),
                    relay.relay.description.as_deref().unwrap_or("")
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

struct GitSnapshot {
    repo_name: String,
    description: String,
    identifier: String,
    root_commit: String,
    git_servers: Vec<String>,
    web: Vec<String>,
    state: HashMap<String, String>,
    state_event: EventV3,
    state_id: Id,
    coordinate: NAddr,
}

fn load_crawler_relays(config_dir: &Path) -> Result<Vec<LoadedRelay>> {
    let mut discovered: BTreeMap<String, LoadedRelay> = BTreeMap::new();

    for nip_entry in fs::read_dir(config_dir).with_context(|| {
        format!("reading crawler relay cache directory {}", config_dir.display())
    })? {
        let nip_entry = nip_entry?;
        if !nip_entry.file_type()?.is_dir() {
            continue;
        }

        let nip_name = nip_entry.file_name().to_string_lossy().to_string();
        if nip_name.parse::<i32>().is_err() {
            continue;
        }

        for relay_entry in fs::read_dir(nip_entry.path())? {
            let relay_entry = relay_entry?;
            let file_name = relay_entry.file_name().to_string_lossy().to_string();
            if !file_name.ends_with(".json") || file_name == "relays.json" {
                continue;
            }

            let content = fs::read_to_string(relay_entry.path())?;
            let value: serde_json::Value = serde_json::from_str(&content)
                .with_context(|| format!("parsing {}", relay_entry.path().display()))?;
            let pubkey_hex = value
                .get("pubkey")
                .and_then(|value| value.as_str())
                .map(ToString::to_string);
            let relay: CrawlerRelay = serde_json::from_value(value)
                .with_context(|| format!("parsing {}", relay_entry.path().display()))?;
            let host = file_name.trim_end_matches(".json");
            let url = format!("wss://{}", host);
            match discovered.entry(url.clone()) {
                std::collections::btree_map::Entry::Occupied(mut entry) => {
                    merge_loaded_relay(entry.get_mut(), relay, pubkey_hex);
                }
                std::collections::btree_map::Entry::Vacant(entry) => {
                    entry.insert(LoadedRelay {
                        url,
                        relay,
                        pubkey_hex,
                    });
                }
            }
        }
    }

    let mut relays: Vec<LoadedRelay> = discovered.into_values().collect();
    relays.sort_by(|a, b| {
        b.relay
            .supported_nips
            .as_ref()
            .map(|v| v.len())
            .unwrap_or(0)
            .cmp(&a.relay.supported_nips.as_ref().map(|v| v.len()).unwrap_or(0))
            .then_with(|| a.relay.name.cmp(&b.relay.name))
    });
    Ok(relays)
}

fn merge_loaded_relay(existing: &mut LoadedRelay, incoming: CrawlerRelay, incoming_pubkey: Option<String>) {
    if existing.pubkey_hex.is_none() {
        existing.pubkey_hex = incoming_pubkey;
    }
    merge_relay(&mut existing.relay, incoming);
}

fn merge_relay(existing: &mut CrawlerRelay, incoming: CrawlerRelay) {
    if existing.contact.is_none() {
        existing.contact = incoming.contact;
    }
    if existing.description.is_none() {
        existing.description = incoming.description;
    }
    if existing.name.is_none() {
        existing.name = incoming.name;
    }
    if existing.ping_ms.is_none() {
        existing.ping_ms = incoming.ping_ms;
    }
    if existing.software.is_none() {
        existing.software = incoming.software;
    }
    if existing.version.is_none() {
        existing.version = incoming.version;
    }
    existing.supported_nips = match (existing.supported_nips.take(), incoming.supported_nips) {
        (Some(mut left), Some(mut right)) => {
            left.append(&mut right);
            left.sort();
            left.dedup();
            Some(left)
        }
        (Some(left), None) => Some(left),
        (None, Some(right)) => Some(right),
        (None, None) => None,
    };
    existing.supported_nip_extensions = match (
        existing.supported_nip_extensions.take(),
        incoming.supported_nip_extensions,
    ) {
        (Some(mut left), Some(mut right)) => {
            left.append(&mut right);
            left.sort();
            left.dedup();
            Some(left)
        }
        (Some(left), None) => Some(left),
        (None, Some(right)) => Some(right),
        (None, None) => None,
    };
}

fn load_relay_urls(config_dir: &Path, relays: &[LoadedRelay]) -> Result<Vec<String>> {
    let relays_yaml = config_dir.join("relays.yaml");
    if let Ok(content) = fs::read_to_string(&relays_yaml) {
        let relays: Vec<String> = serde_yaml::from_str(&content)
            .with_context(|| format!("parsing {}", relays_yaml.display()))?;
        if !relays.is_empty() {
            return Ok(relays);
        }
    }

    let mut urls = Vec::new();
    for nip_entry in fs::read_dir(config_dir)
        .with_context(|| format!("reading {}", config_dir.display()))?
    {
        let nip_entry = nip_entry?;
        if !nip_entry.file_type()?.is_dir() {
            continue;
        }

        let nip_name = nip_entry.file_name().to_string_lossy().to_string();
        if nip_name.parse::<i32>().is_err() {
            continue;
        }

        for relay_entry in fs::read_dir(nip_entry.path())? {
            let relay_entry = relay_entry?;
            let file_name = relay_entry.file_name().to_string_lossy().to_string();
            if !file_name.ends_with(".json") || file_name == "relays.json" {
                continue;
            }
            urls.push(format!("wss://{}", file_name.trim_end_matches(".json")));
        }
    }

    if urls.is_empty() {
        for relay in relays {
            urls.push(relay.url.clone());
        }
    }

    urls.sort();
    urls.dedup();
    Ok(urls)
}

fn relay_pubkey(relay: &LoadedRelay) -> Result<PublicKey> {
    let pubkey = relay
        .pubkey_hex
        .as_deref()
        .context("relay metadata missing pubkey")?;
    PublicKey::try_from_hex_string(pubkey, true)
        .with_context(|| format!("parsing relay pubkey {pubkey}"))
}

fn relay_document_from_relay(relay: &LoadedRelay, pubkey: PublicKey) -> Result<RelayInformationDocument> {
    Ok(RelayInformationDocument {
        name: relay.relay.name.clone(),
        description: relay.relay.description.clone(),
        banner: None,
        icon: None,
        pubkey: Some(pubkey.into()),
        self_pubkey: Some(pubkey.into()),
        contact: relay.relay.contact.clone(),
        supported_nips: relay
            .relay
            .supported_nips
            .clone()
            .unwrap_or_default()
            .into_iter()
            .map(|nip| nip as u32)
            .collect(),
        software: relay.relay.software.clone(),
        version: relay.relay.version.clone(),
        limitation: None,
        retention: vec![],
        relay_countries: vec![],
        language_tags: vec![],
        tags: vec![],
        posting_policy: None,
        payments_url: None,
        fees: None,
        other: serde_json::Map::new(),
    })
}

fn metadata_from_relay(relay: &LoadedRelay) -> Metadata {
    let mut other = serde_json::Map::new();
    if let Some(software) = relay.relay.software.clone() {
        other.insert("software".to_string(), serde_json::Value::String(software));
    }
    if let Some(version) = relay.relay.version.clone() {
        other.insert("version".to_string(), serde_json::Value::String(version));
    }
    if let Some(nips) = relay.relay.supported_nips.clone() {
        other.insert(
            "supported_nips".to_string(),
            serde_json::Value::Array(nips.into_iter().map(|nip| serde_json::Value::from(nip)).collect()),
        );
    }
    Metadata {
        name: relay.relay.name.clone(),
        about: relay.relay.description.clone(),
        picture: None,
        nip05: relay.relay.contact.clone(),
        other,
    }
}

fn relay_list_from_urls(urls: &[String]) -> Result<RelayList> {
    let mut relay_list = RelayList::default();
    for url in urls {
        relay_list
            .0
            .insert(RelayUrl::try_from_str(url)?, RelayListUsage::Both);
    }
    Ok(relay_list)
}

fn load_git_snapshot(
    public_key: PublicKey,
    relay_url: &RelayUrl,
    unchecked_relay: &UncheckedUrl,
) -> Result<GitSnapshot> {
    let repo = git2::Repository::discover(".").context("discovering git repository")?;
    let workdir = repo
        .workdir()
        .context("missing git workdir")?;
    let repo_name = workdir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("gnostr")
        .to_string();
    let head = repo.head().context("reading git HEAD")?;
    let head_commit = head.peel_to_commit().context("reading HEAD commit")?;
    let root_commit = head_commit.id().to_string();
    let mut state = HashMap::new();

    if let Some(head_name) = head.name() {
        state.insert("HEAD".to_string(), format!("ref: {head_name}"));
    } else {
        state.insert("HEAD".to_string(), root_commit.clone());
    }

    for reference in repo.references().context("listing git references")? {
        let reference = reference?;
        let Some(name) = reference.name() else {
            continue;
        };
        if !(name.starts_with("refs/heads/") || name.starts_with("refs/tags/")) {
            continue;
        }
        if let Some(target) = reference.target() {
            state.insert(name.to_string(), target.to_string());
        }
    }

    let description = format!("{} on {}", repo_name, head_commit.id());
    let identifier = repo_name.clone();
    let git_servers = repo
        .find_remote("origin")
        .ok()
        .and_then(|remote| remote.url().map(|url| vec![url.to_string()]))
        .unwrap_or_default();
    let web = git_servers.clone();
    let state_serialized = serde_json::to_string(&state)?;
    let state_hash = Sha256::digest(state_serialized.as_bytes());
    let state_id = Id::try_from_bytes(&state_hash)?;
    let created_at = Unixtime(head_commit.time().seconds());
    let tags = {
        let mut tags = vec![Tag::new(&["d", &identifier])];
        let mut keys: Vec<_> = state.keys().cloned().collect();
        keys.sort();
        for key in keys {
            tags.push(Tag::from_strings(vec![key.clone(), state[&key].clone()]));
        }
        tags
    };
    let state_event = EventV3 {
        id: state_id,
        pubkey: public_key,
        created_at,
        kind: EventKind::GitRepoAnnouncement,
        sig: Signature::zeroes(),
        content: String::new(),
        tags,
    };
    let coordinate = NAddr {
        d: identifier.clone(),
        relays: vec![unchecked_relay.clone()],
        kind: EventKind::GitRepoAnnouncement,
        author: public_key,
    };

    let _ = relay_url;

    Ok(GitSnapshot {
        repo_name,
        description,
        identifier,
        root_commit,
        git_servers,
        web,
        state,
        state_event,
        state_id,
        coordinate,
    })
}
