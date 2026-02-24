use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap, Padding},
};
use std::{
    io::{self, stdout, Write},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, Mutex,
    },
    time::{Duration, Instant},
};

// Spinner utility for TUI
#[derive(Debug, Clone)]
pub struct TuiSpinner {
    frames: Vec<&'static str>,
    current_frame: usize,
}

impl TuiSpinner {
    pub fn new() -> Self {
        TuiSpinner {
            frames: vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            current_frame: 0,
        }
    }

    pub fn next_frame(&mut self) -> &str {
        self.current_frame = (self.current_frame + 1) % self.frames.len();
        self.frames[self.current_frame]
    }
}

#[derive(Debug, Clone)]
pub enum TxStatus {
    CrawlingSeeds,
    FetchingAddresses(String), // Peer address
    ConnectingToPeer(String),  // Peer address
    SendingTransaction(String), // Peer address
    Confirmed(String),         // Peer address
    NotFound(String),          // Peer address
    Error(String, String),     // Peer address, error message
    Done,
}

#[derive(Debug, Clone)]
pub struct TxProgress {
    pub peer_addr: String,
    pub status: TxStatus,
    pub spinner: TuiSpinner,
}

impl TxProgress {
    pub fn new(peer_addr: String, status: TxStatus) -> Self {
        TxProgress {
            peer_addr,
            status,
            spinner: TuiSpinner::new(),
        }
    }
}

pub struct TxApp {
    pub progress_states: Arc<Mutex<Vec<TxProgress>>>,
    pub running: Arc<AtomicBool>,
    pub tick_rate: Duration,
}

impl TxApp {
    pub fn new(progress_states: Arc<Mutex<Vec<TxProgress>>>) -> Self {
        TxApp {
            progress_states,
            running: Arc::new(AtomicBool::new(true)),
            tick_rate: Duration::from_millis(100),
        }
    }

    pub fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> anyhow::Result<()> {
        let (tx, rx) = mpsc::channel();
        let running_clone = self.running.clone();
        let tick_rate = self.tick_rate;

        std::thread::spawn(move || {
            let mut last_tick = Instant::now();
            loop {
                if !running_clone.load(Ordering::SeqCst) {
                    break;
                }
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| Duration::from_secs(0));

                if event::poll(timeout).expect("poll works")
                    && let CEvent::Key(key) = event::read().expect("can read events") {
                        tx.send(Event::Input(key)).expect("can send events");
                    }

                if last_tick.elapsed() >= tick_rate {
                    tx.send(Event::Tick).expect("can send tick event");
                    last_tick = Instant::now();
                }
            }
        });

        while self.running.load(Ordering::SeqCst) {
            terminal.draw(|f| {
                let size = f.size();
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(100)].as_ref())
                    .split(size);

                let mut progress_states_lock = self.progress_states.lock().unwrap();
                let mut lines: Vec<Line> = Vec::new();

                if progress_states_lock.is_empty() {
                    lines.push(Line::from(Span::raw("Initializing transaction delivery...")));
                } else {
                    for progress in progress_states_lock.iter_mut() {
                        let spinner_frame = progress.spinner.next_frame();
                        let status_text = match &progress.status {
                            TxStatus::CrawlingSeeds => format!("{} Crawling DNS seeds...", spinner_frame),
                            TxStatus::FetchingAddresses(addr) => format!("{} Fetching addresses from {}...", spinner_frame, addr),
                            TxStatus::ConnectingToPeer(addr) => format!("{} Connecting to peer {}...", spinner_frame, addr),
                            TxStatus::SendingTransaction(addr) => format!("{} Sending transaction to {}...", spinner_frame, addr),
                            TxStatus::Confirmed(addr) => format!("✅ Transaction confirmed by {}", addr),
                            TxStatus::NotFound(addr) => format!("❌ Transaction not found by {}", addr),
                            TxStatus::Error(addr, err) => format!("❌ Error with {}: {}", addr, err),
                            TxStatus::Done => "✅ All done!".to_string(),
                        };
                        lines.push(Line::from(Span::raw(status_text)));
                    }
                }

                let paragraph = Paragraph::new(lines)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Send Raw Transaction Progress")
                            .border_style(Style::default().fg(Color::Cyan))
                            .padding(ratatui::widgets::Padding::new(2, 2, 2, 2)),
                    )
                    .wrap(Wrap { trim: true });

                f.render_widget(paragraph, chunks[0]);
            })?;

            match rx.recv_timeout(tick_rate) {
                Ok(Event::Input(event)) => match event.code {
                    KeyCode::Char('q') => {
                        self.running.store(false, Ordering::SeqCst);
                    }
                    _ => {}
                },
                Ok(Event::Tick) => {
                    // No specific action needed on tick, drawing handles spinner update
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    self.running.store(false, Ordering::SeqCst);
                }
            }
        }
        Ok(())
    }
}

pub enum Event<I> {
    Input(I),
    Tick,
}

pub fn init_tui() -> anyhow::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

pub fn restore_tui(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> anyhow::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
