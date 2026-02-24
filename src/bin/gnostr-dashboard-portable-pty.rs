// src/bin/gnostr_dashboard.rs
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Terminal,
};
use std::{
    io::{self, Read},
    path::PathBuf,
    sync::{atomic::{AtomicUsize, Ordering}, Arc, Mutex},
    time::{Duration, Instant},
};
use vt100::Parser;

// Standard Bitcoin Orange: #F7931A
const BITCOIN_ORANGE: Color = Color::Rgb(247, 147, 26);

const BITCOIN_LOGO: [&str; 15] = [
    "⠀⠀⠀⠀⠀⠀⠀⠀⣀⣤⣴⣶⣾⣿⣿⣿⣿⣷⣶⣦⣤⣀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⣠⣴⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣦⣄⠀⠀⠀⠀⠀",
    "⠀⠀⠀⣠⣾⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⠿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣄⠀⠀⠀",
    "⠀⠀⣴⣿⣿⣿⣿⣿⣿⣿⠟⠿⠿⡿⠀⢰⣿⠁⢈⣿⣿⣿⣿⣿⣿⣿⣿⣦⠀⠀",
    "⠀⣼⣿⣿⣿⣿⣿⣿⣿⣿⣤⣄⠀⠀⠀⠈⠉⠀⠸⠿⣿⣿⣿⣿⣿⣿⣿⣿⣧⠀",
    "⢰⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡏⠀⠀⢠⣶⣶⣤⡀⠀⠈⢻⣿⣿⣿⣿⣿⣿⣿⡆",
    "⣾⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠃⠀⠀⠼⣿⣿⡿⠃⠀⠀⢸⣿⣿⣿⣿⣿⣿⣿⣷",
    "⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡟⠀⠀⢀⣀⣀⠀⠀⠀⠀⢴⣿⣿⣿⣿⣿⣿⣿⣿⣿",
    "⢿⣿⣿⣿⣿⣿⣿⣿⢿⣿⠁⠀⠀⣼⣿⣿⣿⣦⠀⠀⠈⢻⣿⣿⣿⣿⣿⣿⣿⡿",
    "⠸⣿⣿⣿⣿⣿⣿⣏⠀⠀⠀⠀⠀⠛⠛⠿⠟⠋⠀⠀⠀⣾⣿⣿⣿⣿⣿⣿⣿⠇",
    "⠀⢻⣿⣿⣿⣿⣿⣿⣿⣿⠇⠀⣤⡄⠀⣀⣀⣀⣀⣠⣾⣿⣿⣿⣿⣿⣿⣿⡟⠀",
    "⠀⠀⠻⣿⣿⣿⣿⣿⣿⣿⣄⣰⣿⠁⢀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠟⠀⠀",
    "⠀⠀⠀⠙⢿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⠋⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠙⠻⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠟⠋⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠉⠛⠻⠿⢿⣿⣿⣿⣿⡿⠿⠟⠛⠉⠀⠀⠀⠀⠀⠀⠀⠀",
];

struct TuiNode {
    parser: Arc<Mutex<Parser>>,
    pty_pair: portable_pty::PtyPair,
    byte_count: Arc<AtomicUsize>,
}

