use clap::Parser;
use gnostr_asyncgit::gitui::{cli::Args, gitui_error::Error, term, Res};
use log::LevelFilter;
use ratatui::Terminal;
use std::{backtrace::Backtrace, panic};

pub fn main() -> Res<()> {
    let args = Args::parse();

    if args.version {
        // Setting cargo_suffix enables falling back to Cargo.toml for version
        // `cargo install --locked gnostr-asyncgit` would fail otherwise, as there's no git repo
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

    // Prevents cursor flash when opening gnostr-asyncgit
    terminal.hide_cursor().map_err(Error::Term)?;
    terminal.clear().map_err(Error::Term)?;

    log::debug!("Starting app");
    gnostr_asyncgit::gitui::run(args, &mut terminal)
}
