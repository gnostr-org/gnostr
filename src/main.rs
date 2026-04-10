#![forbid(unsafe_code)]
#![deny(
	unused_imports,
	unused_must_use,
//	dead_code,
	unstable_name_collisions,
	unused_assignments
)]
#![deny(clippy::all, clippy::perf, clippy::nursery, clippy::pedantic)]
#![deny(
	clippy::unwrap_used,
	clippy::filetype_is_file,
	clippy::cargo,
	clippy::unwrap_used,
	clippy::panic,
	clippy::match_like_matches_macro
)]
#![allow(clippy::module_name_repetitions)]
#![allow(
	clippy::multiple_crate_versions,
	clippy::bool_to_int_with_if,
	clippy::module_name_repetitions
)]
// high number of false positives on nightly (as of Oct 2022 with 1.66.0-nightly)
#![allow(clippy::missing_const_for_fn)]

//TODO:
// #![deny(clippy::expect_used)]

mod app;
mod args;
mod bug_report;
mod clipboard;
mod cmdbar;
mod components;
mod input;
mod keys;
mod notify_mutex;
mod options;
mod popup_stack;
mod queue;
mod spinner;
mod string_utils;
mod strings;
mod tabs;
mod ui;
mod version;
mod watcher;

use crate::{app::App, args::process_cmdline};
use anyhow::{bail, Result};
use app::QuitState;
#[cfg(feature = "nostr")]
use asyncgit::nostr::AsyncNostrNotification;
use asyncgit::{
	sync::{utils::repo_work_dir, RepoPath},
	AsyncGitNotification,
};
use backtrace::Backtrace;
use crossbeam_channel::{never, tick, unbounded, Receiver, Select};
use crossterm::{
	terminal::{
		disable_raw_mode, enable_raw_mode, EnterAlternateScreen,
		LeaveAlternateScreen,
	},
	ExecutableCommand,
};
use input::{Input, InputEvent, InputState};
use keys::KeyConfig;
use ratatui::{
	backend::{Backend, CrosstermBackend},
	Terminal,
};
use scopeguard::defer;
use scopetime::scope_time;
use spinner::Spinner;
use std::{
	cell::RefCell,
	io::{self, Write},
	panic, process,
	time::{Duration, Instant},
};
use ui::style::Theme;
use watcher::RepoWatcher;

static TICK_INTERVAL: Duration = Duration::from_secs(5);
static SPINNER_INTERVAL: Duration = Duration::from_millis(80);

