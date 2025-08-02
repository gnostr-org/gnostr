use crate::blockheight::blockheight_sync;
use crate::chat::msg;
use crate::weeble::weeble_sync;
use crate::wobble::wobble_sync;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};

use std::{
    env,
    error::Error,
    io,
    sync::{Arc, Mutex},
    time::Duration,
};
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

#[derive(Default)]
pub enum InputMode {
    #[default]
    Normal,
    Editing,
    Help,
    BackSlash,
    ForwardSlash,
    VimLike,
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
    pub topic: Input,
    pub diffs: Arc<Mutex<Vec<msg::Msg>>>,
}

impl Default for App {
    fn default() -> Self {
        App {
            input: Input::default(),
            input_mode: InputMode::default(),
            messages: Default::default(),
            _on_input_enter: None,
            msgs_scroll: usize::MAX,
            topic: Input::default(),
            diffs: Default::default(), //TODO: like messages
        }
    }
}

impl App {
    pub fn on_submit<F: FnMut(msg::Msg) + 'static>(&mut self, hook: F) {
        self._on_input_enter = Some(Box::new(hook));
    }

    pub fn add_message(&self, msg: msg::Msg) {
        //ref src/lib/chat/ui.rs
        let mut msgs = self.messages.lock().unwrap();
        Self::add_msg(&mut msgs, msg);
    }

    fn add_msg(msgs: &mut Vec<msg::Msg>, msg: msg::Msg) {
        msgs.push(msg);
    }

    //pubic
    pub fn add_msg_fn(&self) -> Box<dyn FnMut(msg::Msg) + 'static + Send> {
        let m = self.messages.clone();
        Box::new(move |msg| {
            let mut msgs = m.lock().unwrap();
            Self::add_msg(&mut msgs, msg);
        })
    }
    //GitCommitDiff type
    pub fn add_diff_message(&self, msg: msg::Msg) {
        let mut diffs = self.diffs.lock().unwrap();
        //TODO add to Topic
        Self::add_diff(&mut diffs, msg);
    }

    fn add_diff(diffs: &mut Vec<msg::Msg>, msg: msg::Msg) {
        diffs.push(msg);
    }

    pub fn add_diff_fn(&self) -> Box<dyn FnMut(msg::Msg) + 'static + Send> {
        let m = self.diffs.clone();
        Box::new(move |msg| {
            let mut diffs = m.lock().unwrap();
            Self::add_diff(&mut diffs, msg);
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
            //env::set_var("BLOCKHEIGHT", blockheight_sync());
            env::set_var("WEEBLE", weeble_sync().unwrap().to_string());
            env::set_var("WOBBLE", wobble_sync().unwrap().to_string());
            continue;
        }

        if let Event::Key(key) = event::read()? {
            if key.code == KeyCode::Char('c')
                && key.modifiers.contains(event::KeyModifiers::CONTROL)
            {
                return Ok(());
            }

            match app.input_mode {
                // Modal Commands
                // InputMode::Normal
                InputMode::Normal => match key.code {
                    KeyCode::Char('?') => {
                        //not empty
                        if !app.input.value().trim().is_empty() {
                            let m = msg::Msg::default()//default Msg type Chat
                                .set_content(app.input.value().to_owned(), 0 as usize);
                            app.add_message(m.clone());
                            if let Some(ref mut hook) = app._on_input_enter {
                                hook(m);
                            }
                        } else {
                            //TODO refresh and query topic nostr DMs
                            let m = msg::Msg::default()//default Msg type Chat
                                .set_content("<?> TODO: help".to_string(), 0 as usize);
                            app.add_message(m.clone());
                            if let Some(ref mut hook) = app._on_input_enter {
                                hook(m);
                            }
                        }
                        app.input.reset();
                        app.input_mode = InputMode::Help;
                    }
                    KeyCode::Char('/') => {
                        if !app.input.value().trim().is_empty() {
                            let m = msg::Msg::default()//default Msg type Chat
                                .set_content(app.input.value().to_owned(), 0 as usize);
                            app.add_message(m.clone());
                            if let Some(ref mut hook) = app._on_input_enter {
                                hook(m);
                            }
                        } else {
                            //TODO refresh and query topic nostr DMs
                            let m = msg::Msg::default() //default Msg type Chat
                                .set_content(
                                    format!(
                                        "{}/{}/{}>{}",
                                        &env::var("WEEBLE").unwrap(),
                                        &env::var("BLOCKHEIGHT").unwrap(),
                                        &env::var("WOBBLE").unwrap(),
                                        "</>".to_string()
                                    ),
                                    0 as usize,
                                );
                            app.add_message(m.clone());
                            if let Some(ref mut hook) = app._on_input_enter {
                                hook(m);
                            }
                        }
                        app.input.reset();
                    }
                    KeyCode::Char('\\') => {
                        if !app.input.value().trim().is_empty() {
                            let m = msg::Msg::default()//default Msg type Chat
                                .set_content(app.input.value().to_owned(), 0 as usize);
                            app.add_message(m.clone());
                            if let Some(ref mut hook) = app._on_input_enter {
                                hook(m);
                            }
                        } else {
                            //TODO refresh and query topic nostr DMs
                            let m = msg::Msg::default() //default Msg type Chat
                                .set_content(
                                    format!(
                                        "{}/{}/{}>{}",
                                        &env::var("WEEBLE").unwrap(),
                                        &env::var("BLOCKHEIGHT").unwrap(),
                                        &env::var("WOBBLE").unwrap(),
                                        "<\\>".to_string()
                                    ),
                                    0 as usize,
                                );
                            app.add_message(m.clone());
                            if let Some(ref mut hook) = app._on_input_enter {
                                hook(m);
                            }
                        }
                        app.input.reset();
                    }
                    KeyCode::Char(':') => {
                        if !app.input.value().trim().is_empty() {
                            let m = msg::Msg::default()//default Msg type Chat
                                .set_content(app.input.value().to_owned(), 0 as usize);
                            app.add_message(m.clone());
                            if let Some(ref mut hook) = app._on_input_enter {
                                hook(m);
                            }
                        } else {
                            //TODO refresh and query topic nostr DMs
                            let m = msg::Msg::default() //default Msg type Chat
                                .set_content(
                                    format!(
                                        "{}/{}/{}>{}",
                                        &env::var("WEEBLE").unwrap(),
                                        &env::var("BLOCKHEIGHT").unwrap(),
                                        &env::var("WOBBLE").unwrap(),
                                        "<:>".to_string()
                                    ),
                                    0 as usize,
                                );
                            app.add_message(m.clone());
                            if let Some(ref mut hook) = app._on_input_enter {
                                hook(m);
                            }
                        }
                        app.input.reset();
                        app.input_mode = InputMode::VimLike;
                    }
                    KeyCode::Char('e') | KeyCode::Char('i') => {
                        app.input_mode = InputMode::Editing;
                        app.msgs_scroll = usize::MAX;
                    }
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    //TODO Navigate to Topic
                    //Edit Topic Mode
                    KeyCode::Up => {
                        let l = app.messages.lock().unwrap().len();

                        app.msgs_scroll = app.msgs_scroll.saturating_sub(1).min(l);
                    }
                    KeyCode::Down => {
                        let l = app.messages.lock().unwrap().len();

                        app.msgs_scroll = app.msgs_scroll.saturating_add(1).min(l);
                    }
                    KeyCode::Enter => {
                        if !app.input.value().trim().is_empty() {
                            let m = msg::Msg::default()
                                .set_content(app.input.value().to_owned(), 0 as usize);
                            app.add_message(m.clone());
                            if let Some(ref mut hook) = app._on_input_enter {
                                hook(m);
                            }
                        } else {
                            //TODO refresh and query topic nostr DMs
                            let m = msg::Msg::default().set_content(
                                format!(
                                    "{}/{}/{}>{}",
                                    &env::var("WEEBLE").unwrap(),
                                    &env::var("BLOCKHEIGHT").unwrap(),
                                    &env::var("WOBBLE").unwrap(),
                                    "<ENTER>".to_string()
                                ),
                                0 as usize,
                            );
                            app.add_message(m.clone());
                            if let Some(ref mut hook) = app._on_input_enter {
                                hook(m);
                            }
                        }
                        app.input.reset();
                    }
                    KeyCode::Esc => {
                        app.msgs_scroll = usize::MAX;
                        app.msgs_scroll = usize::MAX;
                        app.input.reset();
                        let m = msg::Msg::default().set_content(
                            format!(
                                "{}/{}/{}>{}",
                                &env::var("WEEBLE").unwrap(),
                                &env::var("BLOCKHEIGHT").unwrap(),
                                &env::var("WOBBLE").unwrap(),
                                "<ESC>".to_string()
                            ),
                            0 as usize,
                        );
                        app.add_message(m.clone());
                        if let Some(ref mut hook) = app._on_input_enter {
                            hook(m);
                        }
                    }
                    _ => {
                        app.msgs_scroll = usize::MAX;
                    }
                },
                // InputMode::Editing
                InputMode::Editing => match key.code {
                    KeyCode::Enter => {
                        if !app.input.value().trim().is_empty() {
                            let m = msg::Msg::default()
                                .set_content(app.input.value().to_owned(), 0 as usize);
                            app.add_message(m.clone());
                            if let Some(ref mut hook) = app._on_input_enter {
                                hook(m);
                            }
                        } else {
                            //TODO refresh and query topic nostr DMs
                            let m = msg::Msg::default().set_content(
                                "InputMode::Editing:KeyCode::Enter".to_string(),
                                0 as usize,
                            );
                            app.add_message(m.clone());
                            if let Some(ref mut hook) = app._on_input_enter {
                                hook(m);
                            }
                        }
                        app.input.reset();
                    }
                    KeyCode::Esc => {
                        if !app.input.value().trim().is_empty() {
                            //reset input and stay in editing mode
                            app.input.reset();
                        } else {
                            //TODO prompt user if reset app.input
                            app.input_mode = InputMode::Normal;
                            app.msgs_scroll = app.messages.lock().unwrap().len();
                        }
                    }
                    KeyCode::Up => {
                        if !app.input.value().trim().is_empty() {
                        } else {
                            let l = app.messages.lock().unwrap().len();
                            app.msgs_scroll = app.msgs_scroll.saturating_sub(1).min(l);
                        }
                    }
                    KeyCode::Down => {
                        if !app.input.value().trim().is_empty() {
                        } else {
                            let l = app.messages.lock().unwrap().len();
                            app.msgs_scroll = app.msgs_scroll.saturating_add(1).min(l);
                        }
                    }
                    _ => {
                        //TODO count repeat key events
                        app.input.handle_event(&Event::Key(key));
                    }
                },
                // InputMode::BackSlash
                InputMode::BackSlash => match key.code {
                    KeyCode::Esc => {
                        if !app.input.value().trim().is_empty() {
                        } else {
                            //TODO prompt user if reset app.input
                            app.input_mode = InputMode::Normal;
                            app.msgs_scroll = app.messages.lock().unwrap().len();
                        }
                    }
                    _ => {
                        //TODO count repeat key events
                        app.input.handle_event(&Event::Key(key));
                    }
                },
                // InputMode::ForwardSlash
                InputMode::ForwardSlash => match key.code {
                    KeyCode::Esc => {
                        if !app.input.value().trim().is_empty() {
                        } else {
                            //TODO prompt user if reset app.input
                            app.input_mode = InputMode::Normal;
                            app.msgs_scroll = app.messages.lock().unwrap().len();
                        }
                    }
                    _ => {
                        //TODO count repeat key events
                        app.input.handle_event(&Event::Key(key));
                    }
                },
                // InputMode::VimLike
                InputMode::VimLike => match key.code {
                    KeyCode::Esc => {
                        if !app.input.value().trim().is_empty() {
                        } else {
                            //TODO prompt user if reset app.input
                            app.input_mode = InputMode::Normal;
                            app.msgs_scroll = app.messages.lock().unwrap().len();
                        }
                    }
                    _ => {
                        //TODO count repeat key events
                        app.input.handle_event(&Event::Key(key));
                    }
                },
                // InputMode::Help
                InputMode::Help => match key.code {
                    KeyCode::Esc => {
                        if !app.input.value().trim().is_empty() {
                        } else {
                            //TODO prompt user if reset app.input
                            app.input_mode = InputMode::Normal;
                            app.msgs_scroll = app.messages.lock().unwrap().len();
                        }
                    }
                    _ => {
                        //TODO count repeat key events
                        app.input.handle_event(&Event::Key(key));
                    }
                },
            }
        }
    }
}

