use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Terminal,
};
use std::{
    io::{self, Read, Write},
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex,
    },
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

pub struct TuiNode {
    parser: Arc<Mutex<Parser>>,
    master: Arc<Mutex<Box<dyn portable_pty::MasterPty + Send>>>,
    writer: Arc<Mutex<Box<dyn std::io::Write + Send>>>,
    slave: Box<dyn portable_pty::SlavePty + Send>,
    byte_count: Arc<AtomicUsize>,
    gnostr_presented: Arc<AtomicBool>,
}

impl TuiNode {
    pub fn new(width: u16, height: u16) -> Self {
        let pty_system = native_pty_system();
        let pty_pair = pty_system
            .openpty(PtySize {
                rows: height,
                cols: width,
                pixel_width: 0,
                pixel_height: 0,
            })
            .expect("failed to open pty");

        let writer = pty_pair.master.take_writer().expect("failed to take writer");

        Self {
            parser: Arc::new(Mutex::new(Parser::new(height, width, 100))),
            writer: Arc::new(Mutex::new(writer)),
            master: Arc::new(Mutex::new(pty_pair.master)),
            slave: pty_pair.slave,
            byte_count: Arc::new(AtomicUsize::new(0)),
            gnostr_presented: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn spawn(&self, args: Vec<String>, cwd: PathBuf) -> io::Result<()> {
        let mut cmd = CommandBuilder::new("gnostr");
        //cmd.args(["--gitdir", "."]);
        cmd.args(args);
        cmd.cwd(cwd);
        cmd.env("TERM", "xterm-256color");
        cmd.env("COLORTERM", "truecolor");

        let mut _child = self
            .slave
            .spawn_command(cmd)
            .expect("failed to spawn command");
        
        let mut reader = {
            let master = self.master.lock().unwrap();
            master.try_clone_reader().expect("failed to clone reader")
        };
        
        let parser = Arc::clone(&self.parser);
        let byte_count = Arc::clone(&self.byte_count);
        let gnostr_presented = Arc::clone(&self.gnostr_presented);

        std::thread::spawn(move || {
            let mut buf = [0u8; 8192];
            while let Ok(n) = reader.read(&mut buf) {
                if n == 0 {
                    break;
                }
                byte_count.fetch_add(n, Ordering::SeqCst);
                let mut p = parser.lock().unwrap();
                p.process(&buf[..n]);
                if !gnostr_presented.load(Ordering::SeqCst)
                    && p.screen().contents().to_lowercase().contains("gnostr")
                {
                    gnostr_presented.store(true, Ordering::SeqCst);
                }
            }
        });
        Ok(())
    }

    pub fn resize(&self, w: u16, h: u16) {
        let mut p = self.parser.lock().unwrap();
        if p.screen().size() != (h, w) {
            p.set_size(h, w);
            let master = self.master.lock().unwrap();
            let _ = master.resize(PtySize {
                rows: h,
                cols: w,
                pixel_width: 0,
                pixel_height: 0,
            });
        }
    }

    pub fn write_input(&self, input: &[u8]) -> io::Result<()> {
        let mut writer = self.writer.lock().unwrap();
        writer.write_all(input)
    }
}

pub async fn run_dashboard() -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;

    let nodes = vec![TuiNode::new(120, 24), TuiNode::new(120, 24)];
    let project_root = std::env::current_dir()?;

    for (i, node) in nodes.iter().enumerate() {
        let mut args = vec!["--gitdir".into(), ".".into()]; // format!("{}", i + 1)];
        if i == 1 {
            args.extend(vec![]);
        }
        node.spawn(args, project_root.clone())?;
    }

    let start_time = Instant::now();
    let mut ready_since: Option<Instant> = None;
    let mut active_node: Option<usize> = None;
    let mut show_help: bool = false;

    loop {
        terminal.draw(|f| {
            let area = f.area();

            let currently_ready = nodes
                .iter()
                .all(|n| n.gnostr_presented.load(Ordering::SeqCst));
            if currently_ready && ready_since.is_none() {
                ready_since = Some(Instant::now());
            }

            let all_ready = match ready_since {
                Some(t) => {
                    t.elapsed() > Duration::from_secs(1)
                        || start_time.elapsed() > Duration::from_secs(10)
                }
                None => start_time.elapsed() > Duration::from_secs(10), // Fallback
            };

            if !all_ready {
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

                let logo_lines: Vec<Line> = BITCOIN_LOGO
                    .iter()
                    .map(|&l| Line::from(Span::styled(l, Style::default().fg(BITCOIN_ORANGE))))
                    .collect();

                f.render_widget(
                    Paragraph::new(logo_lines).alignment(Alignment::Center),
                    vertical_chunks[1],
                );
                f.render_widget(
                    Paragraph::new("INITIALIZING GNOSTR...")
                        .style(Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD))
                        .alignment(Alignment::Center),
                    vertical_chunks[3],
                );
            } else if show_help {
                // HELP VIEW
                f.render_widget(Clear, area);

                let help_text = vec![
                    Line::from(vec![Span::styled(
                        "GNOSTR DASHBOARD HELP",
                        Style::default().fg(BITCOIN_ORANGE).add_modifier(Modifier::BOLD),
                    )]),
                    Line::from(""),
                    Line::from("Global Controls (when no node is focused):"),
                    Line::from("  [1]     : Focus Node 1"),
                    Line::from("  [2]     : Focus Node 2"),
                    Line::from("  [.]     : Toggle Help Screen"),
                    Line::from("  [q]     : Quit Dashboard"),
                    Line::from(""),
                    Line::from("Node Controls (when a node is focused):"),
                    Line::from("  [Ctrl+X]: Unfocus the current node"),
                    Line::from("  [All]   : All other keys are forwarded to the node's PTY"),
                    Line::from(""),
                    Line::from(vec![Span::styled(
                        "Press '.' or 'ESC' to return to the dashboard.",
                        Style::default().fg(Color::Gray),
                    )]),
                ];

                let block = Block::default()
                    .borders(Borders::ALL)
                    .title(" Help ")
                    .border_style(Style::default().fg(BITCOIN_ORANGE));

                f.render_widget(Paragraph::new(help_text).block(block).alignment(Alignment::Left), area);
            } else {
                // DASHBOARD VIEW
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Percentage(48),
                        Constraint::Min(2),
                        Constraint::Percentage(48),
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
                                        .bg(map_vt_color(cell.bgcolor())),
                                ));
                            }
                        }
                        lines.push(Line::from(spans));
                    }

                    let block_style = if active_node == Some(idx) {
                        Style::default().fg(BITCOIN_ORANGE).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Gray)
                    };

                    let title = format!(" Node {} {} ", idx + 1, if active_node == Some(idx) { "[ACTIVE - Press Ctrl+X to unfocus]" } else { "" });
                    let block = Block::default()
                        .borders(Borders::ALL)
                        .title(title)
                        .border_style(block_style);

                    f.render_widget(Paragraph::new(lines).block(block), chunk);
                }
            }
        })?;

        if event::poll(Duration::from_millis(33))? {
            match event::read()? {
                Event::Key(key) => {
                    if let Some(idx) = active_node {
                        if key.code == KeyCode::Char('x') && key.modifiers.contains(KeyModifiers::CONTROL) {
                            active_node = None;
                        } else {
                            // Basic key mapping for PTY input
                            let input = match key.code {
                                KeyCode::Char(c) => {
                                    if key.modifiers.contains(KeyModifiers::CONTROL) && c.is_ascii_alphabetic() {
                                        // Map Ctrl+letter to 1..26
                                        vec![(c as u8 & 0x1f)]
                                    } else {
                                        vec![c as u8]
                                    }
                                },
                                KeyCode::Esc => vec![27],
                                KeyCode::Enter => vec![b'\r'],
                                KeyCode::Backspace => vec![8],
                                KeyCode::Tab => vec![b'\t'],
                                KeyCode::Up => vec![27, 91, 65],
                                KeyCode::Down => vec![27, 91, 66],
                                KeyCode::Right => vec![27, 91, 67],
                                KeyCode::Left => vec![27, 91, 68],
                                _ => vec![],
                            };
                            if !input.is_empty() {
                                nodes[idx].write_input(&input)?;
                            }
                        }
                    } else if show_help {
                        match key.code {
                            KeyCode::Char('.') | KeyCode::Esc => show_help = false,
                            KeyCode::Char('q') => break,
                            _ => {}
                        }
                    } else {
                        match key.code {
                            KeyCode::Char('q') => break,
                            KeyCode::Char('1') => active_node = Some(0),
                            KeyCode::Char('2') => active_node = Some(1),
                            KeyCode::Char('.') => show_help = true,
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    Ok(())
}

fn map_vt_color(c: vt100::Color) -> Color {
    match c {
        vt100::Color::Default => Color::Reset,
        vt100::Color::Idx(i) => Color::Indexed(i),
        vt100::Color::Rgb(r, g, b) => Color::Rgb(r, g, b),
    }
}
