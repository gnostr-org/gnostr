// n34 - A CLI to interact with NIP-34 and other stuff related to codes in nostr
// Copyright (C) 2025 Awiteb <a@4rs.nl>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://gnu.org/licenses/gpl-3.0.html>.

/// Command line interface module
use n34::cli;
// /// N34 errors
// use n34::error;
// /// Nostr utils module
// use n34::nostr_utils;

use std::{
    process::ExitCode,
    sync::atomic::{AtomicBool, Ordering},
};

use clap::Parser;
use clap_verbosity_flag::Verbosity;
use tracing::Level;
use tracing_subscriber::{Layer, filter, layer::SubscriberExt};

use self::cli::Cli;

/// Whether the editor is currently open. Prevents logging while the editor is
/// open.
static EDITOR_OPEN: AtomicBool = AtomicBool::new(false);

/// Configures the logging level based on the provided verbosity.
///
/// When verbosity is set to TRACE, includes file and line numbers in logs.
fn set_log_level(verbosity: Verbosity) {
    let is_trace = verbosity
        .tracing_level()
        .is_some_and(|l| l == tracing::Level::TRACE);

    let logs_filter = filter::dynamic_filter_fn(move |m, _| {
        // Disable all logs while editor is open
        verbosity.tracing_level().unwrap_or(Level::ERROR) >= *m.level()
            && !EDITOR_OPEN.load(Ordering::Relaxed)
    });

    let logs_layer = tracing_subscriber::fmt::layer()
        .with_file(is_trace)
        .with_line_number(is_trace)
        .without_time();
    let subscriber = tracing_subscriber::registry().with(logs_layer.with_filter(logs_filter));
    tracing::subscriber::set_global_default(subscriber).ok();
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = match cli::post_cli(Cli::parse()) {
        Ok(cli) => cli,
        Err(err) => {
            eprintln!("{err}");
            return ExitCode::FAILURE;
        }
    };

    set_log_level(cli.verbosity);

    if let Err(err) = cli.run().await {
        tracing::error!("{err}");
        return err.exit_code();
    }

    ExitCode::SUCCESS
}
