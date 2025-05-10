#![cfg_attr(not(test), warn(clippy::pedantic))]
#![cfg_attr(not(test), warn(clippy::expect_used))]

use anyhow::{bail, Result};
use backtrace::Backtrace;
use crate::gnostr::*;
use crate::app::App;
use crate::app::QuitState;
use crate::input::{Input, InputEvent, InputState};
use crate::keys::KeyConfig;
use crate::spinner::Spinner;
use crate::ui::style::Theme;
use crate::watcher::RepoWatcher;
use crossbeam_channel::{never, tick, unbounded, Receiver, Select};
use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use gnostr_asyncgit::{
    sync::{utils::repo_work_dir, RepoPath},
    AsyncGitNotification,
};
use nostr_sdk_0_37_0::Keys;
use ratatui::backend::CrosstermBackend;
use scopeguard::defer;
use scopetime;
use scopetime::scope_time;
use serde::ser::StdError;
use std::{
    cell::RefCell,
    io::{self, Stdout},
    panic, process,
    time::{Duration, Instant},
};
use tracing::{debug, Level};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

//use crate::{app::App, cli::process_cmdline};
pub type Terminal = ratatui::Terminal<CrosstermBackend<io::Stdout>>;

pub static TICK_INTERVAL: Duration = Duration::from_secs(5);
pub static SPINNER_INTERVAL: Duration = Duration::from_millis(80);

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

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Updater {
    Ticker,
    NotifyWatcher,
}

/// # Errors
///
/// Will return `Err` if `filename` does not exist or the user does not have
/// permission to read it.
pub fn setup_terminal() -> Result<()> {
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    Ok(())
}

/// # Errors
///
/// Will return `Err` if `filename` does not exist or the user does not have
/// permission to read it.
pub fn shutdown_terminal() {
    let leave_screen = io::stdout().execute(LeaveAlternateScreen).map(|_f| ());

    if let Err(e) = leave_screen {
        eprintln!("leave_screen failed:\n{e}");
    }

    let leave_raw_mode = disable_raw_mode();

    if let Err(e) = leave_raw_mode {
        eprintln!("leave_raw_mode failed:\n{e}");
    }
}

/// # Errors
///
/// Will return `Err` if `filename` does not exist or the user does not have
/// permission to read it.
pub fn draw(terminal: &mut Terminal, app: &App) -> io::Result<()> {
    if app.requires_redraw() {
        terminal.clear()?;
    }

    terminal.draw(|f| {
        if let Err(e) = app.draw(f) {
            log::error!("failed to draw: {e:?}");
        }
    })?;

    Ok(())
}

#[must_use]
pub fn valid_path(repo_path: &RepoPath) -> bool {
    let error = gnostr_asyncgit::sync::repo_open_error(repo_path);
    if let Some(error) = &error {
        log::error!("repo open error: {error}");
    }
    error.is_none()
}

/// # Errors
///
/// Will return `Err` if `filename` does not exist or the user does not have
/// permission to read it.
pub fn select_event(
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
        1 => oper
            .recv(rx_git)
            .map(|e| QueueEvent::AsyncEvent(AsyncNotification::Git(e))),
        2 => oper
            .recv(rx_app)
            .map(|e| QueueEvent::AsyncEvent(AsyncNotification::App(e))),
        3 => oper.recv(rx_ticker).map(|_| QueueEvent::Notify),
        4 => oper.recv(rx_notify).map(|()| QueueEvent::Notify),
        5 => oper.recv(rx_spinner).map(|_| QueueEvent::SpinnerUpdate),
        _ => bail!("unknown select source"),
    }?;

    Ok(ev)
}

