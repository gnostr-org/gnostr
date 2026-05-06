use clap::Parser;
use gnostr_asyncgit::tui::git::term::Term;
use gnostr_asyncgit::tui::git::{cli::Args, gitui_error::Error, term, Res};
use log::LevelFilter;
use simple_logger::SimpleLogger;
use std::{backtrace::Backtrace, panic};

pub fn main() -> Res<()> {
    let args = Args::parse();

    if args.version {
        // Setting cargo_suffix enables falling back to Cargo.toml for version
        // `cargo install --locked gnostr-asyncgit` would fail otherwise, as there's no git repo
        println!("gnostr-asyncgit {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    if args.log {
        SimpleLogger::new()
            .with_level(LevelFilter::Debug)
            .init()
            .map_err(Error::LoggerInit)?;
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
    let mut terminal: Term = Term::new(term::backend()).map_err(Error::Term)?;

    // Prevents cursor flash when opening gnostr-asyncgit
    terminal.hide_cursor().map_err(Error::Term)?;
    terminal.clear().map_err(Error::Term)?;

    log::debug!("Starting app");
    gnostr_asyncgit::tui::git::run(args, &mut terminal)
}
