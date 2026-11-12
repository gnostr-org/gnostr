/*
 * Copyright (c) 2026 Geekspeaker Inc. All Rights Reserved.
 *
 * This software is "Source Available".
 * You may use and modify it for personal use.
 * Redistribution of modified versions is prohibited.
 *
 * See the LICENSE file for more details.
 */

use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

pub mod browser;
pub mod network;
pub mod renderer;

use browser::{ui, BrowserApp, SearchEngine};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Optional URL to open on startup
    url: Option<String>,

    /// Use Google Search
    #[arg(long, group = "search_engine")]
    google: bool,

    /// Use DuckDuckGo Search (Default)
    #[arg(long, group = "search_engine")]
    duckduck: bool,

    /// Use Bing Search
    #[arg(long, group = "search_engine")]
    bing: bool,

    /// Use Brave Search
    #[arg(long, group = "search_engine")]
    brave: bool,

    /// Use Swisscows Search
    #[arg(long, group = "search_engine")]
    swisscows: bool,

    /// Use Qwant Search
    #[arg(long, group = "search_engine")]
    qwant: bool,

    /// Force use of static homepage (no network fetch on startup)
    #[arg(long)]
    static_homepage: bool,
}

pub async fn run_with(
    engine: SearchEngine,
    url: Option<String>,
    static_homepage: bool,
) -> Result<(), anyhow::Error> {
    // Setup Terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create App
    let mut app = BrowserApp::new(engine);
    app.load_bookmarks();

    // Initial Load
    app.load_initial_url(url, static_homepage).await;

    // Main Loop
    loop {
        terminal.draw(|f| ui(f, &app))?;

        let should_quit = app
            .run_step(terminal.size()?.width, terminal.size()?.height)
            .await?;
        if should_quit {
            break;
        }
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

pub async fn run() -> Result<(), anyhow::Error> {
    let args = Args::parse();

    // Determine default engine
    let engine = if args.google {
        SearchEngine::Google
    } else if args.bing {
        SearchEngine::Bing
    } else if args.brave {
        SearchEngine::Brave
    } else if args.swisscows {
        SearchEngine::Swisscows
    } else if args.qwant {
        SearchEngine::Qwant
    } else {
        SearchEngine::DuckDuckGo
    };

    run_with(engine, args.url, args.static_homepage).await
}

/// Run the terminal browser pointed at a specific URL (used by `--tui`).
pub async fn run_url(url: Option<String>) -> Result<(), anyhow::Error> {
    run_with(SearchEngine::DuckDuckGo, url, false).await
}