impl TuiNode {
    fn new(width: u16, height: u16) -> Self {
        let pty_system = native_pty_system();
        let pty_pair = pty_system
            .openpty(PtySize { rows: height, cols: width, pixel_width: 0, pixel_height: 0 })
            .expect("failed to open pty");

        Self {
            parser: Arc::new(Mutex::new(Parser::new(height, width, 100))),
            pty_pair,
            byte_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn spawn(&self, args: Vec<String>, cwd: PathBuf) -> io::Result<()> {
        let mut cmd = CommandBuilder::new("cargo");
        cmd.args(["run", "--bin", "gnostr-bitcoin", "--"]);
        cmd.args(args);
        cmd.cwd(cwd); 
        cmd.env("TERM", "xterm-256color");
        cmd.env("COLORTERM", "truecolor");

        let _child = self.pty_pair.slave.spawn_command(cmd).expect("failed to spawn command");
        let mut reader = self.pty_pair.master.try_clone_reader().expect("failed to clone reader");
        let parser = Arc::clone(&self.parser);
        let byte_count = Arc::clone(&self.byte_count);

        std::thread::spawn(move || {
            let mut buf = [0u8; 8192];
            while let Ok(n) = reader.read(&mut buf) {
                if n == 0 { break; }
                byte_count.fetch_add(n, Ordering::SeqCst);
                let mut p = parser.lock().unwrap();
                p.process(&buf[..n]);
            }
        });
        Ok(())
    }

    fn resize(&self, w: u16, h: u16) {
        let mut p = self.parser.lock().unwrap();
        if p.screen().size() != (h, w) {
            p.set_size(h, w);
            let _ = self.pty_pair.master.resize(PtySize { rows: h, cols: w, pixel_width: 0, pixel_height: 0 });
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;

    let nodes = vec![TuiNode::new(120, 24), TuiNode::new(120, 24)];
    let project_root = std::env::current_dir()?;

    for (i, node) in nodes.iter().enumerate() {
        let mut args = vec!["--datadir".into(), format!("./test_data_{}", i + 1)];
        if i == 1 { args.extend(vec!["--target-peer-addr".into(), "127.0.0.1:8333".into()]); }
        node.spawn(args, project_root.clone())?;
    }

    let start_time = Instant::now();
    let min_splash_duration = Duration::from_secs(5);
    let byte_threshold = 2500; // Ensuring build logs finish

    loop {
        terminal.draw(|f| {
            let area = f.area();
            let all_ready = nodes.iter().all(|n| n.byte_count.load(Ordering::SeqCst) > byte_threshold) 
                            && start_time.elapsed() > min_splash_duration;

            if all_ready {
                // DASHBOARD VIEW
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Percentage(48),
                        Constraint::Min(2),
                        Constraint::Percentage(48)
                    ])
                    .split(area);

                for (idx, &chunk_idx) in [0, 2].iter().enumerate() {
                    let chunk = chunks[chunk_idx];
                    nodes[idx].resize(chunk.width, chunk.height);
                    
                    let p = nodes[idx].parser.lock().unwrap();
                    let screen = p.screen();
                    let mut lines = Vec::new();
                    for row in 0..screen.size().0 {
                        let mut spans = Vec::new();
                        for col in 0..screen.size().1 {
                            if let Some(cell) = screen.cell(row, col) {
                                spans.push(Span::styled(
                                    cell.contents().to_string(),
                                    Style::default()
                                        .fg(map_vt_color(cell.fgcolor()))
                                        .bg(map_vt_color(cell.bgcolor()))
                                ));
                            }
                        }
                        lines.push(Line::from(spans));
                    }
                    // This Paragraph fills the full width of the 'chunk'
                    f.render_widget(Paragraph::new(lines), chunk);
                }
            } else {
                // SPLASH VIEW
                f.render_widget(Clear, area);
                
                let vertical_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Min(0),
                        Constraint::Length(15),
                        Constraint::Length(2),
                        Constraint::Length(1),
                        Constraint::Min(0),
                    ])
                    .split(area);

                // Apply Orange Style to the Logo
                let logo_lines: Vec<Line> = BITCOIN_LOGO.iter()
                    .map(|&l| Line::from(Span::styled(l, Style::default().fg(BITCOIN_ORANGE))))
                    .collect();

                f.render_widget(Paragraph::new(logo_lines).alignment(Alignment::Center), vertical_chunks[1]);
                f.render_widget(
                    Paragraph::new("INITIALIZING GNOSTR...")
                        .style(Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD))
                        .alignment(Alignment::Center),
                    vertical_chunks[3]
                );
            }
        })?;

        if event::poll(Duration::from_millis(33))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') { break; }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}

fn map_vt_color(c: vt100::Color) -> Color {
    match c {
        vt100::Color::Default => Color::Reset,
        vt100::Color::Idx(i) => Color::Indexed(i),
        vt100::Color::Rgb(r, g, b) => Color::Rgb(r, g, b),
    }
}
