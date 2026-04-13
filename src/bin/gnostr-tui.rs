#![forbid(unsafe_code)]
#![deny(
	mismatched_lifetime_syntaxes,
	//unused_imports,
	unused_must_use,
	//dead_code,
	unstable_name_collisions,
	unused_assignments
)]
#![deny(clippy::all, clippy::perf, clippy::nursery, clippy::pedantic)]
#![deny(
	clippy::unwrap_used,
	clippy::filetype_is_file,
	clippy::cargo,
	clippy::panic,
	clippy::match_like_matches_macro
)]
#![allow(
	clippy::multiple_crate_versions,
	clippy::bool_to_int_with_if,
	clippy::module_name_repetitions,
	clippy::empty_docs,
	clippy::unnecessary_debug_formatting
)]

//TODO:
// #![deny(clippy::expect_used)]

type Terminal = ratatui::Terminal<CrosstermBackend<io::Stdout>>;

use gnostr::AsyncNotification;
use gnostr::QueueEvent;
use gnostr::{AsyncAppNotification, Updater};
use gnostr::{
	process_cmdline, App, CliArgs, Gitui, InputEvent, KeyConfig,
	QuitState, Theme,
};
use anyhow::anyhow;
use anyhow::{bail, Result};
use asyncgit::{sync::RepoPath, AsyncGitNotification};
use backtrace::Backtrace;
use crossbeam_channel::{Receiver, Select};
use crossterm::{
	terminal::{
		disable_raw_mode, enable_raw_mode, EnterAlternateScreen,
		LeaveAlternateScreen,
	},
	ExecutableCommand,
};
use ratatui::backend::CrosstermBackend;
use scopeguard::defer;
use std::{
	io::{self, Stdout},
	panic,
	path::Path,
	time::Instant,
};

/// Do `log::error!` and `eprintln!` in one line.¬                                                                   
macro_rules! log_eprintln {
	( $($arg:tt)* ) => {{
		log::error!($($arg)*);
		eprintln!($($arg)*);
	}};
}

fn main() -> Result<()> {
	let app_start = Instant::now();

	let cliargs = process_cmdline()?;

	asyncgit::register_tracing_logging();
	ensure_valid_path(&cliargs.repo_path)?;

	let key_config = KeyConfig::init(
		cliargs.key_bindings_path.as_ref(),
		cliargs.key_symbols_path.as_ref(),
	)
	.map_err(|e| log_eprintln!("KeyConfig loading error: {e}"))
	.unwrap_or_default();
	let theme = Theme::init(&cliargs.theme);

	setup_terminal()?;
	defer! {
		shutdown_terminal();
	}

	set_panic_handler()?;

	let mut terminal =
		start_terminal(io::stdout(), &cliargs.repo_path)?;

	let updater = if cliargs.notify_watcher {
		Updater::NotifyWatcher
	} else {
		Updater::Ticker
	};

	let mut args = cliargs;

	loop {
		let quit_state = run_app(
			app_start,
			args.clone(),
			theme.clone(),
			&key_config,
			updater,
			&mut terminal,
		)?;

		match quit_state {
			QuitState::OpenSubmodule(p) => {
				args = CliArgs {
					repo_path: p,
					select_file: None,
					theme: args.theme,
					notify_watcher: args.notify_watcher,
					key_bindings_path: args.key_bindings_path,
					key_symbols_path: args.key_symbols_path,
				}
			}
			_ => break,
		}
	}

	Ok(())
}

fn run_app(
	app_start: Instant,
	cliargs: CliArgs,
	theme: Theme,
	key_config: &KeyConfig,
	updater: Updater,
	terminal: &mut Terminal,
) -> Result<QuitState, anyhow::Error> {
	let mut gitui = Gitui::new(cliargs, theme, key_config, updater)?;

	log::trace!("app start: {} ms", app_start.elapsed().as_millis());

	gitui.run_main_loop(terminal)
}

fn setup_terminal() -> Result<()> {
	enable_raw_mode()?;
	io::stdout().execute(EnterAlternateScreen)?;
	Ok(())
}

fn shutdown_terminal() {
	let leave_screen =
		io::stdout().execute(LeaveAlternateScreen).map(|_f| ());

	if let Err(e) = leave_screen {
		log::error!("leave_screen failed:\n{e}");
	}

	let leave_raw_mode = disable_raw_mode();

	if let Err(e) = leave_raw_mode {
		log::error!("leave_raw_mode failed:\n{e}");
	}
}

fn ensure_valid_path(repo_path: &RepoPath) -> Result<()> {
	match asyncgit::sync::repo_open_error(repo_path) {
		Some(e) => {
			log::error!("invalid repo path: {e}");
			bail!("invalid repo path: {e}")
		}
		None => Ok(()),
	}
}

fn start_terminal(
	buf: Stdout,
	repo_path: &RepoPath,
) -> Result<Terminal> {
	let mut path = repo_path.gitpath().canonicalize()?;
	let home = dirs::home_dir().ok_or_else(|| {
		anyhow!("failed to find the home directory")
	})?;
	if path.starts_with(&home) {
		let relative_part = path
			.strip_prefix(&home)
			.expect("can't fail because of the if statement");
		path = Path::new("~").join(relative_part);
	}

	let mut backend = CrosstermBackend::new(buf);
	backend.execute(crossterm::terminal::SetTitle(format!(
		"gitui ({})",
		path.display()
	)))?;

	let mut terminal = Terminal::new(backend)?;
	terminal.hide_cursor()?;
	terminal.clear()?;

	Ok(terminal)
}

fn set_panic_handler() -> Result<()> {
	panic::set_hook(Box::new(|e| {
		let backtrace = Backtrace::new();
		shutdown_terminal();
		log_eprintln!("\nGitUI was closed due to an unexpected panic.\nPlease file an issue on https://github.com/gitui-org/gitui/issues with the following info:\n\n{e}\n\ntrace:\n{backtrace:?}");
	}));

	// global threadpool
	rayon_core::ThreadPoolBuilder::new()
		.num_threads(4)
		.build_global()?;

	Ok(())
}
