//use crate::ui::solarized_dark;
//use crate::ui::solarized_light;

//use libp2p::{gossipsub, mdns, noise, swarm::NetworkBehaviour, swarm::SwarmEvent, tcp, yamux};
//use ratatui::prelude::*;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    layout::{Constraint, Direction, Layout},
    style::Color,
    text::Line,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};

use ratatui::style::Style;
use std::{
    error::Error,
    io,
    sync::{Arc, Mutex},
    time::Duration,
};
use textwrap::{fill, Options};
use uuid::Uuid;
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

use crate::p2p::chat::msg::{self, MsgKind};

struct TerminalCleanup;

impl Drop for TerminalCleanup {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
    }
}

#[derive(Default)]
pub enum InputMode {
    #[default]
    Normal,
    Editing,
}

/// App holds the state of the application
pub struct App {
    /// Current value of the input box
    pub input: Input,
    /// Current input mode
    pub input_mode: InputMode,
    /// History of recorded messages
    pub messages: Arc<Mutex<Vec<msg::Msg>>>,
    pub _on_input_enter: Option<Box<dyn FnMut(msg::Msg)>>,
    pub msgs_scroll: usize,
    pub topic: String,
}

impl Default for App {
    fn default() -> Self {
        App {
            input: Input::default(),
            input_mode: InputMode::default(),
            messages: Default::default(),
            _on_input_enter: None,
            msgs_scroll: usize::MAX,
            topic: String::from("gnostr"),
        }
    }
}

impl App {
    pub fn on_submit<F: FnMut(msg::Msg) + 'static>(&mut self, hook: F) {
        self._on_input_enter = Some(Box::new(hook));
    }

    pub fn add_message(&self, msg: msg::Msg) {
        let mut msgs = self.messages.lock().unwrap();
        Self::add_msg(&mut msgs, msg);
    }

    fn add_msg(msgs: &mut Vec<msg::Msg>, msg: msg::Msg) {
        msgs.push(msg);
    }

    pub fn add_msg_fn(&self) -> Box<dyn FnMut(msg::Msg) + 'static + Send> {
        let m = self.messages.clone();
        Box::new(move |msg| {
            let mut msgs = m.lock().unwrap();
            Self::add_msg(&mut msgs, msg);
        })
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let _cleanup_guard = TerminalCleanup;
        // setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // run app
        run_app(&mut terminal, self)?;

        Ok(())
    }
}

fn gen_color_by_hash(s: &str) -> Color {
    static LIGHT_COLORS: [Color; 5] = [
        Color::LightMagenta,
        Color::LightGreen,
        Color::LightYellow,
        Color::LightBlue,
        Color::LightCyan,
        // Color::White,
    ];
    let h = s.bytes().fold(0, |acc, b| acc ^ b as usize);
    LIGHT_COLORS[h % LIGHT_COLORS.len()]
}

