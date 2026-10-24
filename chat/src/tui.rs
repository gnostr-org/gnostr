use std::{
    cell::RefCell,
    io,
    rc::Rc,
    time::{Duration, Instant},
};

#[cfg(feature = "fractal")]
use std::collections::HashMap;

use anyhow::{Context, Result};
use proctitle::set_title;
use ratatui::crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Clear, List, ListItem, ListState, Paragraph, StatefulWidget, Widget, Wrap,
    },
    Frame, Terminal,
};
use tokio::sync::{broadcast::Receiver, mpsc::Sender};
use tracing::debug;
use tui_input::{backend::crossterm::EventHandler, Input};

use crate::{
    event::ChatEvent,
    msg::{Msg, MsgKind},
    session::{ChatNotification, ChatSession},
};
use gnostr_p2p::time::{Clock, ClockStatus, P2PClock};
use gnostr_tui::{
    mouse::{disable_mouse_capture, enable_mouse_capture},
    multi_panel::{MultiPanel, MultiPanelState},
};

pub fn run_chat_tui(topic: String, username: String, session: &ChatSession) -> Result<()> {
    let _ = set_title(&format!("gnostr-chat-{topic}"));
    debug!("run_chat_tui: starting topic={topic} username={username}");

    let input_tx = session.command_sender();
    let mut output_rx = session.subscribe();
    let mut terminal = TerminalGuard::enter()?;
    let mut app = ChatApp::new(topic, username);

    let mut last_tick = Instant::now();
    loop {
        let now = Instant::now();
        let dt = now - last_tick;
        last_tick = now;

        app.update(dt);
        app.drain_incoming(&mut output_rx);
        terminal.terminal.draw(|frame| app.draw(frame))?;

        if event::poll(Duration::from_millis(75)).context("chat tui poll failed")? {
            match event::read().context("chat tui read failed")? {
                Event::Key(key) => {
                    debug!("run_chat_tui: key={key:?} focus={:?}", app.focus);
                    if app.handle_key(key, &input_tx)? {
                        debug!("run_chat_tui: exit requested by key={key:?}");
                        break;
                    }
                }
                Event::Mouse(_) => {
                    // Mouse capture is enabled; events are accepted but not yet wired.
                }
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
    }

    debug!("run_chat_tui: exiting normally");
    Ok(())
}

struct TerminalGuard {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl TerminalGuard {
    fn enter() -> Result<Self> {
        enable_raw_mode()?;
        enable_mouse_capture()?;
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
        let _ = disable_mouse_capture();
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Focus {
    Timeline,
    Composer,
}

struct ChatApp {
    topic: String,
    username: String,
    focus: Focus,
    show_help: bool,
    messages: Vec<Msg>,
    selected: usize,
    input: Input,
    status: String,
    p2p_clock: P2PClock,
    connected_peers: Vec<(String, String)>,
    #[cfg(feature = "fractal")]
    perfect_ip_progress: HashMap<String, (usize, usize)>,
    panel_state: MultiPanelState,
    composer_area: Rc<RefCell<Option<Rect>>>,
}

impl ChatApp {
    fn new(topic: String, username: String) -> Self {
        let p2p_clock = P2PClock::new(1, &clock_checkpoint_path(&topic, &username));
        Self {
            topic,
            username,
            focus: Focus::Composer,
            show_help: false,
            messages: Vec::new(),
            selected: 0,
            input: Input::new(String::new()),
            status: "type a message and press enter".to_string(),
            p2p_clock,
            connected_peers: Vec::new(),
            #[cfg(feature = "fractal")]
            perfect_ip_progress: HashMap::new(),
            panel_state: MultiPanelState::open(8.0),
            composer_area: Rc::new(RefCell::new(None)),
        }
    }

    fn update(&mut self, dt: Duration) {
        self.panel_state.update(dt);
        #[cfg(feature = "fractal")]
        {
            let has_progress = !self.perfect_ip_progress.is_empty();
            let target = if has_progress { 1.0 } else { 0.0 };
            self.panel_state.right_sidebar.set_target(target);
        }
    }

    fn drain_incoming(&mut self, output_rx: &mut Receiver<ChatNotification>) {
        while let Ok(event) = output_rx.try_recv() {
            match event {
                ChatNotification::ChatMessage(msg) => {
                    self.messages.push(msg);
                    self.selected = self.messages.len().saturating_sub(1);
                    self.status = format!("received {} message(s)", self.messages.len());
                }
                ChatNotification::Error(text) => {
                    self.status = text;
                }
                ChatNotification::Info(text) => {
                    self.status = text;
                }
                ChatNotification::Connected { peer_id, endpoint } => {
                    self.status = format!("connected to peer {peer_id} via {endpoint}");
                    self.connected_peers.push((peer_id, endpoint));
                }
                ChatNotification::MessagePublished => {}
                #[cfg(feature = "fractal")]
                ChatNotification::PerfectIpProgress {
                    root,
                    received,
                    total,
                } => {
                    self.perfect_ip_progress
                        .insert(root.clone(), (received, total));
                    self.status = format!("{root}: {received}/{total} slices received");
                }
                #[cfg(feature = "fractal")]
                ChatNotification::PerfectIpComplete { root, output_path } => {
                    self.perfect_ip_progress
                        .insert(root.clone(), (usize::MAX, usize::MAX));
                    self.status = format!(
                        "{root}: reconstruction complete at {}",
                        output_path.display()
                    );
                }
            }
        }
    }

    fn handle_key(&mut self, key: KeyEvent, input_tx: &Sender<ChatEvent>) -> Result<bool> {
        if self.show_help {
            match key.code {
                KeyCode::Esc | KeyCode::Char('?') | KeyCode::F(1) => self.show_help = false,
                _ => {}
            }
            return Ok(false);
        }

        if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('c')) {
            debug!("handle_key: ctrl-c exit requested");
            return Ok(true);
        }

        match self.focus {
            Focus::Composer => self.handle_composer_key(key, input_tx),
            Focus::Timeline => self.handle_timeline_key(key),
        }
    }

    fn handle_composer_key(&mut self, key: KeyEvent, input_tx: &Sender<ChatEvent>) -> Result<bool> {
        match key.code {
            KeyCode::Esc => {
                self.focus = Focus::Timeline;
                self.status = "timeline focused".to_string();
            }
            KeyCode::Tab => {
                self.focus = Focus::Timeline;
                self.status = "timeline focused".to_string();
            }
            KeyCode::Enter => {
                let text = self.input.value().trim().to_string();
                if text.is_empty() {
                    self.status = "compose a non-empty message".to_string();
                    return Ok(false);
                }

                if let Some(nip) = parse_crawler_search_command(&text) {
                    if let Err(e) = input_tx.try_send(ChatEvent::CrawlerSearch { nip }) {
                        self.status = format!("failed to queue crawler search: {e}");
                        return Ok(false);
                    }
                    self.input.reset();
                    self.status = format!("searching crawler bucket {nip}");
                    return Ok(false);
                }

                let msg = Msg::default().set_kind(MsgKind::Chat).set_content(text, 0);
                if let Err(e) = input_tx.try_send(ChatEvent::ChatMessage(msg.clone())) {
                    self.status = format!("failed to queue chat message: {e}");
                    return Ok(false);
                }
                self.messages.push(msg);
                self.selected = self.messages.len().saturating_sub(1);
                self.input.reset();
                self.status = "message queued".to_string();
            }
            KeyCode::Char('?') => self.show_help = true,
            _ => {
                self.input.handle_event(&Event::Key(key));
            }
        }

        Ok(false)
    }

    fn handle_timeline_key(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc | KeyCode::Tab => {
                self.focus = Focus::Composer;
                self.status = "composer focused".to_string();
            }
            KeyCode::Char('q') => {
                debug!("handle_timeline_key: q exit requested");
                return Ok(true);
            }
            KeyCode::Char('?') | KeyCode::F(1) => self.show_help = true,
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.selected = self
                    .selected
                    .saturating_add(1)
                    .min(self.messages.len().saturating_sub(1));
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.selected = 0;
            }
            KeyCode::End | KeyCode::Char('G') => {
                self.selected = self.messages.len().saturating_sub(1);
            }
            KeyCode::PageUp => {
                self.selected = self.selected.saturating_sub(5);
            }
            KeyCode::PageDown => {
                self.selected = self
                    .selected
                    .saturating_add(5)
                    .min(self.messages.len().saturating_sub(1));
            }
            _ => {}
        }

        Ok(false)
    }

    fn draw(&mut self, frame: &mut Frame) {
        *self.composer_area.borrow_mut() = None;

        let widget = self.build_widget();
        frame.render_stateful_widget(widget, frame.area(), &mut self.panel_state);

        if self.focus == Focus::Composer {
            if let Some(area) = *self.composer_area.borrow() {
                let width = area.width.saturating_sub(2) as usize;
                let cursor = self.input.visual_cursor();
                let scroll = self.input.visual_scroll(width);
                frame.set_cursor_position((
                    area.x + 1 + (cursor.max(scroll) - scroll) as u16,
                    area.y,
                ));
            }
        }

        if self.show_help {
            self.draw_help_overlay(frame, frame.area());
        }
    }

    fn build_widget(&self) -> MultiPanel {
        // Snapshot data so the content closures can own it.
        let topic = self.topic.clone();
        let username = self.username.clone();
        let status = self.status.clone();
        let focus = self.focus;
        let messages = self.messages.clone();
        let messages_for_sidebar = messages.clone();
        let selected = self.selected;
        let input_value = self.input.value().to_string();
        let clock_now = self.p2p_clock.now_utc().format("%H:%M:%S").to_string();
        let metrics = self.p2p_clock.get_metrics();
        let clock_status = match metrics.status {
            ClockStatus::Init => "init".to_string(),
            ClockStatus::Synced => "synced".to_string(),
            ClockStatus::Slewing => "slewing".to_string(),
            ClockStatus::Unreliable(reason) => format!("unreliable:{reason}"),
        };
        let clock_offset = format!("{:+}ms", metrics.offset_ms);
        let peers = self.connected_peers.clone();
        let selected_msg = self.selected_message().cloned();

        #[cfg(feature = "fractal")]
        let perfect_ip_progress = self.perfect_ip_progress.clone();

        let composer_area = self.composer_area.clone();

        MultiPanel::new()
            .header_title("gnostr-chat")
            .header_height(4)
            .header_content(move |_state, area, buf| {
                let lines = vec![
                    Line::from(vec![
                        Span::styled("topic: ", Style::default().fg(Color::Magenta)),
                        Span::raw(&topic),
                    ]),
                    Line::from(vec![
                        Span::styled("user: ", Style::default().fg(Color::Magenta)),
                        Span::raw(&username),
                    ]),
                    Line::from(vec![
                        Span::styled("utc: ", Style::default().fg(Color::Magenta)),
                        Span::raw(&clock_now),
                        Span::raw("  "),
                        Span::styled("p2p: ", Style::default().fg(Color::Magenta)),
                        Span::raw(format!("{clock_status} {clock_offset}")),
                    ]),
                ];
                Paragraph::new(lines)
                    .wrap(Wrap { trim: true })
                    .render(area, buf);
            })
            .left_title("status")
            .left_content(move |_state, area, buf| {
                let mut lines = vec![
                    Line::from(vec![
                        Span::styled("focus: ", Style::default().fg(Color::Magenta)),
                        Span::raw(match focus {
                            Focus::Timeline => "timeline",
                            Focus::Composer => "composer",
                        }),
                    ]),
                    Line::from(vec![
                        Span::styled("messages: ", Style::default().fg(Color::Magenta)),
                        Span::raw(messages_for_sidebar.len().to_string()),
                    ]),
                    Line::from(vec![
                        Span::styled("peers: ", Style::default().fg(Color::Magenta)),
                        Span::raw(peers.len().to_string()),
                    ]),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("keys: ", Style::default().fg(Color::Magenta)),
                        Span::raw("tab switch, enter send, ? help, q quit"),
                    ]),
                ];

                if let Some(ref msg) = selected_msg {
                    lines.push(Line::from(""));
                    lines.push(Line::from(vec![Span::styled(
                        "selected",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    )]));
                    lines.push(Line::from(vec![
                        Span::styled("from: ", Style::default().fg(Color::Magenta)),
                        Span::raw(&msg.from),
                    ]));
                    lines.push(Line::from(vec![
                        Span::styled("kind: ", Style::default().fg(Color::Magenta)),
                        Span::raw(format!("{:?}", msg.kind)),
                    ]));
                    lines.push(Line::from(vec![
                        Span::styled("chunks: ", Style::default().fg(Color::Magenta)),
                        Span::raw(match (msg.sequence_num, msg.total_chunks) {
                            (Some(seq), Some(total)) => format!("{}/{}", seq + 1, total),
                            _ => "single".to_string(),
                        }),
                    ]));
                }

                Paragraph::new(lines)
                    .wrap(Wrap { trim: true })
                    .block(Block::default().borders(Borders::ALL))
                    .render(area, buf);
            })
            .main_title("timeline")
            .main_content(move |_state, area, buf| {
                let items: Vec<ListItem> = if messages.is_empty() {
                    vec![ListItem::new("no messages yet")]
                } else {
                    messages
                        .iter()
                        .enumerate()
                        .map(|(idx, msg)| {
                            let style = if idx == selected {
                                Style::default()
                                    .fg(Color::Black)
                                    .bg(Color::Magenta)
                                    .add_modifier(Modifier::BOLD)
                            } else {
                                Style::default()
                            };
                            ListItem::new(Line::from(msg)).style(style)
                        })
                        .collect()
                };

                let mut state = ListState::default();
                state.select(if messages.is_empty() {
                    Some(0)
                } else {
                    Some(selected.min(messages.len().saturating_sub(1)))
                });

                StatefulWidget::render(
                    List::new(items)
                        .block(Block::default().borders(Borders::ALL))
                        .highlight_symbol("▶ "),
                    area,
                    buf,
                    &mut state,
                );
            })
            .statusbar_title("state")
            .statusbar_content(move |_state, area, buf| {
                let lines = vec![
                    Line::from(vec![
                        Span::styled("status: ", Style::default().fg(Color::Magenta)),
                        Span::raw(&status),
                    ]),
                    Line::from(vec![
                        Span::styled("mode: ", Style::default().fg(Color::Magenta)),
                        Span::raw(match focus {
                            Focus::Timeline => "timeline",
                            Focus::Composer => "composer",
                        }),
                    ]),
                ];
                Paragraph::new(lines)
                    .wrap(Wrap { trim: true })
                    .block(Block::default().borders(Borders::ALL))
                    .render(area, buf);
            })
            .footer_title("compose")
            .footer_height(3)
            .footer_content(move |_state, area, buf| {
                let block = Block::default()
                    .title("compose")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(if focus == Focus::Composer {
                        Color::Cyan
                    } else {
                        Color::Gray
                    }));
                let inner = block.inner(area);
                *composer_area.borrow_mut() = Some(inner);
                block.render(area, buf);

                let width = inner.width.saturating_sub(2) as usize;
                let scroll = Input::new(input_value.clone()).visual_scroll(width);
                Paragraph::new(input_value.clone())
                    .scroll((0, scroll as u16))
                    .render(inner, buf);
            })
            .right_title("fractal")
            .right_content(move |_state, area, buf| {
                #[cfg(feature = "fractal")]
                {
                    let mut lines: Vec<Line> = Vec::new();
                    if perfect_ip_progress.is_empty() {
                        lines.push(Line::from("no fractal transfers"));
                    } else {
                        for (root, (received, total)) in &perfect_ip_progress {
                            let label = if *received == usize::MAX {
                                format!("{root}: complete")
                            } else {
                                format!("{root}: {received}/{total}")
                            };
                            lines.push(Line::from(label));
                        }
                    }
                    Paragraph::new(lines)
                        .block(Block::default().borders(Borders::ALL))
                        .render(area, buf);
                }
                #[cfg(not(feature = "fractal"))]
                {
                    Paragraph::new("fractal feature disabled")
                        .block(Block::default().borders(Borders::ALL))
                        .render(area, buf);
                }
            })
    }

    fn draw_help_overlay(&self, frame: &mut Frame, area: Rect) {
        let popup = centered_rect(72, 70, area);
        frame.render_widget(Clear, popup);

        let help = Paragraph::new(vec![
            Line::from("q or Ctrl-c quit"),
            Line::from("tab switch between timeline and composer"),
            Line::from("enter send the current message from the composer"),
            Line::from("/crawler <nip> search crawler bucket providers"),
            Line::from("j/k or arrows move through the timeline"),
            Line::from("home/end and pgup/pgdn jump in the timeline"),
            Line::from("? or F1 open and close this overlay"),
            Line::from("esc leaves the composer or closes help"),
        ])
        .wrap(Wrap { trim: true })
        .block(Block::default().title("help").borders(Borders::ALL));

        frame.render_widget(help, popup);
    }

    fn selected_message(&self) -> Option<&Msg> {
        self.messages.get(self.selected)
    }
}

fn parse_crawler_search_command(input: &str) -> Option<i32> {
    let trimmed = input.trim();
    let nip = trimmed
        .strip_prefix("/crawler ")
        .or_else(|| trimmed.strip_prefix("/search crawler "))?;
    nip.trim().parse().ok()
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
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
        .split(popup_layout[1])[1]
}

fn clock_checkpoint_path(topic: &str, username: &str) -> String {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "gnostr-chat-p2p-{}-{}.json",
        sanitize_for_filename(topic),
        sanitize_for_filename(username)
    ));
    path.to_string_lossy().into_owned()
}

fn sanitize_for_filename(value: &str) -> String {
    value
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_') {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_for_filename_replaces_path_separators() {
        assert_eq!(
            sanitize_for_filename("topic/name with spaces"),
            "topic_name_with_spaces"
        );
    }

    #[test]
    fn clock_checkpoint_path_uses_temp_dir() {
        let path = clock_checkpoint_path("topic/name", "user name");
        assert!(path.contains("gnostr-chat-p2p-topic_name-user_name.json"));
    }
}
