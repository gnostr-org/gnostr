#![cfg_attr(not(test), warn(clippy::pedantic))]
#![cfg_attr(not(test), warn(clippy::expect_used))]
#![allow(clippy::doc_markdown)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::if_not_else)]
use std::{
    cell::RefCell,
    env,
    io::{self, Stdout},
    panic, process,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use anyhow::{Result, bail};
use backtrace::Backtrace;
use crossbeam_channel::{Receiver, Select, never, tick, unbounded};
use crossterm::{
    ExecutableCommand,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use gnostr_asyncgit::{
    AsyncGitNotification,
    sync::{RepoPath, utils::repo_work_dir},
};
use ratatui::backend::CrosstermBackend;
use scopeguard::defer;
use scopetime::{self, scope_time};
use serde::ser::StdError;
use tracing::{Level, debug};
use tracing_subscriber::{EnvFilter, Registry, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    app::{App, QuitState},
    blockheight,
    core::GnostrSubCommands,
    input::{Input, InputEvent, InputState},
    keys::KeyConfig,
    spinner::Spinner,
    types::Keys,
    ui::style::Theme,
    watcher::RepoWatcher,
    weeble, wobble,
};

//use crate::{app::App, cli::process_cmdline};
pub type Terminal = ratatui::Terminal<CrosstermBackend<io::Stdout>>;

pub static TICK_INTERVAL: Duration = Duration::from_secs(5);
pub static SPINNER_INTERVAL: Duration = Duration::from_millis(80);

/// QueueEvent
#[derive(Clone)]
pub enum QueueEvent {
    Tick,
    Notify,
    SpinnerUpdate,
    AsyncEvent(AsyncNotification),
    InputEvent(InputEvent),
}

/// SyntaxHighlightProgress
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyntaxHighlightProgress {
    Progress,
    Done,
}

/// AsyncAppNotification
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AsyncAppNotification {
    SyntaxHighlighting(SyntaxHighlightProgress),
}

/// AsyncNotification
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AsyncNotification {
    App(AsyncAppNotification),

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
pub fn valid_path(gitdir: &RepoPath) -> bool {
    let error = gnostr_asyncgit::sync::repo_open_error(gitdir);
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
pub fn start_terminal(buf: Stdout) -> io::Result<Terminal> {
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
/// GNOSTR_TUI
///
/// # Panics
///
/// Panics if the ctrlc handler cannot be set.
/// Panics if the terminal cannot be started.
/// Panics if the app cannot be run.
///
/// # Errors
///
/// This function will return an error if the command fails.
#[allow(clippy::too_many_lines)]
pub async fn tui(
    mut sub_command_args: GnostrSubCommands,
    cli: &crate::cli::GnostrCli,
) -> Result<(), Box<dyn StdError>> {
    let app_start = Instant::now();
    gnostr_asyncgit::register_tracing_logging();

    let quit_flag = Arc::new(AtomicBool::new(false));
    let r = Arc::clone(&quit_flag);
    if let Err(e) = ctrlc::set_handler(move || {
        r.store(true, Ordering::SeqCst);
    }) {
        log::error!("failed to set ctrlc handler: {e}");
    }

    debug!("239:tui:{:?}", sub_command_args);
    //debug!("240:tui:{:?}", sub_command_args.gitdir.clone().expect(""));

    //TODO gnostr --gitdir
    //TODO if !valid_path invoke mkdir -p GNOSTR_GITDIR; cd GNOSTR_GITDIR; git
    // init?
    let mut gitdir = sub_command_args.gitdir.clone().unwrap_or(".".into());
    if !valid_path(&gitdir) {
        debug!("243:invalid path\nplease run gitui inside of a non-bare git repository");
        if Some(env::var("GNOSTR_GITDIR")).is_some() {
            debug!("247:{}", env::var("GNOSTR_GITDIR").unwrap());
            //let repo_path: RepoPath =
            // RepoPath::from(PathBuf::from(env::var("GNOSTR_GITDIT").unwrap().
            // to_string()));
            let repo_path: RepoPath = RepoPath::from(
                env::var("GNOSTR_GITDIR")
                    .unwrap_or(env::var("HOME").unwrap().clone() /* TODO */)
                    .as_ref(),
            );

            debug!("253:{:?}", repo_path);
            sub_command_args.gitdir = Some(repo_path); //env::var("GNOSTR_GITDIR").unwrap().to_string()
            debug!("257:{:?}", sub_command_args.gitdir);
        } else {
            debug!("GNOSTR_GITDIR NOT set case!");
            debug!("fork no return  case!");
            debug!("TODO:git init in $HOME/.gnostr/tmp repo or /tmp/...");
            //return Ok(());
        }
    } else { /*NOT NOT valid case!*/
    } //must be a valid path to a git repo!

    let key_config = KeyConfig::init()
        .map_err(|e| eprintln!("KeyConfig loading error: {e}"))
        .unwrap_or_default();
    let theme = Theme::init(&sub_command_args.theme.clone().unwrap());

    setup_terminal()?;
    defer! {
        shutdown_terminal();
    }

    set_panic_handlers()?;

    let mut terminal = match start_terminal(io::stdout())/*.await*/ {
        Ok(terminal) => terminal,
        Err(e) => {
            log::error!("failed to start terminal: {e}");
            return Ok(());
        }
    };
    //let mut gitdir = sub_command_args.gitdir.clone().unwrap();
    let input = Input::new();

    let updater = if sub_command_args.notify_watcher {
        Updater::NotifyWatcher
    } else {
        Updater::Ticker
    };

    let sub_command_args = sub_command_args;
    if let Some(name) = sub_command_args.name.clone() {
        use std::env;
        env::set_var("USER", &name);
    }

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
    debug!("\n{:?}\n", &sub_command_args);

    if (sub_command_args.debug || sub_command_args.trace) && sub_command_args.nsec.clone().is_some()
    {
        let keys = Keys::parse(sub_command_args.nsec.clone().unwrap().clone()).unwrap();
        debug!(
            "{{\"private_key\":\"{}\"}}",
            keys.secret_key().display_secret()
        );
        debug!("{{\"public_key\":\"{}\"}}", keys.public_key());
    }

    loop {
        let quit_state = match Box::pin(run_app(
            app_start,
            gitdir.clone(),
            theme.clone(),
            key_config.clone(),
            &input,
            updater,
            &mut terminal,
            cli.screenshots,
            Arc::clone(&quit_flag),
        ))
        .await
        {
            Ok(quit_state) => quit_state,
            Err(e) => {
                log::error!("failed to run app: {e}");
                return Ok(());
            }
        };

        match quit_state {
            QuitState::OpenSubmodule(p) => {
                gitdir = p;
            }
            _ => break,
        }
    }
    //run(sub_command_args).await?;
    Ok(())
}

//pub async fn run(sub_command_args: &GnostrSubCommands) -> Result<(), Box<dyn
// StdError>> {    let _ = crate::tui::tui().await;
//    Ok(())
//}

/// # Panics
///
/// Panics if the app cannot be created.
///
/// # Errors
///
/// Will return `Err` if the app fails to run.
#[allow(clippy::too_many_lines)]
pub async fn run_app(
    app_start: Instant,
    repo: RepoPath,
    theme: Theme,
    key_config: KeyConfig,
    input: &Input,
    updater: Updater,
    terminal: &mut Terminal,
    screenshots: Option<u8>,
    quit_flag: Arc<AtomicBool>,
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

    let mut app = match Box::pin(App::new(
        RefCell::new(repo.clone()),
        tx_git,
        tx_app,
        input.clone(),
        theme,
        key_config,
        Arc::clone(&quit_flag),
    ))
    .await
    {
        Ok(app) => app,
        Err(e) => {
            log::error!("failed to create app: {e}");
            return Err(e);
        }
    };
    let mut spinner = Spinner::default();
    let mut first_update = true;

    log::trace!("app start: {} ms", app_start.elapsed().as_millis());

    let mut last_screenshot = Instant::now();
    loop {
        if let Some(interval) = screenshots {
            if last_screenshot.elapsed() >= Duration::from_secs(u64::from(interval)) {
                let mut path = if let Some(workdir) = repo.workdir() {
                    let mut p = workdir.to_path_buf();
                    p.push(".gnostr");
                    p
                } else {
                    crate::cli::get_app_cache_path().unwrap()
                };
                path.push("screenshots");
                let weeble = weeble::weeble().unwrap_or(0.0) as u64;
                let wobble = wobble::wobble().unwrap_or(0.0) as u64;
                let blockheight = blockheight::blockheight().unwrap_or(0.0) as u64;
                path.push(weeble.to_string());
                path.push(blockheight.to_string());
                path.push(wobble.to_string());
                std::fs::create_dir_all(&path).unwrap();
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                path.push(format!("screenshot-{timestamp}.png"));
                crate::utils::screenshot::make_screenshot_cross_platform(path.to_str().unwrap())
                    .unwrap();
                last_screenshot = Instant::now();
            }
        }
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

            if app.is_quit() || quit_flag.load(Ordering::SeqCst) {
                break;
            }
        }
    }

    Ok(app.quit_state())
}