fn process_and_add_diff_message(app: &mut App, input_text: String) {
    let diff_content = input_text.strip_prefix("--diff ").unwrap_or(&input_text);
    let message_id = Uuid::new_v4().to_string();
    let lines: Vec<String> = diff_content.lines().map(|s| s.to_string()).collect();
    let total_chunks = lines.len(); // Each line is a chunk for diffs

    for (sequence_num, line) in lines.into_iter().enumerate() {
        let m = msg::Msg::default()
            .set_content(line, 0)
            .set_kind(MsgKind::GitDiff)
            .set_message_id(message_id.clone())
            .set_sequence_num(sequence_num)
            .set_total_chunks(total_chunks);
        app.add_message(m.clone());
        if let Some(ref mut hook) = app._on_input_enter {
            hook(m);
        }
    }
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    let tick_rate = Duration::from_millis(100);
    loop {
        terminal.draw(|f| ui(f, app))?;

        if !event::poll(tick_rate)? {
            continue;
        }

        if let Event::Key(key) = event::read()? {
            if key.code == KeyCode::Char('c')
                && key.modifiers.contains(event::KeyModifiers::CONTROL)
            {
                return Ok(());
            }

            match app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('e') | KeyCode::Char('i') => {
                        app.input_mode = InputMode::Editing;
                        app.msgs_scroll = usize::MAX;
                    }
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    KeyCode::Up => {
                        let l = app.messages.lock().unwrap().len();
                        app.msgs_scroll = app.msgs_scroll.saturating_sub(1).min(l);
                    }
                    KeyCode::Down => {
                        let l = app.messages.lock().unwrap().len();
                        app.msgs_scroll = app.msgs_scroll.saturating_add(1).min(l);
                    }
                    KeyCode::Esc => {
                        app.msgs_scroll = usize::MAX;
                        app.msgs_scroll = usize::MAX;
                        app.input.reset();
                    }
                    _ => {
                        app.msgs_scroll = usize::MAX;
                    }
                },
                InputMode::Editing => match key.code {
                    KeyCode::Enter => {
                        if !app.input.value().trim().is_empty() {
                            let input_text = app.input.value().to_owned();
                            if input_text.starts_with("--diff ") {
                                process_and_add_diff_message(app, input_text);
                            } else {
                                let wrapped_lines: Vec<String> = fill(&input_text, 80)
                                    .split('\n')
                                    .map(|s| s.to_string())
                                    .collect();
                                let total_chunks = wrapped_lines.len();
                                let message_id = Uuid::new_v4().to_string();

                                for (sequence_num, line) in wrapped_lines.into_iter().enumerate() {
                                    if !line.trim().is_empty() {
                                        let m = msg::Msg::default()
                                            .set_content(line, 0)
                                            .set_message_id(message_id.clone())
                                            .set_sequence_num(sequence_num)
                                            .set_total_chunks(total_chunks);
                                        app.add_message(m.clone());
                                        if let Some(ref mut hook) = app._on_input_enter {
                                            hook(m);
                                        }
                                    }
                                }
                            }
                        }
                        app.input.reset();
                    }
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                        app.msgs_scroll = app.messages.lock().unwrap().len();
                    }
                    _ => {
                        app.input.handle_event(&Event::Key(key));
                    }
                },
            }
        }
    }
}

