//! gnostr-blossom-tui binary entry point.

use std::io;

use gnostr_blossom_tui::{App, AppMsg, load_state, run_loop, save_state};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let saved = load_state();
    let (server, secret_key) = parse_args(&saved)?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (tx, mut rx) = mpsc::unbounded_channel::<AppMsg>();
    let mut app = App::new(server, secret_key, tx);
    // Restore UI preferences from the saved state (server/key already applied
    // by parse_args; only UI fields are applied here).
    app.apply_state(&saved);

    app.refresh_blobs();
    app.refresh_status();

    let result = run_loop(&mut terminal, &mut app, &mut rx).await;

    // Persist state before tearing down the terminal so any I/O errors are
    // visible in the restored terminal output.
    if let Err(e) = save_state(&app.to_state()) {
        eprintln!("gnostr-blossom-tui: failed to save state: {e}");
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn parse_args(
    saved: &gnostr_blossom_tui::TuiState,
) -> Result<(String, Option<String>), Box<dyn std::error::Error>> {
    // Priority: CLI arg > env var > saved state > compiled default.
    let mut server = saved
        .server
        .clone()
        .unwrap_or_else(|| "http://localhost:3000".into());
    // Env var overrides saved state.
    if let Ok(v) = std::env::var("BLOSSOM_SERVER") {
        server = v;
    }

    let mut secret_key: Option<String> = saved.secret_key.clone();
    if let Ok(v) = std::env::var("BLOSSOM_SECRET_KEY") {
        secret_key = Some(v);
    }

    // Explicit CLI flags take highest priority.
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1usize;
    while i < args.len() {
        match args[i].as_str() {
            "-s" | "--server" => {
                i += 1;
                if let Some(v) = args.get(i) {
                    server = v.clone();
                }
            }
            "-k" | "--key" => {
                i += 1;
                if let Some(v) = args.get(i) {
                    secret_key = Some(gnostr_blossom_tui::decode_secret_key(v)?);
                }
            }
            "-h" | "--help" => {
                print_usage();
                std::process::exit(0);
            }
            "-V" | "--version" => {
                println!("gnostr-blossom-tui {}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            _ => {}
        }
        i += 1;
    }
    Ok((server, secret_key))
}

fn print_usage() {
    println!("gnostr-blossom-tui — Terminal UI for Blossom blob storage\n");
    println!("USAGE:");
    println!("  gnostr-blossom-tui [OPTIONS]\n");
    println!("OPTIONS:");
    println!("  -s, --server <URL>   Blossom server URL [default: http://localhost:3000]");
    println!("  -k, --key <KEY>      Secret key (hex or nsec1 bech32)");
    println!("  -h, --help           Print this help");
    println!("  -V, --version        Print version info\n");
    println!("ENV:");
    println!("  BLOSSOM_SERVER       Server URL (fallback when --server not set)");
    println!("  BLOSSOM_SECRET_KEY   Secret key (fallback when --key not set)\n");
    println!("STATE:");
    println!("  ~/.config/blossom-tui/state.json  Persisted UI state (loaded on startup,");
    println!("                                    written on clean exit). Protect this file");
    println!("                                    if it contains a secret key (mode 0600).");
}
