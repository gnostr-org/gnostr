#![forbid(unsafe_code)]
#![deny(
	unused_must_use,
	unstable_name_collisions,
	unused_assignments
)]
#![warn(dead_code, unused_imports)]
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
	clippy::module_name_repetitions,
	clippy::empty_docs,
	clippy::use_self,
	clippy::legacy_numeric_constants,
	clippy::too_long_first_doc_paragraph,
	clippy::set_contains_or_insert,
	clippy::unknown_lints
)]

//TODO:
// #![deny(clippy::expect_used)]

mod app;
mod args;
mod bug_report;
mod cli;
mod clipboard;
mod cmdbar;
mod components;
mod input;
mod keys;
mod notify_mutex;
mod options;
mod popup_stack;
mod popups;
mod queue;
mod spinner;
mod string_utils;
mod strings;
mod sub_commands;
mod tabs;
mod ui;
mod watcher;

use std::{
	cell::RefCell,
	io::{self, Stdout},
	panic, process,
	time::{Duration, Instant},
};

use anyhow::{Result, bail};
use app::QuitState;
use asyncgit::{
	AsyncGitNotification,
	sync::{RepoPath, utils::repo_work_dir},
};
use backtrace::Backtrace;
use clap::Parser;
use cli::{Cli, Commands};

use crossbeam_channel::{Receiver, Select, never, tick, unbounded};
use crossterm::{
	ExecutableCommand,
	terminal::{
		EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
		enable_raw_mode,
	},
};
use input::{Input, InputEvent, InputState};
use keys::KeyConfig;

use ngit::{
	cli_interactor, client, git, git_events, login, repo_ref,
};

use ratatui::backend::CrosstermBackend;
use scopeguard::defer;
use scopetime::scope_time;
use spinner::Spinner;
use ui::style::Theme;
use watcher::RepoWatcher;

use crate::{app::App, args::process_cmdline};

type Terminal = ratatui::Terminal<CrosstermBackend<io::Stdout>>;

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

async fn increase(number: i32) {
	println!("{}", number + 1);
}

async fn decrease(number: i32) {
	println!("{}", number - 1);
}

async fn help() {
	println!(
		"usage:
match_args <string>
    Check whether given string is the answer.
match_args {{increase|decrease}} <integer>
    Increase or decrease given integer by one."
	);
}

//#[tokio::main]
async fn tui() -> Result<()> {
	let app_start = Instant::now();

	let cliargs = process_cmdline()?;

	asyncgit::register_tracing_logging();

	if !valid_path(&cliargs.repo_path) {
		eprintln!(
			"invalid path\nplease run gitui inside of a non-bare git repository"
		);
		return Ok(());
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
		)?;

		match quit_state {
			QuitState::OpenSubmodule(p) => {
				repo_path = p;
			}
			_ => break Ok(()),
		}
	}
}

#[tokio::main]
async fn main() -> Result<()> {
	use std::env;
	let args: Vec<String> = env::args().collect();
	//dbg!(&args);

	let cli = Cli::parse();

	//let _ = async {
	//dbg!(&args);
	match args.len() {
		// no arguments passed
		1 => {
			println!(
				"My name is 'match_args'. Try passing some arguments!"
			);

			let _ = help();

			//invoke tui
		}
		// one argument passed
		2 => match &cli.command {
			Commands::Fetch(args) => {
				sub_commands::fetch::launch(&cli, args)
					.await
					.expect("REASON")
			}
			Commands::Login(args) => {
				sub_commands::login::launch(&cli, args)
					.await
					.expect("REASON")
			}
			Commands::Init(args) => {
				sub_commands::init::launch(&cli, args)
					.await
					.expect("REASON")
			}
			Commands::Send(args) => {
				sub_commands::send::launch(&cli, args, false)
					.await
					.expect("REASON")
			}
			Commands::List => {
				sub_commands::list::launch().await.expect("REASON")
			}
			Commands::Pull => {
				sub_commands::pull::launch().await.expect("REASON")
			}
			Commands::Push(args) => {
				sub_commands::push::launch(&cli, args)
					.await
					.expect("REASON")
			}
			_ => println!("Ain't special"),
			//2 => match args[1].parse() {
			//        Ok(42) => println!("This is the answer!"),
			//        _ => println!("This is not the answer."),
		},
		3 => {
			let cmd = &args[1];
			let num = &args[2];
			// parse the number
			let number: i32 = match num.parse() {
				Ok(n) => n,
				Err(_) => {
					eprintln!(
						"error: second argument not an integer"
					);
					let _ = help();
					0
				}
			};
			// parse the command
			match &cmd[..] {
				"increase" => increase(number).await,
				"decrease" => decrease(number).await,
				_ => {
					eprintln!("error: invalid command");
					let _ = help();
				}
			}
		}
		// all the other cases
		_ => {
			// show a help message
			let _ = help();
		}
	};
	//};

	Ok(())
}

