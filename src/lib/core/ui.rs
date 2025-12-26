/// This example is taken from https://raw.githubusercontent.com/fdehau/tui-rs/master/examples/user_input.rs
//use crate::ui::event::Event;
//use libp2p::{gossipsub, mdns, noise, swarm::NetworkBehaviour, swarm::SwarmEvent, tcp, yamux};
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

pub struct TerminalCleanup;

impl Drop for TerminalCleanup {
    fn drop(&mut self) {
        println!("TerminalCleanup::drop called - restoring terminal");
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
    }
}

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
    pub rendered_line_offset: usize,
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
            rendered_line_offset: 0,
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

    pub fn run(&mut self, cli: &crate::cli::GnostrCli) -> Result<(), Box<dyn Error>> {
        let _cleanup_guard = TerminalCleanup;
        // setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // run app
        run_app(&mut terminal, self, cli.screenshots)?;

        Ok(())
    }
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    screenshots: Option<u8>,
) -> io::Result<()> {
    let tick_rate = Duration::from_millis(100);
    let mut last_screenshot = std::time::Instant::now();

    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Some(interval) = screenshots {
            if last_screenshot.elapsed() >= Duration::from_secs(interval as u64) {
                let mut path = crate::cli::get_app_cache_path().unwrap();
                path.push("screenshots");
                std::fs::create_dir_all(&path).unwrap();
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                path.push(format!("screenshot-{}.png", timestamp));
                crate::utils::screenshot::make_screenshot_cross_platform(path.to_str().unwrap())
                    .unwrap();
                last_screenshot = std::time::Instant::now();
            }
        }

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
                        app.rendered_line_offset = 0; // Reset scroll when entering editing mode
                    }
                    KeyCode::Char('q') => {
                        //Char q is inappropriate for "Quit"
                        //in nested shell contexts (tui-term like widgets)
                        //for example gnostr edit mode - vim :q
                        //and we want gnostr (tui)
                        //to be compatable as a library
                        app.input_mode = InputMode::Normal;
                        //app.msgs_scroll = app.messages.lock().unwrap().len(); // Removed, as we're using rendered_line_offset
                        //return Ok(());
                    }
                    KeyCode::Up => {
                        app.rendered_line_offset = app.rendered_line_offset.saturating_add(1);
                    }
                    KeyCode::Down => {
                        app.rendered_line_offset = app.rendered_line_offset.saturating_sub(1);
                    }
                    KeyCode::Esc => {
                        app.input.reset();
                    }
                    _ => {
                        //app.msgs_scroll = usize::MAX; // Removed, as we're using rendered_line_offset
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
                        app.rendered_line_offset = 0; // Reset scroll to bottom on new message
                    }
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
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
fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        // .margin(2)
        .constraints([Constraint::Fill(5), Constraint::Length(3)].as_ref())
        .split(f.area());

    let width = chunks[0].width.max(3) - 3; // keep 2 for borders and 1 for cursor

    let scroll = app.input.visual_scroll(width as usize);
    let input = Paragraph::new(app.input.value())
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Style::default().fg(Color::Cyan),
        })
        .scroll((0, scroll as u16))
        .block(Block::default().borders(Borders::ALL).title("Input"));
    f.render_widget(input, chunks[1]);

    match app.input_mode {
        InputMode::Normal =>
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            {}

        InputMode::Editing => {
            // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
            f.set_cursor_position((
                // Put cursor past the end of the input text
                chunks[1].x + ((app.input.visual_cursor()).max(scroll) - scroll) as u16 + 1,
                // Move one line down, from the border to the input line
                chunks[1].y + 1,
            ));
        }
    }

    let height = chunks[0].height as usize;
    let msgs = app.messages.lock().unwrap();
    let all_rendered_lines: Vec<Line> = msgs
        .iter()
        .rev() // Display newest messages at the bottom
        .flat_map(|m| m.to_lines())
        .collect();

    let num_rendered_lines = all_rendered_lines.len();

    // Adjust rendered_line_offset to stay within bounds
    app.rendered_line_offset = app
        .rendered_line_offset
        .min(num_rendered_lines.saturating_sub(1));

    let start_index = app.rendered_line_offset;
    let end_index = (start_index + height).min(num_rendered_lines);

    let messages: Vec<ListItem> = all_rendered_lines[start_index..end_index]
        .iter()
        .rev() // Reverse again to display in correct order (newest at bottom)
        .cloned()
        .map(ListItem::new)
        .collect();
    let messages = List::new(messages)
        .direction(ratatui::widgets::ListDirection::BottomToTop)
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(messages, chunks[0]);
}
