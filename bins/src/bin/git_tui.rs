use clap::Parser;
use gnostr_asyncgit::tui::git::{cli::Args, gitui_error::Error, term, Res};
use log::LevelFilter;
use git2::Repository;
use ratatui::Terminal;
use simple_logger::SimpleLogger;
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
        SimpleLogger::new()
            .with_level(LevelFilter::Debug)
            .init()
            .map_err(Error::LoggerInit)?;
    }

    if matches!(
        args.command,
        Some(gnostr_asyncgit::tui::git::cli::Commands::Notes)
    ) {
        inspect_notes()?;
        return Ok(());
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
    gnostr_asyncgit::tui::git::run(args, &mut terminal)
}

fn inspect_notes() -> Res<()> {
    let repo = Repository::open_from_env().map_err(Error::OpenRepo)?;
    let mut found = false;

    let notes_refs = repo.references_glob("refs/notes/*").map_err(Error::ListGitReferences)?;
    for reference in notes_refs {
        let reference = reference.map_err(Error::ListGitReferences)?;
        let Some(ref_name) = reference.name() else {
            continue;
        };

        found = true;
        println!("== {ref_name} ==");

        let notes = repo.notes(Some(ref_name)).map_err(Error::ReadGitConfig)?;
        for note in notes {
            let (_note_oid, object_oid) = note.map_err(Error::ReadOid)?;
            let note = repo.find_note(Some(ref_name), object_oid).map_err(Error::ReadOid)?;

            println!("-- object: {object_oid}");
            if let Some(message) = note.message() {
                print!("{message}");
                if !message.ends_with('\n') {
                    println!();
                }
            }
            println!();
        }
    }

    if !found {
        println!("No git notes refs found.");
    }

    Ok(())
}
