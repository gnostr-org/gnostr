use std::{io, time::Duration};

use anyhow::Result;
use ratatui::crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use proctitle::set_title;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Tabs, Wrap},
    Frame, Terminal,
};
use tokio::sync::mpsc::{Receiver, Sender};
use tui_input::{backend::crossterm::EventHandler, Input};

use crate::{
    event::ChatEvent,
    msg::{Msg, MsgKind},
};

pub fn run_chat_tui(
    topic: String,
    username: String,
    input_tx: Sender<ChatEvent>,
    mut output_rx: Receiver<ChatEvent>,
) -> Result<()> {
    let _ = set_title(&format!("gnostr-chat-{topic}"));

    let mut terminal = TerminalGuard::enter()?;
    let mut app = ChatTui::new(topic, username);

    loop {
        app.drain_incoming(&mut output_rx);
        terminal.terminal.draw(|frame| app.draw(frame))?;

        if event::poll(Duration::from_millis(75))? {
            match event::read()? {
                Event::Key(key) => {
                    if app.handle_key(key, &input_tx)? {
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
    Timeline,
    Composer,
}

struct ChatTui {
    topic: String,
    username: String,
    focus: Focus,
    show_help: bool,
    messages: Vec<Msg>,
    selected: usize,
    input: Input,
    status: String,
}

impl ChatTui {
    fn new(topic: String, username: String) -> Self {
        Self {
            topic,
            username,
            focus: Focus::Composer,
            show_help: false,
            messages: Vec::new(),
            selected: 0,
            input: Input::new(String::new()),
            status: "type a message and press enter".to_string(),
        }
    }

    fn drain_incoming(&mut self, output_rx: &mut Receiver<ChatEvent>) {
        while let Ok(event) = output_rx.try_recv() {
            match event {
                ChatEvent::ChatMessage(msg) => {
                    self.messages.push(msg);
                    self.selected = self.messages.len().saturating_sub(1);
                    self.status = format!("received {} message(s)", self.messages.len());
                }
                ChatEvent::ShowErrorMsg(text) => {
                    self.status = text;
                }
                ChatEvent::ShowInfoMsg(text) => {
                    self.status = text;
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

                let msg = Msg::default()
                    .set_kind(MsgKind::Chat)
                    .set_content(text, 0);
                input_tx
                    .blocking_send(ChatEvent::ChatMessage(msg.clone()))
                    .map_err(|e| anyhow::anyhow!("failed to queue chat message: {e}"))?;
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
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Char('?') | KeyCode::F(1) => self.show_help = true,
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.selected = self.selected.saturating_add(1).min(self.messages.len().saturating_sub(1));
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

    fn draw(&self, frame: &mut Frame) {
        let root = frame.area();
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)])
            .split(root);

        self.draw_header(frame, layout[0]);

        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(28), Constraint::Min(0)])
            .split(layout[1]);
        self.draw_sidebar(frame, body[0]);
        self.draw_timeline_and_composer(frame, body[1]);

        self.draw_footer(frame, layout[2]);

        if self.show_help {
            self.draw_help_overlay(frame, root);
        }
    }

    fn draw_header(&self, frame: &mut Frame, area: Rect) {
        let tabs = Tabs::new(vec![
            Line::from("timeline"),
            Line::from("composer"),
            Line::from("hybrid"),
        ])
        .select(match self.focus {
            Focus::Timeline => 0,
            Focus::Composer => 1,
        })
        .style(Style::default().fg(Color::Gray))
        .highlight_style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))
        .divider(Span::raw(" | "))
        .block(Block::default().title("chat").borders(Borders::ALL));

        frame.render_widget(tabs, area);
    }

    fn draw_sidebar(&self, frame: &mut Frame, area: Rect) {
        let selected = self.selected_message();
        let details = selected.map(|msg| {
            vec![
                Line::from(vec![Span::styled("from: ", Style::default().fg(Color::Magenta)), Span::raw(&msg.from)]),
                Line::from(vec![Span::styled("kind: ", Style::default().fg(Color::Magenta)), Span::raw(format!("{:?}", msg.kind))]),
                Line::from(vec![Span::styled("commit: ", Style::default().fg(Color::Magenta)), Span::raw(msg.commit_id.to_string())]),
                Line::from(vec![Span::styled("nostr: ", Style::default().fg(Color::Magenta)), Span::raw(if msg.nostr_event.is_some() { "yes" } else { "no" })]),
                Line::from(vec![Span::styled("chunks: ", Style::default().fg(Color::Magenta)), Span::raw(match (msg.sequence_num, msg.total_chunks) {
                    (Some(seq), Some(total)) => format!("{}/{}", seq + 1, total),
                    _ => "single".to_string(),
                })]),
            ]
        });

        let mut lines = vec![
            Line::from(vec![Span::styled("user: ", Style::default().fg(Color::Magenta)), Span::raw(&self.username)]),
            Line::from(vec![Span::styled("topic: ", Style::default().fg(Color::Magenta)), Span::raw(&self.topic)]),
            Line::from(vec![Span::styled("transport: ", Style::default().fg(Color::Magenta)), Span::raw("libp2p + nostr")]),
            Line::from(vec![Span::styled("focus: ", Style::default().fg(Color::Magenta)), Span::raw(match self.focus {
                Focus::Timeline => "timeline",
                Focus::Composer => "composer",
            })]),
            Line::from(vec![Span::styled("messages: ", Style::default().fg(Color::Magenta)), Span::raw(self.messages.len().to_string())]),
            Line::from(""),
            Line::from(vec![Span::styled("keys: ", Style::default().fg(Color::Magenta)), Span::raw("tab switch, enter send, ? help, q quit")]),
        ];

        if let Some(mut detail_lines) = details {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled("selected", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))]));
            lines.append(&mut detail_lines);
        }

        frame.render_widget(
            Paragraph::new(lines)
                .wrap(Wrap { trim: true })
                .block(Block::default().title("status").borders(Borders::ALL)),
            area,
        );
    }

    fn draw_timeline_and_composer(&self, frame: &mut Frame, area: Rect) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(5)])
            .split(area);
        self.draw_timeline(frame, layout[0]);
        self.draw_composer(frame, layout[1]);
    }

    fn draw_timeline(&self, frame: &mut Frame, area: Rect) {
        let items = if self.messages.is_empty() {
            vec![ListItem::new("no messages yet")]
        } else {
            self.messages
                .iter()
                .enumerate()
                .map(|(idx, msg)| {
                    let style = if idx == self.selected {
                        Style::default().fg(Color::Black).bg(Color::Magenta).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };
                    ListItem::new(Line::from(msg)).style(style)
                })
                .collect::<Vec<_>>()
        };

        let mut state = ListState::default();
        let selected = if self.messages.is_empty() {
            Some(0)
        } else {
            Some(self.selected.min(self.messages.len().saturating_sub(1)))
        };
        state.select(selected);
        frame.render_stateful_widget(
            List::new(items)
                .block(Block::default().title("timeline").borders(Borders::ALL))
                .highlight_symbol("▶ "),
            area,
            &mut state,
        );
    }

    fn draw_composer(&self, frame: &mut Frame, area: Rect) {
        let input = Paragraph::new(self.input.value())
            .style(Style::default().fg(if self.focus == Focus::Composer { Color::Cyan } else { Color::Gray }))
            .scroll((0, self.input.visual_scroll(area.width.saturating_sub(4) as usize) as u16))
            .block(Block::default().title("compose").borders(Borders::ALL));

        frame.render_widget(input, area);

        if self.focus == Focus::Composer {
            let cursor = self.input.visual_cursor();
            let scroll = self.input.visual_scroll(area.width.saturating_sub(4) as usize);
            frame.set_cursor_position((
                area.x + 1 + (cursor.max(scroll) - scroll) as u16,
                area.y + 1,
            ));
        }
    }

    fn draw_footer(&self, frame: &mut Frame, area: Rect) {
        let status = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("status: ", Style::default().fg(Color::Magenta)),
                Span::raw(&self.status),
            ]),
            Line::from(vec![
                Span::styled("mode: ", Style::default().fg(Color::Magenta)),
                Span::raw(match self.focus {
                    Focus::Timeline => "timeline",
                    Focus::Composer => "composer",
                }),
            ]),
        ])
        .wrap(Wrap { trim: true })
        .block(Block::default().title("state").borders(Borders::ALL));

        frame.render_widget(status, area);
    }

    fn draw_help_overlay(&self, frame: &mut Frame, area: Rect) {
        let popup = centered_rect(72, 70, area);
        frame.render_widget(Clear, popup);

        let help = Paragraph::new(vec![
            Line::from("q or Ctrl-c quit"),
            Line::from("tab switch between timeline and composer"),
            Line::from("enter send the current message from the composer"),
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
