use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Tabs},
    Terminal,
};
use std::{
    io::{self, Read, Write},
    path::PathBuf,
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};
use vt100::Parser;

// Standard Gnostr Purple: #960096
fn gnostr_purple() -> Color {
    if let Ok(colorterm) = std::env::var("COLORTERM") {
        if colorterm == "truecolor" || colorterm == "24bit" {
            return Color::Rgb(150, 0, 150);
        }
    }
    // Fallback for terminals that don't advertise truecolor support
    Color::Magenta
}

fn gnostr_server_available() -> bool {
    which::which("gnostr-server").is_ok()
}

fn git_tui_available() -> bool {
    which::which("git-tui").is_ok()
}

fn spawn_gnostr_server(project_root: PathBuf) -> io::Result<bool> {
    if !gnostr_server_available() {
        return Ok(false);
    }

    let mut command = Command::new("gnostr-server");

    command
        .current_dir(project_root)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    command.spawn().map(|_| true)
}

fn ensure_git_tui_available() -> io::Result<bool> {
    if git_tui_available() {
        return Ok(true);
    }

    let status = Command::new("cargo").args(["install", "gnostr"]).status()?;
    Ok(status.success() && git_tui_available())
}

fn server_install_dialog() -> Vec<Line<'static>> {
    vec![
        Line::from(vec![Span::styled(
            "GNOSTR SERVER NOT INSTALLED",
            Style::default().fg(gnostr_purple()).add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from("Install the server binary, then restart the dashboard:"),
        Line::from("  cargo build --bin gnostr-server"),
        Line::from("  cargo install --path /absolute/path/to/gnostr --bin gnostr-server"),
        Line::from(""),
        Line::from("Replace /absolute/path/to/gnostr with your checkout path."),
        Line::from("Then make sure `gnostr-server` is on your PATH."),
    ]
}

const GNOSTR_ICON_TINY: [&str; 7] = [
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⣀⣤⣶⣶⣦⣄⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⢀⣠⣴⣶⣌⡙⠻⠿⢿⣿⣿⣷⣦⣀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⣀⣴⣾⣿⣿⣿⣿⣿⣷⣤⠀⣀⠛⠿⣿⣿⣿⣿⣶⣤⣀⠀⠀",
    "⠰⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⣿⣿⣆⠀⠀⢉⣿⣿⣿⣿⡷⠆",
    "⠀⠀⠉⠛⢿⣿⣿⣿⣿⣿⡿⠛⠀⠛⣿⣿⣿⣿⣿⣿⠿⠛⠉⠀⠀",
    "⠀⠀⠀⠀⠀⠈⠙⠻⢿⣿⣿⣶⣶⣶⣿⣿⡿⠟⠋⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠉⠛⠿⠿⠟⠋⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀",
];

const GNOSTR_ICON: [&str; 28] = [
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣤⣤⣤⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣤⣿⣿⣿⣿⣿⣤⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣤⣿⣿⣿⣿⣿⣿⣿⣿⣿⣤⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢠⡄⠀⠀⢻⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣧⡄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢠⣼⣿⣿⣧⠀⠀⠘⢻⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣧⡄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣼⣿⣿⣿⣿⣿⣿⣤⠀⠀⠘⢻⠛⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣤⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣤⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡄⠀⠀⠀⠀⠀⢻⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣤⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⣤⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡄⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣤⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⢠⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣧⡄⠀⠀⠀⠀⠀⢻⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣧⡄⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⢠⣼⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡇⠀⠀⣤⡄⠀⠘⢻⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣧⡄⠀⠀⠀⠀",
    "⠀⠀⢠⣼⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡇⠀⠀⣿⣿⣤⠀⠀⠘⢻⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣧⠀⠀⠀",
    "⠀⣤⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡇⠀⠀⣿⣿⣿⣿⣤⠀⠀⠀⠀⠘⠛⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣤⠀",
    "⢸⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡇⠀⠀⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠘⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡇",
    "⠘⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡇⠀⠀⣿⣿⣿⣿⣿⡄⠀⠀⠀⠀⠀⢠⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠃",
    "⠀⠀⠛⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡇⠀⠀⣿⣿⣿⣿⣿⣿⣤⣤⣤⣤⣼⣿⣿⣿⣿⣿⣿⣿⣿⣿⡟⠀⠀",
    "⠀⠀⠀⠘⢻⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡇⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡟⠃⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠘⢻⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠃⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡟⠃⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠘⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡟⠀⠀⠀⠀⠀⠘⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠛⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠛⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡄⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠛⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠛⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡄⠀⠀⠀⢠⣼⣿⣿⣿⣿⣿⣿⣿⣿⣿⠛⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢻⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡟⠃⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠘⢻⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡟⠃⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠘⢻⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠃⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠛⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠛⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠛⣿⣿⣿⣿⣿⣿⣿⠛⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠛⣿⣿⣿⠛⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
];

const GNOSTR_ICON_LARGE: [&str; 55] = [
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⣿⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⣿⣿⣿⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀",
    "⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀",
    "⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀",
    "⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀",
    "⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀",
    "⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀",
    "⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
];

pub struct TuiNode {
    parser: Arc<Mutex<Parser>>,
    master: Option<Arc<Mutex<Box<dyn portable_pty::MasterPty + Send>>>>,
    writer: Option<Arc<Mutex<Box<dyn std::io::Write + Send>>>>,
    slave: Option<Box<dyn portable_pty::SlavePty + Send>>,
    byte_count: Arc<AtomicUsize>,
    gnostr_presented: Arc<AtomicBool>,
    is_tui: Arc<AtomicBool>,
}

impl TuiNode {
    pub fn new(width: u16, height: u16) -> Self {
        let width = width.max(1);
        let height = height.max(1);
        let parser = Arc::new(Mutex::new(Parser::new(height, width, 100)));
        let pty_system = native_pty_system();
        let pty_pair = match pty_system.openpty(PtySize {
            rows: height,
            cols: width,
            pixel_width: 0,
            pixel_height: 0,
        }) {
            Ok(pair) => pair,
            Err(e) => {
                eprintln!("dashboard: falling back to a no-op node because pty allocation failed: {e}");
                return Self {
                    parser,
                    master: None,
                    writer: None,
                    slave: None,
                    byte_count: Arc::new(AtomicUsize::new(0)),
                    gnostr_presented: Arc::new(AtomicBool::new(false)),
                    is_tui: Arc::new(AtomicBool::new(false)),
                };
            }
        };

        let writer = match pty_pair.master.take_writer() {
            Ok(writer) => writer,
            Err(e) => {
                eprintln!(
                    "dashboard: falling back to a no-op node because pty writer allocation failed: {e}"
                );
                return Self {
                    parser,
                    master: None,
                    writer: None,
                    slave: None,
                    byte_count: Arc::new(AtomicUsize::new(0)),
                    gnostr_presented: Arc::new(AtomicBool::new(false)),
                    is_tui: Arc::new(AtomicBool::new(false)),
                };
            }
        };

        Self {
            parser,
            writer: Some(Arc::new(Mutex::new(writer))),
            master: Some(Arc::new(Mutex::new(pty_pair.master))),
            slave: Some(pty_pair.slave),
            byte_count: Arc::new(AtomicUsize::new(0)),
            gnostr_presented: Arc::new(AtomicBool::new(false)),
            is_tui: Arc::new(AtomicBool::new(false)),
        }
    }

///// the recusive loop must start here
    pub fn spawn(&self, args: Vec<String>, cwd: PathBuf, command_override: Option<String>) -> io::Result<()> {
        let Some(slave) = &self.slave else {
            return Ok(());
        };
        let Some(master) = &self.master else {
            return Ok(());
        };
        let mut cmd = if let Some(cmd_str) = command_override {
            let parts: Vec<&str> = cmd_str.split_whitespace().collect();
            if parts.is_empty() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "empty dashboard command override",
                ));
            } else {
                let mut cb = CommandBuilder::new(parts[0]);
                cb.args(&parts[1..]);
                cb
            }
        } else {
            if args.is_empty() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "dashboard spawn requires an explicit command",
                ));
            }
            let mut cb = CommandBuilder::new(&args[0]);
            cb.args(&args[1..]);
            cb
        };
        
        cmd.cwd(cwd);
        cmd.env("TERM", "xterm-256color");
        cmd.env("COLORTERM", "truecolor");

        // Avoid recursive dashboard relaunches; always spawn an explicit child command.
        let mut _child = slave
            .spawn_command(cmd)
            .expect("failed to spawn command");
        
        let mut reader = {
            let master = master.lock().unwrap();
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
        let w = w.max(1);
        let h = h.max(1);
        let mut p = self.parser.lock().unwrap();
        if p.screen().size() != (h, w) || force {
            p.set_size(h, w);
            if let Some(master) = &self.master {
                let master = master.lock().unwrap();
                let _ = master.resize(PtySize {
                    rows: h,
                    cols: w,
                    pixel_width: 0,
                    pixel_height: 0,
                });
            }
        }
    }

    pub fn write_input(&self, input: &[u8]) -> io::Result<()> {
        if let Some(writer) = &self.writer {
            let mut writer = writer.lock().unwrap();
            writer.write_all(input)
        } else {
            Ok(())
        }
    }
}

