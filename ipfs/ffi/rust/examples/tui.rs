use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use kubo_ffi::{self, Node};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Terminal;
use std::io::stdout;
use std::time::{Duration, Instant};

fn main() -> std::io::Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let dir = std::env::temp_dir().join("kubo-rust-tui");
    let _ = std::fs::remove_dir_all(&dir);
    let repo = dir.to_string_lossy().to_string();

    let mut status = String::from("Initializing repo...");
    let _ = kubo_ffi::init_repo(&repo);
    let node = match Node::start(&repo, true) {
        Ok(n) => {
            status = format!("Node started: {}", n.peer_id().unwrap_or_default());
            Some(n)
        }
        Err(e) => {
            status = format!("Node start failed: {}", e);
            None
        }
    };

    let mut peers: Vec<String> = vec![];
    let mut logs = String::new();
    let mut last_peer_poll = Instant::now();
    let mut last_log_poll = Instant::now();

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(1),
                    Constraint::Length(8),
                ])
                .split(f.area());

            let title = Paragraph::new(status.clone()).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Kubo IPFS FFI - TUI"),
            );
            f.render_widget(title, chunks[0]);

            let peer_text = if peers.is_empty() {
                Text::from("No peers yet...".to_string())
            } else {
                Text::from(
                    peers
                        .iter()
                        .take(20)
                        .cloned()
                        .collect::<Vec<_>>()
                        .join("\n"),
                )
            };
            let peers_widget = Paragraph::new(peer_text).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Peers ({})", peers.len())),
            );
            f.render_widget(peers_widget, chunks[1]);

            let logs_widget = Paragraph::new(Text::from(logs.clone()))
                .block(Block::default().borders(Borders::ALL).title("Logs"))
                .style(Style::default().fg(Color::Green));
            f.render_widget(logs_widget, chunks[2]);
        })?;

        // poll logs
        if last_log_poll.elapsed() >= Duration::from_millis(500) {
            let chunk = kubo_ffi::get_logs();
            if !chunk.is_empty() {
                logs = format!("{}{}", chunk, logs);
                // keep last 2000 chars
                if logs.len() > 2000 {
                    logs = logs[logs.len() - 2000..].to_string();
                }
            }
            last_log_poll = Instant::now();
        }

        // poll peers
        if last_peer_poll.elapsed() >= Duration::from_secs(4) {
            if let Some(n) = &node {
                peers = n.swarm_peers();
                // dial bootstrap on first poll
                if peers.is_empty() {
                    let bs = "/dnsaddr/bootstrap.libp2p.io/p2p/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN";
                    let _ = n.connect(bs);
                }
            }
            last_peer_poll = Instant::now();
        }

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                    break;
                }
            }
        }
    }

    if let Some(n) = node {
        n.stop();
    }
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