/// # Errors
///
/// Will return `Err` if `filename` does not exist or the user does not have
/// permission to read it.
pub async fn start_terminal(buf: Stdout) -> io::Result<Terminal> {
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

/// # Errors
///
/// Will return `Err` if `filename` does not exist or the user does not have
/// permission to read it.
pub fn set_panic_handlers() -> Result<()> {
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


pub async fn tui(sub_command_args: &GnostrSubCommands) -> Result<(), Box<dyn StdError>> {
    print!("{:?}", sub_command_args);


    let app_start = Instant::now();
    gnostr_asyncgit::register_tracing_logging();

    if !valid_path(&cliargs.repo_path) {
        eprintln!("invalid path\nplease run gitui inside of a non-bare git repository");
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

    let mut terminal = start_terminal(io::stdout()).await.expect("");
    let mut repo_path = cliargs.repo_path;
    let input = Input::new();

    let updater = if cliargs.notify_watcher {
        Updater::NotifyWatcher
    } else {
        Updater::Ticker
    };

    //loop {
    //    let quit_state = run_app(
    //        app_start,
    //        repo_path.clone(),
    //        theme.clone(),
    //        key_config.clone(),
    //        &input,
    //        updater,
    //        &mut terminal,
    //    )
    //    .await
    //    .expect("");

    //    match quit_state {
    //        QuitState::OpenSubmodule(p) => {
    //            repo_path = p;
    //        }
    //        _ => break,
    //    }
    //}

    let sub_command_args = sub_command_args;
    if let Some(name) = sub_command_args.name.clone() {
        use std::env;
        env::set_var("USER", &name);
    };

    let level = if sub_command_args.debug {
        Level::DEBUG
    } else if sub_command_args.trace {
        Level::TRACE
    } else if sub_command_args.info {
        Level::INFO
    } else {
        Level::WARN
    };

    let filter = EnvFilter::default()
        .add_directive(level.into())
        .add_directive("nostr_sdk=off".parse().unwrap())
        .add_directive("nostr_sdk::relay_pool=off".parse().unwrap())
        .add_directive("nostr_sdk::client=off".parse().unwrap())
        .add_directive("nostr_sdk::client::handler=off".parse().unwrap())
        .add_directive("nostr_relay_pool=off".parse().unwrap())
        .add_directive("nostr_sdk::relay::connection=off".parse().unwrap())
        .add_directive("gnostr::chat::p2p=off".parse().unwrap())
        .add_directive("gnostr::message=off".parse().unwrap())
        .add_directive("gnostr::nostr_proto=off".parse().unwrap())
        .add_directive("libp2p_mdns::behaviour::iface=off".parse().unwrap())
        //
        .add_directive("libp2p_gossipsub::behaviour=off".parse().unwrap());

    let subscriber = Registry::default()
        .with(
            fmt::layer()
                .with_writer(std::io::stdout)
                //.with_timer(fmt::time::Utc::rfc_3339()) // Corrected line
                .with_thread_ids(true),
        )
        .with(filter);

    let _ = subscriber.try_init();
    tracing::trace!("\n{:?}\n", &sub_command_args);
    tracing::debug!("\n{:?}\n", &sub_command_args);
    tracing::info!("\n{:?}\n", &sub_command_args);
    //print!("{:?}", &sub_command_args);

    if sub_command_args.debug || sub_command_args.trace {
        if sub_command_args.nsec.clone().is_some() {
            let keys = Keys::parse(&sub_command_args.nsec.clone().unwrap().clone()).unwrap();
            debug!(
                "{{\"private_key\":\"{}\"}}",
                keys.secret_key().display_secret()
            );
            debug!("{{\"public_key\":\"{}\"}}", keys.public_key());
        }
    }

    //run(sub_command_args).await?;
    Ok(())
}

//pub async fn run(sub_command_args: &GnostrSubCommands) -> Result<(), Box<dyn StdError>> {
//    let _ = crate::tui::tui().await;
//    Ok(())
//}


/// # Errors
///
/// Will return `Err` if `filename` does not exist or the user does not have
/// permission to read it.
pub async fn run_app(
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
            let repo_watcher = RepoWatcher::new(repo_work_dir(&repo)?.as_str());

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
    )
    .await
    .expect("");

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
                    //detect external chat
                    if matches!(ev, InputEvent::State(InputState::Polling)) {
                        //Note: external ed closed, we need to
                        // re-hide cursor
                        terminal.hide_cursor()?;
                    }
                    app.event(ev)?;
                }
                //tick rate for nostr network time (weeble/wobble)
                //needs a friendly async request/ureq
                //relay crawler also needs to be friendly and async
                QueueEvent::Tick | QueueEvent::Notify => {
                    app.update()?;
                }
                QueueEvent::AsyncEvent(ev) => {
                    if !matches!(
                        ev,
                        AsyncNotification::Git(AsyncGitNotification::FinishUnchanged)
                    ) {
                        app.update_async(ev)?;
                    }
                }
                QueueEvent::SpinnerUpdate => unreachable!(),
            }

            //the chat swarm needs to be invoked in the main app
            //loop/lifecycle
            //default topic gnostr
            //the actual topic will be passed when scrolling/displaying commit topics
            //in the topiclist
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
