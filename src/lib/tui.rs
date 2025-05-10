#![forbid(unsafe_code)]
#![deny(unused_must_use, unstable_name_collisions, unused_assignments)]
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
    //clippy::unknown_lints
)]

//TODO:
// #![deny(clippy::expect_used)]

use std::{
    cell::RefCell,
    io::{self, Stdout},
    panic, process,
    time::{Duration, Instant},
};

use crate::app::App;
use crate::app::QuitState;
use crate::input::{Input, InputEvent, InputState};
use crate::keys::KeyConfig;
use crate::spinner::Spinner;
use crate::sub_commands::tui::*;
use crate::ui::style::Theme;
use crate::watcher::RepoWatcher;
use anyhow::{bail, Result};
use backtrace::Backtrace;
use crossbeam_channel::{never, tick, unbounded, Receiver, Select};
use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use gnostr_asyncgit::{
    sync::{utils::repo_work_dir, RepoPath},
    AsyncGitNotification,
};
use ratatui::backend::CrosstermBackend;
use scopeguard::defer;
use scopetime;
use scopetime::scope_time;


/// # Errors
///
/// Will return `Err` if `filename` does not exist or the user does not have
/// permission to read it.
pub async fn tui() -> Result<()> {
    let app_start = Instant::now();

    //let cliargs = process_cmdline()?;

    gnostr_asyncgit::register_tracing_logging();

    //if !valid_path(&cliargs.repo_path) {
    //    eprintln!("invalid path\nplease run gitui inside of a non-bare git repository");
    //    return Ok(());
    //}

    //let key_config = KeyConfig::init()
    //    .map_err(|e| eprintln!("KeyConfig loading error: {e}"))
    //    .unwrap_or_default();
    //let theme = Theme::init(&cliargs.theme);

    //setup_terminal()?;
    //defer! {
    //    shutdown_terminal();
    //}

    //set_panic_handlers()?;

    //let mut terminal = start_terminal(io::stdout()).await.expect("");
    //let mut repo_path = cliargs.repo_path;
    //let input = Input::new();

    //let updater = if cliargs.notify_watcher {
    //    Updater::NotifyWatcher
    //} else {
    //    Updater::Ticker
    //};

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

    Ok(())
}

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

