//! Chat TUI demo for the asyncgit Nostr widgets.
//!
//! The screen layout mirrors the JS app's main sections: home, relays, settings,
//! NIP explorer, NIP-34, and search.

use std::{collections::HashMap, io, time::Duration};

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use gnostr_asyncgit::{
    tui::nostr::widgets::*,
    types::{
        ChannelCreationEvent, ChannelMessageEvent, EventKind, EventReference, EventV3, Filter,
        Id, IdHex, Metadata, NAddr, Nip19, Nip19Address, Nip19Event, Nip19Profile, Profile,
        PublicKey, PublicKeyHex, RelayInformationDocument, RelayList, RelayListUsage, RelayMessage,
        RelayUrl, RepoRef, RepoState, Tag, Unixtime, UncheckedUrl, Url, RelayUsageSet,
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

fn main() -> anyhow::Result<()> {
    let mut terminal = setup_terminal()?;
    let mut app = App::new();

    loop {
        terminal.draw(|frame| app.draw(frame))?;

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

    restore_terminal(&mut terminal)?;
    Ok(())
}

fn setup_terminal() -> anyhow::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Ok(Terminal::new(backend)?)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> anyhow::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

struct App {
    pages: Vec<Page>,
    selected: usize,
    data: DemoData,
}

impl App {
    fn new() -> Self {
        Self {
            pages: vec![
                Page::Overview,
                Page::Relays,
                Page::Settings,
                Page::Explorer,
                Page::Nip34,
                Page::Search,
            ],
            selected: 0,
            data: DemoData::sample(),
        }
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
        let root = frame.size();
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
        self.draw_page(frame, content[1], self.pages[self.selected]);

        frame.render_widget(
            Paragraph::new(self.status_line())
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

    fn draw_page(&self, frame: &mut Frame, area: Rect, page: Page) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(9), Constraint::Min(0)])
            .split(area);

        frame.render_widget(
            Paragraph::new(page.summary(&self.data))
                .wrap(Wrap { trim: true })
                .block(Block::default().title(page.title()).borders(Borders::ALL)),
            chunks[0],
        );

        page.render(frame, chunks[1], &self.data);
    }

    fn status_line(&self) -> String {
        let page = self.pages[self.selected].title();
        format!("selected view: {page} | crawler relays: {} | relay entries: {}", self.data.crawler_relays.len(), self.data.relay_list.0.len())
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
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
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
    crawler_relays: Vec<CrawlerRelay>,
    filter: Filter,
}

impl DemoData {
    fn sample() -> Self {
        let public_key = PublicKey::try_from_hex_string(
            "b0635d6a9851d3aed0cd6c495b282167acf761729078d975fc341b22650b07b9",
            true,
        )
        .expect("valid demo public key");
        let id = Id::try_from_hex_string(
            "5df64b33303d62afc799bdc36d178c07b2e1f0d824f31b7dc812219440affab6",
        )
        .expect("valid demo event id");
        let relay_url = RelayUrl::try_from_str("wss://relay.example.com").expect("valid relay url");
        let unchecked_relay = UncheckedUrl::from_str("wss://relay.example.com");
        let profile = Profile {
            pubkey: public_key,
            relays: vec![UncheckedUrl::from_str("wss://relay.example.com")],
        };
        let metadata = Metadata {
            name: Some("gnostr demo".to_string()),
            about: Some("chat TUI view that mirrors the JS app".to_string()),
            picture: Some("https://example.com/avatar.png".to_string()),
            nip05: Some("demo@example.com".to_string()),
            other: serde_json::Map::new(),
        };
        let event = EventV3::new_dummy();
        let relay_document = RelayInformationDocument {
            name: Some("demo relay".to_string()),
            description: Some("relay panel data from the chat demo".to_string()),
            banner: Some(Url::try_from_str("https://example.com/banner.png").expect("valid url")),
            icon: Some(Url::try_from_str("https://example.com/icon.png").expect("valid url")),
            pubkey: Some((&public_key).into()),
            self_pubkey: Some((&public_key).into()),
            contact: Some("mailto:relay@example.com".to_string()),
            supported_nips: vec![1, 11, 28, 34, 50],
            software: Some("gnostr-relay".to_string()),
            version: Some("0.1.0".to_string()),
            limitation: None,
            retention: vec![],
            relay_countries: vec!["US".to_string()],
            language_tags: vec!["en".to_string()],
            tags: vec!["nostr".to_string(), "relay".to_string()],
            posting_policy: None,
            payments_url: None,
            fees: None,
            other: serde_json::Map::new(),
        };
        let mut relay_list = RelayList::default();
        relay_list.0.insert(relay_url.clone(), RelayListUsage::Both);
        let relay_usage = RelayUsageSet::new_all();
        let nip19 = Nip19::Profile(Nip19Profile {
            public_key,
            relays: vec![relay_url.clone()],
        });
        let naddr = NAddr {
            d: "gnostr-demo".to_string(),
            relays: vec![unchecked_relay.clone()],
            kind: EventKind::LongFormContent,
            author: public_key,
        };
        let event_reference = EventReference::Id {
            id,
            author: Some(public_key),
            relays: vec![relay_url.clone()],
            marker: Some("root".to_string()),
        };
        let tag = Tag::new_event(id, Some(unchecked_relay.clone()), Some("root".to_string()));
        let repo_ref = RepoRef {
            name: "gnostr".to_string(),
            description: "repository announcement in the demo".to_string(),
            identifier: "gnostr".to_string(),
            root_commit: "0123456789abcdef0123456789abcdef01234567".to_string(),
            git_server: vec!["https://github.com/gnostr-org/gnostr.git".to_string()],
            web: vec!["https://gnostr.org".to_string()],
            relays: vec![unchecked_relay.clone()],
            hashtags: vec!["gnostr".to_string(), "nostr".to_string()],
            maintainers: vec![public_key],
            trusted_maintainer: public_key,
            events: HashMap::new(),
        };
        let mut repo_state_map = HashMap::new();
        repo_state_map.insert(
            "HEAD".to_string(),
            "ref: refs/heads/main".to_string(),
        );
        repo_state_map.insert(
            "refs/heads/main".to_string(),
            "0123456789abcdef0123456789abcdef01234567".to_string(),
        );
        let repo_state = RepoState {
            identifier: "gnostr".to_string(),
            state: repo_state_map,
            event: event.clone(),
        };
        let channel_creation = ChannelCreationEvent {
            channel_id: "gnostr-demo".to_string(),
            channel_name: Some("gnostr demo".to_string()),
            channel_description: Some("A chat panel that mirrors the JS relays and crawler views".to_string()),
            channel_picture: Some("https://example.com/channel.png".to_string()),
            relay_url: Some(UncheckedUrl::from_str("wss://relay.example.com")),
            pubkey: public_key,
        };
        let channel_message = ChannelMessageEvent {
            channel_id: "gnostr-demo".to_string(),
            message: "relay, crawler, and chat panes are aligned now".to_string(),
            reply_to: Some(id),
            root_message: Some(id),
            pubkey: public_key,
            relay_url: Some(UncheckedUrl::from_str("wss://relay.example.com")),
        };
        let relay_message = RelayMessage::Notice("relay message demo".to_string());
        let crawler_relays = vec![
            CrawlerRelay {
                contact: Some("relay@example.com".to_string()),
                description: Some("crawler-discovered relay".to_string()),
                name: Some("Relay One".to_string()),
                ping_ms: Some(17),
                software: Some("nostr-rs-relay".to_string()),
                supported_nips: Some(vec![1, 11, 34]),
                supported_nip_extensions: Some(vec!["nip50".to_string()]),
                version: Some("1.0".to_string()),
            },
            CrawlerRelay {
                contact: None,
                description: Some("backup relay".to_string()),
                name: Some("Relay Two".to_string()),
                ping_ms: Some(42),
                software: Some("gnostr-relay".to_string()),
                supported_nips: Some(vec![1, 28, 50]),
                supported_nip_extensions: Some(vec!["search".to_string()]),
                version: Some("0.9".to_string()),
            },
        ];
        let mut filter = Filter::new();
        let id_hex = IdHex::from(id);
        filter.add_id(&id_hex);
        filter.add_author(&PublicKeyHex::from(public_key));
        filter.add_event_kind(EventKind::TextNote);
        filter.add_event_kind(EventKind::ChannelCreation);
        filter.add_tag_value('e', id.as_hex_string());
        filter.add_tag_value('p', public_key.as_hex_string());
        filter.since = Some(Unixtime(1_668_572_286));
        filter.until = Some(Unixtime(1_789_000_000));
        filter.limit = Some(20);

        Self {
            public_key,
            profile,
            metadata,
            event,
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
        }
    }

    fn crawler_relays_text(&self) -> String {
        self.crawler_relays
            .iter()
            .enumerate()
            .map(|(idx, relay)| {
                format!(
                    "{}. {} | ping={}ms | nips={:?} | {}",
                    idx + 1,
                    relay.name.as_deref().unwrap_or("unknown"),
                    relay.ping_ms.unwrap_or_default(),
                    relay.supported_nips.clone().unwrap_or_default(),
                    relay.description.as_deref().unwrap_or("")
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}
