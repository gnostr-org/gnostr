// src/bin/gnostr_dashboard.rs
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Terminal,
};
use std::{io, process::Stdio, sync::{Arc, Mutex}, time::{Duration, Instant}};
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use vt100::Parser;

struct TuiNode {
    parser: Arc<Mutex<Parser>>,
}

impl TuiNode {
    fn new() -> Self {
        // Initialize with a standard size; dynamic resizing happens in the loop
        Self {
            parser: Arc::new(Mutex::new(Parser::new(24, 80, 100))),
        }
    }

    async fn spawn(&self, args: Vec<String>) -> io::Result<tokio::process::Child> {
        let mut child = Command::new("cargo")
            .args(["run", "--bin", "gnostr-bitcoin", "--"])
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .env("TERM", "xterm-256color")
            .env("COLORTERM", "truecolor")
            .spawn()?;

        let mut stdout = child.stdout.take().unwrap();
        let parser = Arc::clone(&self.parser);

        tokio::spawn(async move {
            let mut buf = [0u8; 8192];
            while let Ok(n) = stdout.read(&mut buf).await {
                if n == 0 { break; }
                let mut p = parser.lock().unwrap();
                p.process(&buf[..n]);
            }
        });

        Ok(child)
    }

    fn resize(&self, width: u16, height: u16) {
        let mut p = self.parser.lock().unwrap();
        if p.screen().size() != (height, width) {
            p.set_size(height, width);
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    let nodes = vec![TuiNode::new(), TuiNode::new()];

    let mut children = Vec::new();
    for (i, node) in nodes.iter().enumerate() {
        let mut args = vec!["--datadir".into(), format!("./test_data_{}", i + 1)];
        if i == 0 { args.push("--listen".into()); }
        else { args.extend(vec!["--target-peer-addr".into(), "127.0.0.1:8333".into()]); }
        children.push(node.spawn(args).await?);
    }

    let tick_rate = Duration::from_millis(33);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(48), Constraint::Percentage(48)])
                .split(f.area());

            for (idx, area) in chunks.iter().enumerate() {
                // Now resizing to the FULL area width/height since borders are removed
                nodes[idx].resize(area.width, area.height);

                let p = nodes[idx].parser.lock().unwrap();
                let screen = p.screen();
                let mut lines = Vec::new();

                for row in 0..screen.size().0 {
                    let mut spans = Vec::new();
                    for col in 0..screen.size().1 {
                        if let Some(cell) = screen.cell(row, col) {
                            let style = Style::default()
                                .fg(match cell.fgcolor() {
                                    vt100::Color::Default => Color::Reset,
                                    vt100::Color::Idx(i) => Color::Indexed(i),
                                    vt100::Color::Rgb(r, g, b) => Color::Rgb(r, g, b),
                                })
                                .bg(match cell.bgcolor() {
                                    vt100::Color::Default => Color::Reset,
                                    vt100::Color::Idx(i) => Color::Indexed(i),
                                    vt100::Color::Rgb(r, g, b) => Color::Rgb(r, g, b),
                                });
                            spans.push(Span::styled(cell.contents().to_string(), style));
                        }
                    }
                    lines.push(Line::from(spans));
                }

                // Render the paragraph directly into the area with no Block wrapper
                f.render_widget(Paragraph::new(lines), *area);
            }
        })?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') { break; }
            }
        }
        if last_tick.elapsed() >= tick_rate { last_tick = Instant::now(); }
    }

    // Cleanup
    for mut child in children { let _ = child.kill().await; }
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}