fn run_app(
	app_start: Instant,
	repo: RepoPath,
	theme: Theme,
	key_config: KeyConfig,
	input: &Input,
	updater: Updater,
	terminal: &mut Terminal,
) -> Result<QuitState, anyhow::Error> {
	let (tx_git, rx_git) = unbounded();
	let (tx_app, rx_app) = unbounded();

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
		tx_git,
		tx_app,
		input.clone(),
		theme,
		key_config,
	)?;

	let mut spinner = Spinner::default();
	let mut first_update = true;

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
						//Note: external ed closed, we need to
						// re-hide cursor
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
			}

			draw(terminal, &app)?;

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

fn draw(terminal: &mut Terminal, app: &App) -> io::Result<()> {
	if app.requires_redraw() {
		terminal.clear()?;
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
		log::error!("repo open error: {error}");
	}
	error.is_none()
}

fn select_event(
	rx_input: &Receiver<InputEvent>,
	rx_git: &Receiver<AsyncGitNotification>,
	rx_app: &Receiver<AsyncAppNotification>,
	rx_ticker: &Receiver<Instant>,
	rx_notify: &Receiver<()>,
	rx_spinner: &Receiver<Instant>,
) -> Result<QueueEvent> {
	let mut sel = Select::new();

	sel.recv(rx_input);
	sel.recv(rx_git);
	sel.recv(rx_app);
	sel.recv(rx_ticker);
	sel.recv(rx_notify);
	sel.recv(rx_spinner);

	let oper = sel.select();
	let index = oper.index();

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

fn start_terminal(buf: Stdout) -> io::Result<Terminal> {
	let backend = CrosstermBackend::new(buf);
	let mut terminal = Terminal::new(backend)?;
	terminal.hide_cursor()?;
	terminal.clear()?;

	Ok(terminal)
}

// do log::error! and eprintln! in one line, pass string, error and
// backtrace
macro_rules! log_eprintln {
	($string:expr, $e:expr, $bt:expr) => {
		log::error!($string, $e, $bt);
		eprintln!($string, $e, $bt);
	};
}

fn set_panic_handlers() -> Result<()> {
	// regular panic handler
	panic::set_hook(Box::new(|e| {
		let backtrace = Backtrace::new();
		shutdown_terminal();
		log_eprintln!(
			"\nGitUI was close due to an unexpected panic.\nPlease file an issue on https://github.com/extrawurst/gitui/issues with the following info:\n\n{:?}\ntrace:\n{:?}",
			e,
			backtrace
		);
	}));

	// global threadpool
	rayon_core::ThreadPoolBuilder::new()
		.panic_handler(|e| {
			let backtrace = Backtrace::new();
			shutdown_terminal();
			log_eprintln!("\nGitUI was close due to an unexpected panic.\nPlease file an issue on https://github.com/extrawurst/gitui/issues with the following info:\n\n{:?}\ntrace:\n{:?}", e, backtrace);
			process::abort();
		})
		.num_threads(4)
		.build_global()?;

	Ok(())
}