//as popup widget is constructed in chat_details/mod.rs
fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        // .margin(2)
        .constraints(
            [
                Constraint::Length(10), /*fit a git commit*/
                Constraint::Fill(5),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(f.size());

    let width = chunks[0].width.max(3) - 3; // keep 2 for borders and 1 for cursor

    let scroll = app.input.visual_scroll(width as usize);
    let input = Paragraph::new(app.input.value())
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Style::default().fg(Color::Cyan),
            InputMode::BackSlash => Style::default().fg(Color::Blue),
            InputMode::ForwardSlash => Style::default().fg(Color::Blue),
            InputMode::VimLike => Style::default().fg(Color::Green),
            InputMode::Help => Style::default().fg(Color::Red),
        })
        .scroll((0, scroll as u16))
        .block(Block::default().borders(Borders::ALL).title("Input"));
    f.render_widget(input, chunks[2]);

    match app.input_mode {
        InputMode::Normal =>
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            {}

        InputMode::Editing => {
            // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
            f.set_cursor(
                // Put cursor past the end of the input text
                chunks[2].x + ((app.input.visual_cursor()).max(scroll) - scroll) as u16 + 1,
                // Move one line down, from the border to the input line
                chunks[2].y + 1,
            )
        }
        InputMode::BackSlash => {}
        InputMode::ForwardSlash => {}
        InputMode::VimLike => {}
        InputMode::Help => {}
    }

    let height = chunks[1].height;
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

    let topic = Paragraph::new(app.topic.value())
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Style::default().fg(Color::Cyan),
            InputMode::BackSlash => Style::default().fg(Color::Blue),
            InputMode::ForwardSlash => Style::default().fg(Color::Blue),
            InputMode::VimLike => Style::default().fg(Color::Green),
            InputMode::Help => Style::default().fg(Color::Red),
        })
        .scroll((0, scroll as u16))
        .block(Block::default().borders(Borders::ALL).title("TOPIC"));
    f.render_widget(topic, chunks[0]);
}