///
#[derive(Clone)]
pub enum QueueEvent {
	Tick,
	Notify,
	SpinnerUpdate,
	AsyncEvent(AsyncNotification),
	InputEvent(InputEvent),
	#[cfg(feature = "nostr")]
	NostrEvent(AsyncNostrNotification),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyntaxHighlightProgress {
	Progress,
	Done,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AsyncAppNotification {
	///
	SyntaxHighlighting(SyntaxHighlightProgress),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AsyncNotification {
	///
	App(AsyncAppNotification),
	///
	Git(AsyncGitNotification),
}

#[derive(Clone, Copy, PartialEq)]
enum Updater {
	Ticker,
	NotifyWatcher,
}

fn main() -> Result<()> {
	let app_start = Instant::now();

	let cliargs = process_cmdline()?;

	// --generate-nostr-key: print a fresh keypair and exit immediately.
	#[cfg(feature = "nostr")]
	if cliargs.nostr_generate {
		use asyncgit::nostr::generate_keypair_strings;
		let (nsec, npub) = generate_keypair_strings();
		println!("nsec: {nsec}");
		println!("npub: {npub}");
		println!();
		println!("Save the nsec somewhere safe.  Set it with:");
		println!("  git config nostr.key <nsec>");
		println!("  export NOSTR_KEY=<nsec>");
		println!("  gnostr-tui --key <nsec>");
		return Ok(());
	}

	asyncgit::register_tracing_logging();

	if !valid_path(&cliargs.repo_path) {
		//TODO: gnostr-cli init
		bail!("invalid path\nplease run gnostr-tui inside of a non-bare git repository");
	}

	let key_config = KeyConfig::init()
		.map_err(|e| eprintln!("KeyConfig loading error: {e}"))
		.unwrap_or_default();
	let theme = Theme::init(&cliargs.theme);

	setup_terminal()?;
	defer! {
		shutdown_terminal();
	}

	set_panic_handlers()?;

	let mut terminal = start_terminal(io::stdout())?;
	let mut repo_path = cliargs.repo_path;
	let input = Input::new();

	let updater = if cliargs.notify_watcher {
		Updater::NotifyWatcher
	} else {
		Updater::Ticker
	};

	loop {
		let quit_state = run_app(
			app_start,
			repo_path.clone(),
			theme.clone(),
			key_config.clone(),
			&input,
			updater,
			&mut terminal,
			#[cfg(feature = "nostr")]
			cliargs.nostr_key.clone(),
			#[cfg(feature = "nostr")]
			cliargs.nostr_relays.clone(),
		)?;

		match quit_state {
			QuitState::OpenSubmodule(p) => {
				repo_path = p;
			}
			_ => break,
		}
	}

	Ok(())
}

fn run_app(
	app_start: Instant,
	repo: RepoPath,
	theme: Theme,
	key_config: KeyConfig,
	input: &Input,
	updater: Updater,
	terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
	#[cfg(feature = "nostr")] nostr_key: Option<String>,
	#[cfg(feature = "nostr")] nostr_relays: Vec<String>,
) -> Result<QuitState, anyhow::Error> {
	let (tx_git, rx_git) = unbounded();
	let (tx_app, rx_app) = unbounded();
	#[cfg(feature = "nostr")]
	let (tx_nostr, rx_nostr) = unbounded::<AsyncNostrNotification>();

	let rx_input = input.receiver();

	let (rx_ticker, rx_watcher) = match updater {
		Updater::NotifyWatcher => {
			let repo_watcher =
				RepoWatcher::new(repo_work_dir(&repo)?.as_str());

			(never(), repo_watcher.receiver())
		}
		Updater::Ticker => (tick(TICK_INTERVAL), never()),
	};

	let spinner_ticker = tick(SPINNER_INTERVAL);

	let mut app = App::new(
		RefCell::new(repo),
		&tx_git,
		&tx_app,
		#[cfg(feature = "nostr")]
		tx_nostr,
		input.clone(),
		theme,
		key_config,
		#[cfg(feature = "nostr")]
		nostr_key,
		#[cfg(feature = "nostr")]
		nostr_relays,
	)?;

	let mut spinner = Spinner::default();
	let mut first_update = true;
	#[cfg(feature = "nostr")]
	let mut nostr_started = false;

	log::trace!("app start: {} ms", app_start.elapsed().as_millis());

	loop {
		let event = if first_update {
			first_update = false;
			QueueEvent::Notify
		} else {
			select_event(
				&rx_input,
				&rx_git,
				&rx_app,
				#[cfg(feature = "nostr")]
				&rx_nostr,
				&rx_ticker,
				&rx_watcher,
				&spinner_ticker,
			)?
		};

		{
			if matches!(event, QueueEvent::SpinnerUpdate) {
				spinner.update();
				spinner.draw(terminal)?;
				continue;
			}

			scope_time!("loop");

			match event {
				QueueEvent::InputEvent(ev) => {
					if matches!(
						ev,
						InputEvent::State(InputState::Polling)
					) {
						//Note: external ed closed, we need to re-hide cursor
						terminal.hide_cursor()?;
					}
					app.event(ev)?;
				}
				QueueEvent::Tick | QueueEvent::Notify => {
					app.update()?;
				}
				QueueEvent::AsyncEvent(ev) => {
					if !matches!(
						ev,
						AsyncNotification::Git(
							AsyncGitNotification::FinishUnchanged
						)
					) {
						app.update_async(ev)?;
					}
				}
				QueueEvent::SpinnerUpdate => unreachable!(),
				#[cfg(feature = "nostr")]
				QueueEvent::NostrEvent(ev) => {
					#[cfg(feature = "nostr")]
					app.update_nostr(ev);
				}
			}

			draw(terminal, &app)?;

			// Start the nostr background thread AFTER the first frame is
			// drawn so the terminal is fully initialised before any
			// background activity begins.
			#[cfg(feature = "nostr")]
			if !nostr_started {
				nostr_started = true;
				app.start_nostr();
			}

			spinner.set_state(app.any_work_pending());
			spinner.draw(terminal)?;

			if app.is_quit() {
				break;
			}
		}
	}

	Ok(app.quit_state())
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
		eprintln!("leave_screen failed:\n{e}");
	}

	let leave_raw_mode = disable_raw_mode();

	if let Err(e) = leave_raw_mode {
		eprintln!("leave_raw_mode failed:\n{e}");
	}
}

fn draw<B: Backend>(
	terminal: &mut Terminal<B>,
	app: &App,
) -> io::Result<()> {
	if app.requires_redraw() {
		terminal.resize(terminal.size()?)?;
	}

	terminal.draw(|f| {
		if let Err(e) = app.draw(f) {
			log::error!("failed to draw: {:?}", e);
		}
	})?;

	Ok(())
}

fn valid_path(repo_path: &RepoPath) -> bool {
	let error = asyncgit::sync::repo_open_error(repo_path);
	if let Some(error) = &error {
		eprintln!("repo open error: {error}");
	}
	error.is_none()
}

fn select_event(
	rx_input: &Receiver<InputEvent>,
	rx_git: &Receiver<AsyncGitNotification>,
	rx_app: &Receiver<AsyncAppNotification>,
	#[cfg(feature = "nostr")] rx_nostr: &Receiver<
		AsyncNostrNotification,
	>,
	rx_ticker: &Receiver<Instant>,
	rx_notify: &Receiver<()>,
	rx_spinner: &Receiver<Instant>,
) -> Result<QueueEvent> {
	let mut sel = Select::new();

	sel.recv(rx_input); // 0
	sel.recv(rx_git); // 1
	sel.recv(rx_app); // 2
	#[cfg(feature = "nostr")]
	sel.recv(rx_nostr); // 3  (nostr feature only)
	sel.recv(rx_ticker); // 3 / 4
	sel.recv(rx_notify); // 4 / 5
	sel.recv(rx_spinner); // 5 / 6

	let oper = sel.select();
	let index = oper.index();

	#[cfg(feature = "nostr")]
	let ev = match index {
		0 => oper.recv(rx_input).map(QueueEvent::InputEvent),
		1 => oper.recv(rx_git).map(|e| {
			QueueEvent::AsyncEvent(AsyncNotification::Git(e))
		}),
		2 => oper.recv(rx_app).map(|e| {
			QueueEvent::AsyncEvent(AsyncNotification::App(e))
		}),
		3 => oper.recv(rx_nostr).map(QueueEvent::NostrEvent),
		4 => oper.recv(rx_ticker).map(|_| QueueEvent::Notify),
		5 => oper.recv(rx_notify).map(|()| QueueEvent::Notify),
		6 => oper.recv(rx_spinner).map(|_| QueueEvent::SpinnerUpdate),
		_ => bail!("unknown select source"),
	}?;

	#[cfg(not(feature = "nostr"))]
	let ev = match index {
		0 => oper.recv(rx_input).map(QueueEvent::InputEvent),
		1 => oper.recv(rx_git).map(|e| {
			QueueEvent::AsyncEvent(AsyncNotification::Git(e))
		}),
		2 => oper.recv(rx_app).map(|e| {
			QueueEvent::AsyncEvent(AsyncNotification::App(e))
		}),
		3 => oper.recv(rx_ticker).map(|_| QueueEvent::Notify),
		4 => oper.recv(rx_notify).map(|()| QueueEvent::Notify),
		5 => oper.recv(rx_spinner).map(|_| QueueEvent::SpinnerUpdate),
		_ => bail!("unknown select source"),
	}?;

	Ok(ev)
}

fn start_terminal<W: Write>(
	buf: W,
) -> io::Result<Terminal<CrosstermBackend<W>>> {
	let backend = CrosstermBackend::new(buf);
	let mut terminal = Terminal::new(backend)?;
	terminal.hide_cursor()?;
	terminal.clear()?;

	Ok(terminal)
}

// do log::error! and eprintln! in one line, pass sting, error and backtrace
macro_rules! log_eprintln {
	($string:expr, $e:expr, $bt:expr) => {
		log::error!($string, $e, $bt);
		eprintln!($string, $e, $bt);
	};
}

fn set_panic_handlers() -> Result<()> {
	// regular panic handler
	// Only restore the terminal from the main thread.  Background threads
	// (asyncnostr, asyncgit, rayon) must NOT call shutdown_terminal() — doing
	// so from a non-main thread corrupts the terminal state while the TUI is
	// still running on the main thread.
	panic::set_hook(Box::new(|e| {
		let backtrace = Backtrace::new();
		let is_main_thread = matches!(
			std::thread::current().name(),
			None | Some("main")
		);
		if is_main_thread {
			// On the main thread we restore the terminal first so the
			// panic message is readable, then print to stderr.
			shutdown_terminal();
			log_eprintln!("panic: {:?}\ntrace:\n{:?}", e, backtrace);
		} else {
			// On background threads (asyncnostr, asyncgit, rayon) writing
			// to stderr would corrupt the TUI still running on the main
			// thread.  Log only.
			log::error!(
				"panic in thread '{}': {:?}\ntrace:\n{:?}",
				std::thread::current().name().unwrap_or("unnamed"),
				e,
				backtrace
			);
		}
	}));

	// global threadpool
	rayon_core::ThreadPoolBuilder::new()
		.panic_handler(|e| {
			let backtrace = Backtrace::new();
			log_eprintln!("panic: {:?}\ntrace:\n{:?}", e, backtrace);
			shutdown_terminal();
			process::abort();
		})
		.num_threads(4)
		.build_global()?;

	Ok(())
}
