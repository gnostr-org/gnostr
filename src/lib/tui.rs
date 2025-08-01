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

use chrono::{Local, Timelike};
use std::{cell::RefCell, time::Instant};

use crate::app::App;
use crate::app::QuitState;
use crate::input::{Input, InputEvent, InputState};
use crate::keys::KeyConfig;
use crate::spinner::Spinner;
use crate::sub_commands::tui::{
    draw, select_event, AsyncNotification, QueueEvent, Terminal, Updater, SPINNER_INTERVAL,
    TICK_INTERVAL,
};
use crate::ui::style::Theme;
use crate::watcher::RepoWatcher;
use anyhow::Result;
use crossbeam_channel::{never, tick, unbounded};
use gnostr_asyncgit::{
    sync::{utils::repo_work_dir, RepoPath},
    AsyncGitNotification,
};
use scopetime;
use scopetime::scope_time;

use crate::blockhash::blockhash_async;
use crate::blockheight::blockheight_async;
use crate::weeble::weeble_async;
use crate::wobble::wobble_async;
use std::env;
use tracing::debug;
/// # Errors
///
/// Will return `Err` if `filename` does not exist or the user does not have
/// permission to read it.
pub async fn tui() -> Result<()> {
    let app_start = Instant::now();
    gnostr_asyncgit::register_tracing_logging();
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
        let my_string = "hello".to_string();
        let my_string2 = "hello".to_string();

        //// Check if the current second is odd
        //let handle = tokio::spawn(async {
        //    let now = Local::now();

        //    // Get the current second
        //    let current_second = now.second();

        //    if current_second % 2 != 0 {
        //        debug!("Current second ({}) is odd!", current_second);
        //        //env::set_var("BLOCKHEIGHT", &blockheight_async().await);
        //        env::set_var("WEEBLE", &weeble_async().await.unwrap().to_string());
        //        //env::set_var("BLOCKHASH", &blockhash_async().await);
        //    } else {
        //        debug!(
        //            "Current second ({}) is even. Skipping this iteration.",
        //            current_second
        //        );
        //    }
        //});

        //debug!("Still running other things while the task is awaited...");

        //handle.await.unwrap_or(()); // Wait for the async task to complete
        //debug!("All done!");

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

                //let my_string = "hello".to_string();
                //let my_string2 = "hello".to_string();

                //// Check if the current second is odd
                //let handle = tokio::spawn(async {
                //    let now = Local::now();

                //    // Get the current second
                //    let current_second = now.second();

                //    if current_second % 2 != 0 {
                //        debug!("Current second ({}) is odd!", current_second);
                //        //env::set_var("BLOCKHEIGHT", &blockheight_async().await);
                //        env::set_var("WEEBLE", &weeble_async().await.unwrap().to_string());
                //        //env::set_var("BLOCKHASH", &blockhash_async().await);
                //    } else {
                //        debug!(
                //            "Current second ({}) is even. Skipping this iteration.",
                //            current_second
                //        );
                //    }
                //});

                //debug!("Still running other things while the task is awaited...");

                //handle.await.unwrap_or(()); // Wait for the async task to complete
                //debug!("All done!");

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
