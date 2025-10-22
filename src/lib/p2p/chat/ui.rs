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
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

use crate::p2p::chat::msg;

#[derive(Default)]
pub enum InputMode {
    Normal,
    #[default]
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
        // setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // run app
        run_app(&mut terminal, self)?;

        // restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        Ok(())
    }
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    let tick_rate = Duration::from_millis(100);
    loop {
        terminal.draw(|f| ui(f, &app))?;

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
                            let m =
                                msg::Msg::default().set_content(app.input.value().to_owned(), 0);
                            app.add_message(m.clone());
                            if let Some(ref mut hook) = app._on_input_enter {
                                hook(m);
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
        .constraints([
            Constraint::Length(3), // Header height
            Constraint::Fill(5),   // Messages height
            Constraint::Length(3), // Input height
        ].as_ref())
        .split(f.size());

    // Header Widget
    let header_text = vec![Line::from(app.topic.as_str())];
    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::ALL).title("Topic"));
    f.render_widget(header, chunks[0]);

    // Messages Widget
    let height = chunks[1].height; // Use height of the messages chunk
    let msgs = app.messages.lock().unwrap();
    let messages: Vec<ListItem> = msgs[0..app.msgs_scroll.min(msgs.len())]
        .iter()
        .rev()
        .map(|m| ListItem::new(Line::from(m)))
        .take(height as usize)
        .collect();
    let messages = List::new(messages)
        .direction(ratatui::widgets::ListDirection::BottomToTop)
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(messages, chunks[1]);

    // Input Widget
    let width = chunks[2].width.max(3) - 3; // Use width of the input chunk
    let scroll = app.input.visual_scroll(width as usize);
    let input = Paragraph::new(app.input.value())
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Style::default().fg(Color::Cyan),
        })
        .scroll((0, scroll as u16))
        .block(Block::default().borders(Borders::ALL).title("Input"));
    f.render_widget(input, chunks[2]);

    match app.input_mode {
        InputMode::Normal => {}
        InputMode::Editing => {
            f.set_cursor(
                chunks[2].x + ((app.input.visual_cursor()).max(scroll) - scroll) as u16 + 1,
                chunks[2].y + 1,
            )
        }
    }
}
