use clap::Parser;
use gnostr_asyncgit::gitui::{cli::Args, gitui_error::Error, term, Res, state, config};
use log::LevelFilter;
use ratatui::Terminal;
use std::{backtrace::Backtrace, panic, rc::Rc, time::{Duration, Instant}};
use crossterm::event::{self, Event, KeyCode};

pub fn main() -> Res<()> {
    let args = Args::parse();

    if args.version {
        println!(
            "gnostr-asyncgit {}",
            git_version::git_version!(cargo_suffix = "")
        );
        return Ok(());
    }

    if args.log {
        simple_logging::log_to_file(gnostr_asyncgit::gitui::LOG_FILE_NAME, LevelFilter::Debug)
            .map_err(Error::OpenLogFile)?;
    }

    panic::set_hook(Box::new(|panic_info| {
        term::cleanup_alternate_screen();
        term::cleanup_raw_mode();

        eprintln!("{panic_info}");
        eprintln!("trace: \n{}", Backtrace::force_capture());
    }));

    if args.print {
        setup_term_and_run(&args)?;
    } else {
        term::alternate_screen(|| term::raw_mode(|| setup_term_and_run(&args)))?
    }

    Ok(())
}

fn setup_term_and_run(args: &Args) -> Res<()> {
    log::debug!("Initializing terminal backend");
    let mut terminal = Terminal::new(term::backend()).map_err(Error::Term)?;

    terminal.hide_cursor().map_err(Error::Term)?;
    terminal.clear().map_err(Error::Term)?;

    log::debug!("Starting app with custom loop");
    
    // Manual implementation of gnostr_asyncgit::gitui::run to inject Double-ESC logic
    let dir = std::env::current_dir().map_err(|e| Error::FindGitDir(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    let repo = git2::Repository::discover(dir).map_err(Error::OpenRepo)?;
    let config = Rc::new(config::init_config()?);

    let mut state = state::State::create(
        Rc::new(repo),
        terminal.size().map_err(Error::Term)?,
        args,
        config.clone(),
        true,
    )?;

    state.redraw_now(&mut terminal)?;

    let mut last_esc_time: Option<Instant> = None;

    while !state.quit {
        if terminal.backend_mut().poll_event(Duration::from_millis(100))? {
            let event = terminal.backend_mut().read_event()?;
            
            match event {
                Event::Key(key) if key.code == KeyCode::Esc => {
                    if let Some(time) = last_esc_time {
                        if time.elapsed() < Duration::from_millis(500) {
                            // On double ESC, trigger a full redraw instead of quitting
                            state.stage_redraw();
                            last_esc_time = None;
                        } else {
                            last_esc_time = Some(Instant::now());
                            state.handle_event(&mut terminal, event)?;
                        }
                    } else {
                        last_esc_time = Some(Instant::now());
                        state.handle_event(&mut terminal, event)?;
                    }
                }
                _ => {
                    last_esc_time = None;
                    state.handle_event(&mut terminal, event)?;
                }
            }
        }
        state.update(&mut terminal)?;
    }

    Ok(())
}
