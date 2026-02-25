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
    is_tui: Arc<AtomicBool>,
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
            is_tui: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn spawn(&self, args: Vec<String>, cwd: PathBuf, command_override: Option<String>) -> io::Result<()> {
        let mut cmd = if let Some(cmd_str) = command_override {
            let parts: Vec<&str> = cmd_str.split_whitespace().collect();
            if parts.is_empty() {
                CommandBuilder::new("gnostr")
            } else {
                let mut cb = CommandBuilder::new(parts[0]);
                cb.args(&parts[1..]);
                cb
            }
        } else {
            let mut cb = CommandBuilder::new("gnostr");
            cb.args(args);
            cb
        };
        
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
        let is_tui = Arc::clone(&self.is_tui);

        std::thread::spawn(move || {
            let mut buf = [0u8; 8192];
            while let Ok(n) = reader.read(&mut buf) {
                if n == 0 {
                    break;
                }
                let chunk = &buf[..n];
                byte_count.fetch_add(n, Ordering::SeqCst);
                
                // Detect alternate screen buffer (CSI ? 1049 h) or hide cursor (CSI ? 25 l)
                if !is_tui.load(Ordering::SeqCst) {
                    if chunk.windows(8).any(|w| w == b"\x1b[?1049h" || w == b"\x1b[?1047h") 
                        || chunk.windows(6).any(|w| w == b"\x1b[?25l") {
                        is_tui.store(true, Ordering::SeqCst);
                    }
                }

                let mut p = parser.lock().unwrap();
                p.process(chunk);
                if !gnostr_presented.load(Ordering::SeqCst)
                    && p.screen().contents().to_lowercase().contains("gnostr")
                {
                    gnostr_presented.store(true, Ordering::SeqCst);
                }
            }
        });
        Ok(())
    }

    pub fn resize(&self, w: u16, h: u16, force: bool) {
        let mut p = self.parser.lock().unwrap();
        if p.screen().size() != (h, w) || force {
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

pub async fn run_dashboard(commands: Vec<String>) -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;

    let nodes = vec![TuiNode::new(120, 24), TuiNode::new(120, 24)];
    let project_root = std::env::current_dir()?;

    for (i, node) in nodes.iter().enumerate() {
        let cmd_override = commands.get(i).cloned();
        let args = if cmd_override.is_none() {
            vec!["--gitdir".into(), ".".into()]
        } else {
            vec![]
        };
        node.spawn(args, project_root.clone(), cmd_override)?;
    }

    let start_time = Instant::now();
    let mut ready_since: Option<Instant> = None;
    let mut active_node: Option<usize> = None;
    let mut selected_node: usize = 0;
    let mut show_help: bool = false;
    let mut last_esc_time: Option<Instant> = None;
    let mut was_ready = false;
    let mut force_redraw = false;

    loop {
        if force_redraw {
            terminal.clear()?;
        }

        terminal.draw(|f| {
            let area = f.area();

            let currently_ready = nodes.iter().all(|n| {
                n.gnostr_presented.load(Ordering::SeqCst) || (n.byte_count.load(Ordering::SeqCst) > 0 && start_time.elapsed() > Duration::from_secs(3))
            });
            
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

            if all_ready && !was_ready {
                was_ready = true;
                force_redraw = true; // Force a redraw on transition
            }

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
                    Line::from("  [Up/Down]: Select a node"),
                    Line::from("  [Enter]  : Focus the selected node"),
                    Line::from("  [1]      : Focus Node 1"),
                    Line::from("  [2]      : Focus Node 2"),
                    Line::from("  [.]      : Toggle Help Screen"),
                    Line::from("  [q]      : Quit Dashboard"),
                    Line::from(""),
                    Line::from("Node Controls (when a node is focused):"),
                    Line::from("  [Double ESC]: Unfocus the current node"),
                    Line::from("  [All]       : All other keys are forwarded to the node's PTY"),
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
                    nodes[idx].resize(chunk.width, chunk.height.saturating_sub(2), force_redraw);

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
                    } else if active_node.is_none() && selected_node == idx {
                        Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Gray)
                    };
                    
                    let is_tui = nodes[idx].is_tui.load(Ordering::SeqCst);
                    let type_str = if is_tui { "TUI" } else { "CLI" };

                    let title = format!(
                        " Node {} [{}] {} ",
                        idx + 1,
                        type_str,
                        if active_node == Some(idx) {
                            "[ACTIVE - Double ESC to unfocus]"
                        } else if active_node.is_none() && selected_node == idx {
                            "[SELECTED - Press Enter to focus]"
                        } else {
                            ""
                        }
                    );
                    let block = Block::default()
                        .borders(Borders::TOP | Borders::BOTTOM)
                        .title(title)
                        .border_style(block_style);

                    f.render_widget(Paragraph::new(lines).block(block), chunk);
                }
            }
        })?;
        force_redraw = false;

        if event::poll(Duration::from_millis(33))? {
            match event::read()? {
                Event::Resize(_, _) => {
                    force_redraw = true;
                }
                Event::Key(key) => {
                    if let Some(idx) = active_node {
                        let mut deactivated = false;
                        if key.code == KeyCode::Esc {
                            if let Some(time) = last_esc_time {
                                if time.elapsed() < Duration::from_millis(500) {
                                    active_node = None;
                                    last_esc_time = None;
                                    deactivated = true;
                                } else {
                                    last_esc_time = Some(Instant::now());
                                }
                            } else {
                                last_esc_time = Some(Instant::now());
                            }
                        } else {
                            last_esc_time = None;
                        }

                        if !deactivated {
                            let input = encode_key(key);
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
                            KeyCode::Up => {
                                if selected_node > 0 {
                                    selected_node -= 1;
                                } else {
                                    selected_node = nodes.len().saturating_sub(1);
                                }
                            }
                            KeyCode::Down => {
                                if selected_node < nodes.len().saturating_sub(1) {
                                    selected_node += 1;
                                } else {
                                    selected_node = 0;
                                }
                            }
                            KeyCode::Enter => {
                                active_node = Some(selected_node);
                            }
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

fn encode_key(key: event::KeyEvent) -> Vec<u8> {
    use crossterm::event::{KeyCode, KeyModifiers};
    let mut buf = Vec::new();
    
    if key.modifiers.contains(KeyModifiers::ALT) {
        buf.push(27); // ESC
    }

    match key.code {
        KeyCode::Char(c) => {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                let c = c.to_ascii_uppercase();
                if c >= '@' && c <= '_' {
                    buf.push((c as u8) - 64);
                } else if c == '?' {
                    buf.push(127); // DEL
                } else if c == ' ' {
                    buf.push(0); // Ctrl+Space
                } else {
                    let mut char_buf = [0; 4];
                    buf.extend_from_slice(c.encode_utf8(&mut char_buf).as_bytes());
                }
            } else {
                let mut char_buf = [0; 4];
                buf.extend_from_slice(c.encode_utf8(&mut char_buf).as_bytes());
            }
        }
        KeyCode::Backspace => buf.push(127), // 127 is standard for backspace in many terminals
        KeyCode::Enter => buf.push(b'\r'),
        KeyCode::Left => buf.extend_from_slice(b"\x1b[D"),
        KeyCode::Right => buf.extend_from_slice(b"\x1b[C"),
        KeyCode::Up => buf.extend_from_slice(b"\x1b[A"),
        KeyCode::Down => buf.extend_from_slice(b"\x1b[B"),
        KeyCode::Home => buf.extend_from_slice(b"\x1b[H"),
        KeyCode::End => buf.extend_from_slice(b"\x1b[F"),
        KeyCode::PageUp => buf.extend_from_slice(b"\x1b[5~"),
        KeyCode::PageDown => buf.extend_from_slice(b"\x1b[6~"),
        KeyCode::Tab => buf.push(b'\t'),
        KeyCode::BackTab => buf.extend_from_slice(b"\x1b[Z"),
        KeyCode::Delete => buf.extend_from_slice(b"\x1b[3~"),
        KeyCode::Insert => buf.extend_from_slice(b"\x1b[2~"),
        KeyCode::F(n) => {
            match n {
                1 => buf.extend_from_slice(b"\x1bOP"),
                2 => buf.extend_from_slice(b"\x1bOQ"),
                3 => buf.extend_from_slice(b"\x1bOR"),
                4 => buf.extend_from_slice(b"\x1bOS"),
                5 => buf.extend_from_slice(b"\x1b[15~"),
                6 => buf.extend_from_slice(b"\x1b[17~"),
                7 => buf.extend_from_slice(b"\x1b[18~"),
                8 => buf.extend_from_slice(b"\x1b[19~"),
                9 => buf.extend_from_slice(b"\x1b[20~"),
                10 => buf.extend_from_slice(b"\x1b[21~"),
                11 => buf.extend_from_slice(b"\x1b[23~"),
                12 => buf.extend_from_slice(b"\x1b[24~"),
                _ => {}
            }
        }
        KeyCode::Esc => buf.push(27),
        _ => {}
    }
    buf
}