pub async fn run_dashboard(mut commands: Vec<String>) -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;

    let node_commands: Vec<Option<String>> = if commands.is_empty() {
        vec![Some("gnostr-tui".to_string())]
    } else {
        commands.drain(..).map(Some).collect()
    };

    let mut nodes = Vec::new();
    for _ in 0..node_commands.len() {
        nodes.push(TuiNode::new(120, 24));
    }
    
    let git_tui_node = TuiNode::new(120, 24);
    let relay_node = TuiNode::new(120, 24);
    let chat_node = TuiNode::new(120, 24);
    #[cfg(feature = "blossom-tui")]
    let server_node = TuiNode::new(120, 24);
    #[cfg(not(feature = "blossom-tui"))]
    let server_node = TuiNode::new(1, 1);
    let project_root = std::env::current_dir()?;
    let server_available = spawn_gnostr_server(project_root.clone())?;
    let mut git_tui_started = false;
    let mut git_tui_error: Option<String> = None;

    for (i, node) in nodes.iter().enumerate() {
        if let Some(cmd_override) = node_commands.get(i).and_then(|cmd| cmd.clone()) {
            node.spawn(vec![], project_root.clone(), Some(cmd_override))?;
        }
    }
    
    relay_node.spawn(vec![], project_root.clone(), Some("gnostr relay".to_string()))?;
    chat_node.spawn(vec![], project_root.clone(), Some("gnostr chat".to_string()))?;
    #[cfg(feature = "blossom-tui")]
    if server_available {
        server_node.spawn(
            vec![],
            project_root.clone(),
            Some("gnostr-server".to_string()),
        )?;
    }

    let start_time = Instant::now();
    let mut ready_since: Option<Instant> = None;
    let mut active_node: Option<usize> = Some(0);
    let mut selected_node: usize = 0;
    let mut last_esc_time: Option<Instant> = None;
    let mut was_ready = false;
    let mut force_redraw = false;
    let mut layout_direction = Direction::Vertical;
    let mut visible_nodes = vec![true; nodes.len()];
    let mut initial_node_redraw = true;
    let mut active_tab: usize = 0;
    let mut last_active_tab: usize = active_tab;
    let mut tab_titles = vec!["Nodes", "Relay", "Chat"];
    #[cfg(feature = "blossom-tui")]
    tab_titles.push("Server");
    tab_titles.push("Help");
    let git_tui_tab_index = tab_titles.len();
    let help_tab_index = tab_titles.len() - 1;
    #[cfg(feature = "blossom-tui")]
    let server_tab_index = help_tab_index - 1;
    let mut is_git_tui_active = false;
    let mut is_relay_active = false;
    let mut is_chat_active = false;
    #[cfg(feature = "blossom-tui")]
    let mut is_server_active = false;
    #[cfg(not(feature = "blossom-tui"))]
    let mut is_server_active = false;
    #[cfg(feature = "blossom-tui")]
    let server_ready = server_node.byte_count.load(Ordering::SeqCst) > 0
        || start_time.elapsed() > Duration::from_secs(3);
    #[cfg(not(feature = "blossom-tui"))]
    let server_ready = true;
    #[cfg(not(feature = "blossom-tui"))]
    let server_tab_index = usize::MAX;

    loop {
        if active_tab != last_active_tab {
            force_redraw = true;
            initial_node_redraw = true;
            last_active_tab = active_tab;
        }

        if active_tab == git_tui_tab_index && !git_tui_started && git_tui_error.is_none() {
            match ensure_git_tui_available() {
                Ok(true) => match git_tui_node.spawn(vec![], project_root.clone(), Some("git-tui".to_string())) {
                    Ok(()) => {
                        git_tui_started = true;
                        force_redraw = true;
                    }
                    Err(err) => {
                        git_tui_error = Some(format!("Failed to start git-tui: {err}"));
                        force_redraw = true;
                    }
                },
                Ok(false) => {
                    git_tui_error = Some("git-tui is not on PATH. Run `cargo install gnostr` and try again.".to_string());
                    force_redraw = true;
                }
                Err(err) => {
                    git_tui_error = Some(format!("Failed to prepare git-tui: {err}"));
                    force_redraw = true;
                }
            }
        }

        if force_redraw {
            terminal.clear()?;
        }

        terminal.draw(|f| {
            let area = f.area();

            let currently_ready = nodes.iter().all(|n| {
                n.gnostr_presented.load(Ordering::SeqCst)
                    || (n.byte_count.load(Ordering::SeqCst) > 0 && start_time.elapsed() > Duration::from_secs(3))
            }) && (relay_node.byte_count.load(Ordering::SeqCst) > 0 || start_time.elapsed() > Duration::from_secs(3))
               && (chat_node.byte_count.load(Ordering::SeqCst) > 0 || start_time.elapsed() > Duration::from_secs(3))
               && server_ready;

            if currently_ready && ready_since.is_none() {
                ready_since = Some(Instant::now());
            }

            let all_ready = match ready_since {
                Some(t) => {
                    t.elapsed() > Duration::from_secs(1)
                        || start_time.elapsed() > Duration::from_secs(if cfg!(debug_assertions) { 60 } else { 10 })
                }
                None => start_time.elapsed() > Duration::from_secs(if cfg!(debug_assertions) { 60 } else { 10 }),
            };

            if all_ready && !was_ready {
                was_ready = true;
                force_redraw = true;
            }

            if !all_ready {
                f.render_widget(Clear, area);

                let (icon_to_use, icon_height) = if area.height >= 65 && area.width >= 100 {
                    (&GNOSTR_ICON_LARGE[..], 55)
                } else if area.height >= 35 {
                    (&GNOSTR_ICON[..], 28)
                } else {
                    (&GNOSTR_ICON_TINY[..], 7)
                };

                let vertical_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Min(0),
                        Constraint::Length(icon_height),
                        Constraint::Length(2),
                        Constraint::Length(1),
                        Constraint::Min(0),
                    ])
                    .split(area);

                let logo_lines: Vec<Line> = icon_to_use
                    .iter()
                    .map(|&l| Line::from(Span::styled(l, Style::default().fg(gnostr_purple()))))
                    .collect();

                f.render_widget(
                    Paragraph::new(logo_lines).alignment(Alignment::Center),
                    vertical_chunks[1],
                );
                f.render_widget(
                    Paragraph::new("GNOSTR")
                        .style(Style::default().fg(gnostr_purple()).add_modifier(Modifier::BOLD))
                        .alignment(Alignment::Center),
                    vertical_chunks[3],
                );
            } else {
                let main_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(1), // Header + Tab bar
                        Constraint::Min(0),    // Content
                    ])
                    .split(area);

                // Render Header & Tab Bar
                let header_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Length(3), // CIRCLED_G
                        Constraint::Min(0),    // Left Tabs
                        Constraint::Length(9), // Right Tab (GitUI)
                    ])
                    .split(main_chunks[0]);

                f.render_widget(
                    Paragraph::new(crate::strings::symbol::CIRCLED_G_STR)
                        .style(Style::default().fg(gnostr_purple()).add_modifier(Modifier::BOLD)),
                    header_chunks[0],
                );

                let tabs = Tabs::new(tab_titles.iter().map(|t| Line::from(*t)).collect::<Vec<_>>())
                    .block(Block::default().borders(Borders::NONE))
                    .select(if active_tab == git_tui_tab_index { 999 } else { active_tab })
                    .style(Style::default().fg(Color::Gray))
                    .highlight_style(Style::default().fg(gnostr_purple()).add_modifier(Modifier::BOLD))
                    .divider(Span::raw(" | "));
                
                f.render_widget(tabs, header_chunks[1]);

                let git_tab_style = if active_tab == git_tui_tab_index {
                    Style::default().fg(gnostr_purple()).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Gray)
                };
                
                f.render_widget(
                    Paragraph::new("GitUI [\\\\]").style(git_tab_style).alignment(Alignment::Right),
                    header_chunks[2],
                );

                let content_area = main_chunks[1].inner(Margin {
                    horizontal: 1,
                    vertical: 1,
                });
                let git_tui_area = Rect {
                    x: content_area.x,
                    y: content_area.y,
                    width: content_area.width.saturating_sub(20).max(1),
                    height: content_area.height.saturating_sub(10).max(1),
                };
                let right_border_area = if active_tab == git_tui_tab_index {
                    git_tui_area
                } else {
                    content_area
                };

                if active_tab == git_tui_tab_index {
                    if let Some(error) = &git_tui_error {
                        let block = Block::default()
                            .borders(Borders::NONE)
                            .title(" GitUI [UNAVAILABLE] ")
                            .border_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));
                        f.render_widget(
                            Paragraph::new(error.clone()).alignment(Alignment::Center).block(block),
                            git_tui_area,
                        );
                    } else if git_tui_started {
                        let git_tui_width = git_tui_area.width;
                        git_tui_node.resize(
                            git_tui_width,
                            git_tui_area.height,
                            force_redraw,
                        );

                        let p = git_tui_node.parser.lock().unwrap();
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

                        let block_style = if is_git_tui_active {
                            Style::default().fg(gnostr_purple()).add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::Gray)
                        };

                        let title = if is_git_tui_active {
                            " GitUI [ACTIVE - Double ESC to unfocus] "
                        } else {
                            " GitUI [SELECTED - Press Enter to focus] "
                        };
                        let block = Block::default().borders(Borders::NONE).title(title).border_style(block_style);
                        f.render_widget(Paragraph::new(lines).block(block), git_tui_area);
                    } else {
                        let block = Block::default()
                            .borders(Borders::NONE)
                            .title(" GitUI ")
                            .border_style(Style::default().fg(Color::Gray));
                        f.render_widget(
                            Paragraph::new("Starting git-tui...").alignment(Alignment::Center).block(block),
                            git_tui_area,
                        );
                    }
                } else if active_tab == 0 { // Nodes Tab
                    let visible_indices: Vec<usize> = nodes.iter().enumerate()
                        .filter(|&(i, _)| visible_nodes[i])
                        .map(|(i, _)| i)
                        .collect();

                    if visible_indices.is_empty() {
                        f.render_widget(Paragraph::new("No nodes visible. Press 'q' to exit."), content_area);
                    } else {
                        let constraints: Vec<Constraint> = if let Some(active_idx) = active_node {
                            visible_indices.iter()
                                .map(|&idx| {
                                    if idx == active_idx {
                                        Constraint::Min(0)
                                    } else {
                                        Constraint::Length(3)
                                    }
                                })
                                .collect()
                        } else {
                            visible_indices.iter()
                                .map(|_| Constraint::Ratio(1, visible_indices.len() as u32))
                                .collect()
                        };

                        let chunks = Layout::default()
                            .direction(layout_direction)
                            .constraints(constraints)
                            .split(content_area);
                        let node_force_redraw = force_redraw || initial_node_redraw;

                        for (chunk_idx, &idx) in visible_indices.iter().enumerate() {
                            let chunk = chunks[chunk_idx];
                            nodes[idx].resize(chunk.width, chunk.height, node_force_redraw);

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
                                Style::default().fg(gnostr_purple()).add_modifier(Modifier::BOLD)
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
                                .borders(Borders::NONE)
                                .title(title)
                                .border_style(block_style);

                            f.render_widget(Paragraph::new(lines).block(block), chunk);
                        }
                    }
                } else if active_tab == 1 { // Relay Tab
                    relay_node.resize(content_area.width, content_area.height, force_redraw || initial_node_redraw);
                    
                    let p = relay_node.parser.lock().unwrap();
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
                    
                    let block_style = if is_relay_active {
                        Style::default().fg(gnostr_purple()).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Gray)
                    };
                    
                    let title = if is_relay_active { " Relay [ACTIVE - Double ESC to unfocus] " } else { " Relay [SELECTED - Press Enter to focus] " };
                    let block = Block::default().borders(Borders::NONE).title(title).border_style(block_style);
                    f.render_widget(Paragraph::new(lines).block(block), content_area);
                } else if active_tab == 2 { // Chat Tab
                    chat_node.resize(content_area.width, content_area.height, force_redraw || initial_node_redraw);
                    
                    let p = chat_node.parser.lock().unwrap();
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
                    
                    let block_style = if is_chat_active {
                        Style::default().fg(gnostr_purple()).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Gray)
                    };
                    
                    let title = if is_chat_active { " Chat [ACTIVE - Double ESC to unfocus] " } else { " Chat [SELECTED - Press Enter to focus] " };
                    let block = Block::default().borders(Borders::NONE).title(title).border_style(block_style);
                    f.render_widget(Paragraph::new(lines).block(block), content_area);
                } else if active_tab == server_tab_index && server_available {
                    server_node.resize(content_area.width, content_area.height, force_redraw || initial_node_redraw);

                    let p = server_node.parser.lock().unwrap();
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

                    let block_style = if is_server_active {
                        Style::default().fg(gnostr_purple()).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Gray)
                    };

                    let title = if is_server_active { " Server [ACTIVE - Double ESC to unfocus] " } else { " Server [SELECTED - Press Enter to focus] " };
                    let block = Block::default().borders(Borders::NONE).title(title).border_style(block_style);
                    f.render_widget(Paragraph::new(lines).block(block), content_area);
                } else if active_tab == server_tab_index && !server_available {
                    let install_lines = server_install_dialog();
                    f.render_widget(
                        Paragraph::new(install_lines).block(Block::default().borders(Borders::NONE).title(" Server ")),
                        content_area,
                    );
                } else if active_tab == help_tab_index { // Help Tab
                    let mut help_text = vec![
                        Line::from(vec![Span::styled(
                            "GNOSTR DASHBOARD HELP",
                            Style::default().fg(gnostr_purple()).add_modifier(Modifier::BOLD),
                        )]),
                        Line::from(""),
                        Line::from("Global Controls (when no node is focused):"),
                        Line::from("  [Tab]       : Cycle through tabs"),
                        Line::from("  [Up/Down]   : Select a node"),
                        Line::from("  [Left/Right]: Toggle horizontal/vertical layout (if at edge)"),
                        Line::from("  [Left/Right]: Select adjacent node (if not at edge)"),
                        Line::from("  [Enter]     : Focus the selected node"),
                        Line::from("  [1-9]       : Focus Node 1-9"),
                        Line::from("  [\\]         : Toggle the GitUI tab"),
                        Line::from("  [q]         : Hide selected node (Quits if only 1 node visible)"),
                        Line::from("  [.]         : Toggle Help Screen"),
                        Line::from("  [Ctrl+C]    : Force Quit Dashboard"),
                        Line::from(""),
                        Line::from("Node Controls (when a node is focused):"),
                        Line::from("  [Double ESC]: Unfocus the current node"),
                        Line::from("  [All]       : All other keys are forwarded to the node's PTY"),
                    ];
                    #[cfg(feature = "blossom-tui")]
                    help_text.insert(10, Line::from("  Server tab  : Available when blossom-tui is compiled in"));
                    if !server_available {
                        help_text.push(Line::from(""));
                        help_text.push(Line::from("Server tab opens an install dialog until gnostr-server is available."));
                    }
                    f.render_widget(Paragraph::new(help_text).block(Block::default().borders(Borders::ALL)), content_area);
                }

                f.render_widget(
                    Block::default()
                        .borders(Borders::RIGHT)
                        .border_style(Style::default().fg(Color::DarkGray)),
                    right_border_area,
                );
            }
        })?;
        force_redraw = false;
        initial_node_redraw = false;

        if event::poll(Duration::from_millis(33))? {
            match event::read()? {
                Event::Resize(_, _) => {
                    force_redraw = true;
                }
                Event::Key(key) => {
                    // Global Ctrl+C handler
                    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                        break;
                    }

                    if is_git_tui_active {
                        let input = encode_key(key);
                        if !input.is_empty() {
                            git_tui_node.write_input(&input)?;
                        }

                        let mut deactivated = false;
                        if key.code == KeyCode::Esc {
                            if let Some(time) = last_esc_time {
                                if time.elapsed() < Duration::from_millis(500) {
                                    is_git_tui_active = false;
                                    last_esc_time = None;
                                    deactivated = true;
                                    force_redraw = true;
                                } else {
                                    last_esc_time = Some(Instant::now());
                                }
                            } else {
                                last_esc_time = Some(Instant::now());
                            }
                        } else {
                            last_esc_time = None;
                        }
                    } else if is_relay_active {
                        let mut deactivated = false;
                        if key.code == KeyCode::Esc {
                            if let Some(time) = last_esc_time {
                                if time.elapsed() < Duration::from_millis(500) {
                                    is_relay_active = false;
                                    last_esc_time = None;
                                    deactivated = true;
                                    force_redraw = true;
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
                                relay_node.write_input(&input)?;
                            }
                        }
                    } else if is_chat_active {
                        let mut deactivated = false;
                        if key.code == KeyCode::Esc {
                            if let Some(time) = last_esc_time {
                                if time.elapsed() < Duration::from_millis(500) {
                                    is_chat_active = false;
                                    last_esc_time = None;
                                    deactivated = true;
                                    force_redraw = true;
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
                                chat_node.write_input(&input)?;
                            }
                        }
                    } else if is_server_active {
                        let mut deactivated = false;
                        if key.code == KeyCode::Esc {
                            if let Some(time) = last_esc_time {
                                if time.elapsed() < Duration::from_millis(500) {
                                    is_server_active = false;
                                    last_esc_time = None;
                                    deactivated = true;
                                    force_redraw = true;
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
                                server_node.write_input(&input)?;
                            }
                        }
                    } else if let Some(idx) = active_node {
                        let input = encode_key(key);
                        if !input.is_empty() {
                            nodes[idx].write_input(&input)?;
                        }

                        let mut deactivated = false;
                        if key.code == KeyCode::Esc {
                            if let Some(time) = last_esc_time {
                                if time.elapsed() < Duration::from_millis(500) {
                                    active_node = None;
                                    last_esc_time = None;
                                    deactivated = true;
                                    force_redraw = true; // Trigger full redraw on exit
                                } else {
                                    last_esc_time = Some(Instant::now());
                                }
                            } else {
                                last_esc_time = Some(Instant::now());
                            }
                        } else {
                            last_esc_time = None;
                        }
                    } else {
                        match key.code {
                            KeyCode::Char('\\') => {
                                active_tab = if active_tab == git_tui_tab_index { 0 } else { git_tui_tab_index };
                                force_redraw = true;
                            }
                            KeyCode::Tab => {
                                active_tab = (active_tab + 1) % tab_titles.len();
                                force_redraw = true;
                            }
                            KeyCode::Char('q') => {
                                let visible_count = visible_nodes.iter().filter(|&&v| v).count();
                                if visible_count <= 1 {
                                    break; // Quit if only 1 node is visible
                                } else {
                                    visible_nodes[selected_node] = false;
                                    // Move selection to next visible node
                                    loop {
                                        if selected_node < nodes.len().saturating_sub(1) {
                                            selected_node += 1;
                                        } else {
                                            selected_node = 0;
                                        }
                                        if visible_nodes[selected_node] {
                                            break;
                                        }
                                    }
                                    force_redraw = true;
                                }
                            }
                            KeyCode::Char(c) if c.is_ascii_digit() && c != '0' => {
                                let target_idx = (c as usize) - ('1' as usize);
                                if nodes.get(target_idx).is_some()
                                    && visible_nodes.get(target_idx).copied().unwrap_or(false)
                                {
                                    active_node = Some(target_idx);
                                    active_tab = 0;
                                }
                            }
                            KeyCode::Char('.') => {
                                active_tab = if active_tab == help_tab_index { 0 } else { help_tab_index };
                                force_redraw = true;
                            }
                            KeyCode::Up => {
                                if nodes.is_empty() {
                                    continue;
                                }
                                loop {
                                    if selected_node > 0 {
                                        selected_node -= 1;
                                    } else {
                                        selected_node = nodes.len().saturating_sub(1);
                                    }
                                    if visible_nodes[selected_node] { break; }
                                    if visible_nodes.iter().all(|&v| !v) { break; }
                                }
                            }
                            KeyCode::Down => {
                                if nodes.is_empty() {
                                    continue;
                                }
                                loop {
                                    if selected_node < nodes.len().saturating_sub(1) {
                                        selected_node += 1;
                                    } else {
                                        selected_node = 0;
                                    }
                                    if visible_nodes[selected_node] { break; }
                                    if visible_nodes.iter().all(|&v| !v) { break; }
                                }
                            }
                            KeyCode::Left => {
                                if nodes.is_empty() {
                                    continue;
                                }
                                let mut prev_idx = selected_node.checked_sub(1);
                                let mut found = false;
                                while let Some(idx) = prev_idx {
                                    if visible_nodes[idx] {
                                        selected_node = idx;
                                        found = true;
                                        break;
                                    }
                                    prev_idx = idx.checked_sub(1);
                                }
                                if !found {
                                    layout_direction = match layout_direction {
                                        Direction::Vertical => Direction::Horizontal,
                                        Direction::Horizontal => Direction::Vertical,
                                    };
                                    force_redraw = true;
                                }
                            }
                            KeyCode::Right => {
                                if nodes.is_empty() {
                                    continue;
                                }
                                let mut next_idx = selected_node + 1;
                                let mut found = false;
                                while next_idx < nodes.len() {
                                    if visible_nodes[next_idx] {
                                        selected_node = next_idx;
                                        found = true;
                                        break;
                                    }
                                    next_idx += 1;
                                }
                                if !found {
                                    layout_direction = match layout_direction {
                                        Direction::Vertical => Direction::Horizontal,
                                        Direction::Horizontal => Direction::Vertical,
                                    };
                                    force_redraw = true;
                                }
                            }
                            KeyCode::Enter => {
                                #[cfg(feature = "blossom-tui")]
                                {
                                    if active_tab == server_tab_index && server_available {
                                        is_server_active = true;
                                    } else if active_tab == server_tab_index && !server_available {
                                        force_redraw = true;
                                    } else if active_tab == git_tui_tab_index {
                                        is_git_tui_active = true;
                                    } else if active_tab == 2 {
                                        is_chat_active = true;
                                    } else if active_tab == 1 {
                                        is_relay_active = true;
                                    } else if active_tab == 0 && visible_nodes.get(selected_node).copied().unwrap_or(false) {
                                        active_node = Some(selected_node);
                                        active_tab = 0;
                                    }
                                }
                                #[cfg(not(feature = "blossom-tui"))]
                                {
                                    if active_tab == git_tui_tab_index {
                                        is_git_tui_active = true;
                                    } else if active_tab == 2 {
                                        is_chat_active = true;
                                    } else if active_tab == 1 {
                                        is_relay_active = true;
                                    } else if active_tab == 0 && visible_nodes.get(selected_node).copied().unwrap_or(false) {
                                        active_node = Some(selected_node);
                                        active_tab = 0;
                                    }
                                }
                            }
                            KeyCode::Esc => {
                                if active_tab != 0 {
                                    active_tab = 0;
                                    force_redraw = true;
                                }
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
