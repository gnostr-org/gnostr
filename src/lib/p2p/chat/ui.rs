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
    text::{Line, Span}, // Added Span here
    widgets::{Block, Borders, List, ListItem, Paragraph, Clear, ListState},
    Frame, Terminal,
};

use ratatui::style::Style;
use ratatui::prelude::Stylize;
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

#[derive(Default, PartialEq, Eq)] // Add PartialEq, Eq for comparison
pub enum AppMode {
    #[default]
    Normal,
    Editing,
    SelectingMessage {
        messages: Vec<msg::Msg>,      // Filtered list of messages to select from
        selected_index: usize,        // Index of the currently selected message
        scroll_state: ListState,      // Scroll state for the selectable list
    },
}

/// App holds the state of the application
pub struct App {
    /// Current value of the input box
    pub input: Input,
    /// Current input mode
    pub mode: AppMode, // Renamed from input_mode to mode
    /// History of recorded messages
    pub messages: Arc<Mutex<Vec<msg::Msg>>>,
    pub _on_input_enter: Option<Box<dyn FnMut(msg::Msg)>>,
    pub msgs_scroll: usize,
    pub topic: String,
    pub selected_message_content_scroll: usize, // New field for scrolling within a selected message
    pub show_side_panel: bool,
}

impl Default for App {
    fn default() -> Self {
        App {
            input: Input::default(),
            mode: AppMode::default(),
            messages: Default::default(),
            _on_input_enter: None,
            msgs_scroll: usize::MAX,
            topic: String::from("gnostr"),
            selected_message_content_scroll: 0, // Initialize new field
            show_side_panel: false,
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

            match app.mode { // Changed from app.input_mode
                AppMode::Normal => {
                    let msgs = app.messages.lock().unwrap();
                    let is_single_scrollable_message = msgs.len() == 1
                        && (msgs[0].kind == MsgKind::GitDiff || msgs[0].kind == MsgKind::OneShot);

                    match key.code {
                        KeyCode::Char('e') | KeyCode::Char('i') => {
                            app.mode = AppMode::Editing;
                            app.msgs_scroll = usize::MAX;
                        }
                        KeyCode::Char('q') => {
                            return Ok(());
                        }
                        KeyCode::Char('\\') => {
                            // Toggle side panel
                            app.show_side_panel = !app.show_side_panel;
                        }
                        KeyCode::Up => {
                            if is_single_scrollable_message {
                                // Max scroll is determined by content length minus visible height
                                // This requires the actual visible height which is only available in `ui` function.
                                // For now, we'll use a rough estimate or handle overflow in `ui`.
                                // A more robust solution would pass visible height to run_app or calculate here.
                                let total_content_lines = msgs[0].content.len();
                                let visible_height = terminal.size()?.height.saturating_sub(6) as usize; // Estimate: total height - header - input
                                let max_scroll = total_content_lines.saturating_sub(visible_height).max(0);
                                app.selected_message_content_scroll = (app.selected_message_content_scroll + 1).min(max_scroll);
                            } else {
                                let l = msgs.len();
                                app.msgs_scroll = app.msgs_scroll.saturating_add(1).min(l);
                            }
                        }
                        KeyCode::Down => {
                            if is_single_scrollable_message {
                                app.selected_message_content_scroll = app.selected_message_content_scroll.saturating_sub(1);
                            } else {
                                let l = msgs.len();
                                app.msgs_scroll = app.msgs_scroll.saturating_sub(1).min(l);
                            }
                        }
                        KeyCode::Esc => {
                            app.msgs_scroll = usize::MAX;
                            app.selected_message_content_scroll = 0; // Reset content scroll on Esc
                            app.input.reset();
                        }
                        _ => {
                            if !is_single_scrollable_message {
                                app.msgs_scroll = usize::MAX;
                            }
                        }
                    }
                }
                AppMode::Editing => match key.code { // Changed from InputMode::Editing
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
                        app.mode = AppMode::Normal; // Changed from app.input_mode
                        app.msgs_scroll = app.messages.lock().unwrap().len();
                    }
                    _ => {
                        app.input.handle_event(&Event::Key(key));
                    }
                },
                AppMode::SelectingMessage { ref mut messages, ref mut selected_index, ref mut scroll_state } => {
                    match key.code {
                        KeyCode::Up => {
                            if *selected_index > 0 {
                                *selected_index -= 1;
                            }
                            scroll_state.select(Some(*selected_index));
                        }
                        KeyCode::Down => {
                            if *selected_index < messages.len() - 1 {
                                *selected_index += 1;
                            }
                            scroll_state.select(Some(*selected_index));
                        }
                        KeyCode::Enter => {
                            // Display the selected message
                            let selected_message = messages[*selected_index].clone();
                            // Clear existing messages and add the selected message for display
                            let mut all_messages = app.messages.lock().unwrap();
                            all_messages.clear(); // Clear existing messages
                            all_messages.push(selected_message); // Add the selected message
                            app.msgs_scroll = usize::MAX; // Scroll to bottom
                            app.selected_message_content_scroll = 0; // Reset scroll for new message

                            app.mode = AppMode::Normal; // Exit selection mode
                        }
                        KeyCode::Esc => {
                            app.mode = AppMode::Normal; // Exit selection mode
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

//as popup widget is constructed in chat_details/mos.rs
fn ui(f: &mut Frame, app: &App) {
    let main_layout_constraints = if app.show_side_panel {
        vec![Constraint::Percentage(70), Constraint::Percentage(30)]
    } else {
        vec![Constraint::Percentage(100)]
    };

    let main_layout_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(main_layout_constraints)
        .split(f.area());

    let message_and_input_area = main_layout_chunks[0];

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
        .split(message_and_input_area);

    // Header Widget
    let header_text = vec![Line::from(app.topic.as_str())];
    let header =
        Paragraph::new(header_text).block(Block::default().borders(Borders::ALL).title("Topic"));
    f.render_widget(header, chunks[0]);

    // Messages Widget
    let height = chunks[1].height; // Re-introduce height variable
    let message_area_width = chunks[1].width;
    let current_messages = app.messages.lock().unwrap();
    let is_single_scrollable_message = current_messages.len() == 1
        && (current_messages[0].kind == MsgKind::GitDiff || current_messages[0].kind == MsgKind::OneShot);

    let messages_to_render: Vec<ListItem>;
    let mut all_messages_list_items: Vec<ListItem> = Vec::new(); // Declare here

    if is_single_scrollable_message {
        let msg = &current_messages[0];
        let start_index = app.selected_message_content_scroll;
        let end_index = (start_index + height as usize).min(msg.content.len());
        
        let mut lines: Vec<Line> = Vec::new();
        for line_content in msg.content.iter().skip(start_index).take(end_index - start_index) {
            match msg.kind {
                MsgKind::GitDiff => {
                    let style = if line_content.starts_with('+') {
                        Style::default().fg(Color::Green)
                    } else if line_content.starts_with('-') {
                        Style::default().fg(Color::Red)
                    } else if line_content.starts_with('@') || line_content.starts_with('\\') {
                        Style::default().fg(Color::Cyan)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    lines.push(Line::from(Span::styled(line_content.clone(), style)));
                },
                MsgKind::OneShot => {
                    let orange_style = Style::default().fg(Color::Rgb(255, 165, 0));
                    lines.push(Line::from(Span::styled(line_content.clone(), orange_style)));
                },
                _ => unreachable!(), // Should not happen given `is_single_scrollable_message` check
            }
        }
        messages_to_render = vec![ListItem::new(lines)];

        let messages_list = List::new(messages_to_render)
            .block(Block::default().borders(Borders::NONE));
        f.render_widget(messages_list, chunks[1]);

    } else {
        // Populate all_messages_list_items only in this branch
        for msg in current_messages.iter().rev() {
            match msg.kind {
                MsgKind::Chat => {
                    let mut chat_spans: Vec<ratatui::text::Span> = Vec::new();
                    let (prefix, indent) = if msg.from == *msg::USER_NAME {
                        (format!("{}{} ", &msg.from, ">"), " ".repeat(msg.from.len() + 2))
                    } else {
                        (format!(" {}{}", &msg.from, "> "), " ".repeat(msg.from.len() + 3))
                    };
                    let prefix_style = Style::default().fg(gen_color_by_hash(&msg.from));
                    chat_spans.push(ratatui::text::Span::styled(prefix.clone(), prefix_style));

                    let content_width = message_area_width.saturating_sub(prefix.len() as u16);
                    let wrapped_content = textwrap::wrap(&msg.content[0], Options::new(content_width as usize));

                    for (idx, segment) in wrapped_content.into_iter().enumerate() {
                        if idx > 0 {
                            chat_spans.push(ratatui::text::Span::raw("\n"));
                            chat_spans.push(ratatui::text::Span::raw(indent.clone()));
                        }
                        chat_spans.push(ratatui::text::Span::raw(segment.to_string()));
                    }
                    all_messages_list_items.push(ListItem::new(Line::from(chat_spans)));
                },
                MsgKind::GitDiff => {
                    for line_content in msg.content.iter() {
                        let style = if line_content.starts_with('+') {
                            Style::default().fg(Color::Green)
                        } else if line_content.starts_with('-') {
                            Style::default().fg(Color::Red)
                        } else if line_content.starts_with('@') || line_content.starts_with('\\') {
                            Style::default().fg(Color::Cyan)
                        } else {
                            Style::default().fg(Color::White)
                        };
                        all_messages_list_items.push(ListItem::new(Line::from(Span::styled(line_content.clone(), style))));
                    }
                },
                _ => {
                    all_messages_list_items.push(ListItem::new(ratatui::text::Text::from(Line::from(msg))));
                }
            }
        }
        
        let num_total_messages = all_messages_list_items.len();
        let num_visible_messages = height as usize;

        let start_index = app.msgs_scroll.min(num_total_messages.saturating_sub(num_visible_messages).max(0));
        let end_index = (start_index + num_visible_messages).min(num_total_messages);
        
        let visible_messages: Vec<ListItem> = all_messages_list_items[start_index..end_index].to_vec();

        let messages_list = List::new(visible_messages)
            .direction(ratatui::widgets::ListDirection::BottomToTop)
            .block(Block::default().borders(Borders::NONE));
        f.render_widget(messages_list, chunks[1]);
    }

    if let AppMode::SelectingMessage { messages, selected_index: _, scroll_state } = &app.mode {
        let popup_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ])
            .split(f.area())[1];

        let popup_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ])
            .split(popup_area)[1];

        f.render_widget(Clear, f.area()); // Clear the area first

        let items: Vec<ListItem> = messages.iter().map(|msg| {
            let mut lines: Vec<Line> = Vec::new();
            let message_id_display = msg.message_id.as_deref().unwrap_or("N/A");
            lines.push(Line::from(Span::styled(format!("ID: {}", message_id_display), Style::default().fg(Color::DarkGray))));

            match msg.kind {
                MsgKind::GitDiff => {
                    for line_content in msg.content.iter() {
                        let style = if line_content.starts_with('+') {
                            Style::default().fg(Color::Green)
                        } else if line_content.starts_with('-') {
                            Style::default().fg(Color::Red)
                        } else if line_content.starts_with('@') || line_content.starts_with('\\') {
                            Style::default().fg(Color::Cyan)
                        } else {
                            Style::default().fg(Color::White)
                        };
                        lines.push(Line::from(Span::styled(line_content.clone(), style)));
                    }
                },
                MsgKind::OneShot => {
                    let orange_style = Style::default().fg(Color::Rgb(255, 165, 0));
                    if let Some(first_line) = msg.content.first() {
                        lines.push(Line::from(Span::styled(format!("[ONESHOT] {}: {}", msg.from, first_line), orange_style)));
                    }
                    for line_content in msg.content.iter().skip(1) {
                        lines.push(Line::from(Span::styled(line_content.clone(), orange_style)));
                    }
                },
                _ => {
                    // Fallback for other message types, though only GitDiff and OneShot should be here
                    if let Some(first_line) = msg.content.first() {
                        lines.push(Line::from(Span::styled(first_line.clone(), Style::default().fg(Color::White))));
                    }
                }
            }
            ListItem::new(lines)
        }).collect();

        let diff_list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Select Message"))
            .highlight_style(Style::default().fg(Color::Black).bg(Color::White));
            
        f.render_stateful_widget(diff_list, popup_area, &mut scroll_state.clone());
    }

    // Input Widget
    let width = chunks[2].width.max(3) - 3; // Use width of the input chunk
    let scroll = app.input.visual_scroll(width as usize);

    let input_str = app.input.value();
    let mut spans = Vec::new();
    let default_input_style = match app.mode {
        AppMode::Normal => Style::default(),
        AppMode::Editing => Style::default().fg(Color::Cyan),
        AppMode::SelectingMessage { .. } => Style::default().fg(Color::DarkGray), // Indicate non-editable
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

    match app.mode {
        AppMode::Normal => {}
        AppMode::Editing => f.set_cursor_position((
            chunks[2].x + ((app.input.visual_cursor()).max(scroll) - scroll) as u16 + 1,
            chunks[2].y + 1,
        )),
        AppMode::SelectingMessage { .. } => {} // No cursor in this mode
    }

    // Render side panel if active
    if app.show_side_panel {
        let side_panel_area = main_layout_chunks[1];
        let side_panel = Block::default()
            .borders(Borders::ALL)
            .title("Side Panel")
            .fg(Color::White);
        f.render_widget(side_panel, side_panel_area);
    }
}