//as popup widget is constructed in chat_details/mos.rs
fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3), // Header height
                Constraint::Fill(5),   // Messages height
                Constraint::Length(3), // Input height
            ]
            .as_ref(),
        )
        .split(f.area());

    // Header Widget
    let header_text = vec![Line::from(app.topic.as_str())];
    let header =
        Paragraph::new(header_text).block(Block::default().borders(Borders::ALL).title("Topic"));
    f.render_widget(header, chunks[0]);

    // Messages Widget
    let height = chunks[1].height; // Re-introduce height variable
    let message_area_width = chunks[1].width;
    let msgs = app.messages.lock().unwrap();
    
    let mut messages: Vec<ListItem> = Vec::new();

    for msg in msgs.iter().rev() {
        let mut current_message_spans: Vec<ratatui::text::Span> = Vec::new();
        let options = Options::new(message_area_width as usize);

        match msg.kind {
            MsgKind::Chat => {
                let (prefix, indent) = if msg.from == *msg::USER_NAME {
                    (format!("{}{} ", &msg.from, ">"), " ".repeat(msg.from.len() + 2))
                } else {
                    (format!(" {}{}", &msg.from, "> "), " ".repeat(msg.from.len() + 3))
                };
                let prefix_style = Style::default().fg(gen_color_by_hash(&msg.from));
                current_message_spans.push(ratatui::text::Span::styled(prefix.clone(), prefix_style));

                let content_width = message_area_width.saturating_sub(prefix.len() as u16);
                let wrapped_content = textwrap::wrap(&msg.content[0], Options::new(content_width as usize));

                for (idx, segment) in wrapped_content.into_iter().enumerate() {
                    if idx > 0 {
                        current_message_spans.push(ratatui::text::Span::raw("\n"));
                        current_message_spans.push(ratatui::text::Span::raw(indent.clone()));
                    }
                    current_message_spans.push(ratatui::text::Span::raw(segment.to_string()));
                }
            },
            MsgKind::GitDiff => {
                let first_line_indent_char_len = if !msg.content.is_empty() {
                    let first_line_char = msg.content[0].chars().next();
                    if first_line_char == Some('+') || first_line_char == Some('-') || first_line_char == Some('@') || first_line_char == Some('\\') {
                        1
                    } else {
                        0
                    }
                } else {
                    0
                };
                let indent_str = " ".repeat(first_line_indent_char_len);

                for (line_idx, line_content) in msg.content.iter().enumerate() {
                    let style = if line_content.starts_with('+') {
                        Style::default().fg(Color::Green)
                    } else if line_content.starts_with('-') {
                        Style::default().fg(Color::Red)
                    } else if line_content.starts_with('@') || line_content.starts_with('\\') {
                        Style::default().fg(Color::Cyan)
                    } else {
                        Style::default().fg(Color::White)
                    };

                    let wrapped_lines = textwrap::wrap(line_content, options.clone());

                    for (segment_idx, wrapped_segment) in wrapped_lines.iter().enumerate() { // Iterate over reference
                        if segment_idx > 0 { // This is a wrapped part of the same original line
                            current_message_spans.push(ratatui::text::Span::raw("\n"));
                            current_message_spans.push(ratatui::text::Span::raw(indent_str.clone()));
                        }
                        current_message_spans.push(ratatui::text::Span::styled(wrapped_segment.to_string(), style));
                    }
                    // Add a newline between distinct lines of the diff, unless it's the very last segment of the very last line
                    if line_idx < msg.content.len() - 1 || (!wrapped_lines.is_empty() && wrapped_lines.last().unwrap().len() == message_area_width as usize) {
                        current_message_spans.push(ratatui::text::Span::raw("\n"));
                    }
                }
            },
            _ => {
                // For other MsgKind, directly convert to ListItem
                messages.push(ListItem::new(ratatui::text::Text::from(Line::from(msg)))); // Fixed: Convert Line to Text
                continue; // Continue to the next message
            }
        }
        
        if !current_message_spans.is_empty() {
            messages.push(ListItem::new(Line::from(current_message_spans)));
        }
    }
    messages.truncate(height as usize); // Take only the visible number of lines

    let messages = List::new(messages)
        .direction(ratatui::widgets::ListDirection::BottomToTop)
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(messages, chunks[1]);

    // Input Widget
    let width = chunks[2].width.max(3) - 3; // Use width of the input chunk
    let scroll = app.input.visual_scroll(width as usize);

    let input_str = app.input.value();
    let mut spans = Vec::new();
    let default_input_style = match app.input_mode {
        InputMode::Normal => Style::default(),
        InputMode::Editing => Style::default().fg(Color::Cyan),
    };

    for (i, c) in input_str.chars().enumerate() {
        if (i + 1) % 80 == 0 { // Highlight every 80th character
            spans.push(ratatui::text::Span::styled(
                c.to_string(),
                default_input_style.fg(Color::Red),
            ));
        } else {
            spans.push(ratatui::text::Span::styled(
                c.to_string(),
                default_input_style,
            ));
        }
    }

    let input_line = Line::from(spans);

    let input = Paragraph::new(input_line)
        .style(default_input_style)
        .scroll((0, scroll as u16))
        .block(Block::default().borders(Borders::ALL).title("Input"));
    f.render_widget(input, chunks[2]);

    match app.input_mode {
        InputMode::Normal => {}
        InputMode::Editing => f.set_cursor_position((
            chunks[2].x + ((app.input.visual_cursor()).max(scroll) - scroll) as u16 + 1,
            chunks[2].y + 1,
        )),
    }
}
