//! `gnostr-crawler` owns relay discovery, query construction, and relay
//! metadata serving.
//!
//! It re-exports the shared asyncgit-backed Nostr types for the query path,
//! while `gnostr-p2p` keeps its relay bucket helpers local so the graph stays
//! one-way: `types -> asyncgit -> crawler/p2p`.
//! If a query type needs to be shared with chat, add it to `types` or the
//! asyncgit-backed wire surface rather than depending on `p2p` here.

use std::{
    ffi::CString,
    io::{self, Write},
    os::raw::c_char,
    sync::{Mutex, OnceLock},
};

use tracing_subscriber::fmt;
pub mod processor;
pub mod api;
pub mod cli;
pub mod message;
pub mod relay_metadata;
pub mod relay_fetch;
pub mod relay_io;
pub mod pubkeys;
pub mod commands;
pub mod query;
pub mod relay_manager;
pub mod relays;
pub mod stats;
pub mod tui;
mod api_cache;
mod api_routes;
mod git_helpers;

pub use cli::{dispatch_cli_command, run, Cli, CliArgs, Commands};
pub use message::*;
pub use query::{build_gnostr_query, send, Config, ConfigBuilder};
pub use api::{
    run_api_server,
    run_api_server_detached,
    run_api_server_with_shutdown,
    run_api_server_with_shutdown_and_ready,
    run_sniper_service_with_shutdown,
};
pub use commands::{run_nip34, run_sniper, run_watch};
pub use relay_metadata::Relay;
pub use relay_fetch::{fetch_relay_texts, parse_relay_metadata, websocket_http_url};
pub use relay_io::{load_file, load_relays_or_bootstrap, load_shitlist, preprocess_line};
pub use git_helpers::{
    log_message_matches, match_with_parent, print_commit, print_time, sig_matches,
};

type LogCallback = Option<unsafe extern "C" fn(*const c_char)>;

static LOG_CALLBACK: OnceLock<Mutex<LogCallback>> = OnceLock::new();
static SNIPER_LOGS: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
static SNIPER_LIFECYCLE: OnceLock<Mutex<Vec<String>>> = OnceLock::new();

fn log_callback_slot() -> &'static Mutex<LogCallback> {
    LOG_CALLBACK.get_or_init(|| Mutex::new(None))
}

fn sniper_log_slot() -> &'static Mutex<Vec<String>> {
    SNIPER_LOGS.get_or_init(|| Mutex::new(Vec::new()))
}

fn sniper_lifecycle_slot() -> &'static Mutex<Vec<String>> {
    SNIPER_LIFECYCLE.get_or_init(|| Mutex::new(Vec::new()))
}

fn emit_log_line(line: impl AsRef<str>) {
    let line = line.as_ref();
    let callback = *log_callback_slot().lock().unwrap();
    if let Some(callback) = callback {
        let sanitized = line.replace('\0', " ");
        if let Ok(c_line) = CString::new(sanitized) {
            unsafe {
                callback(c_line.as_ptr());
            }
            return;
        }
    }

    eprintln!("{}", line);
}

pub fn record_sniper_log(line: impl AsRef<str>) {
    let line = line.as_ref().trim_end().to_string();
    if line.is_empty() {
        return;
    }

    eprintln!("{line}");
    let mut logs = sniper_log_slot().lock().unwrap();
    logs.push(line);
    if logs.len() > 1_000 {
        let drain = logs.len() - 1_000;
        logs.drain(0..drain);
    }
}

pub fn sniper_service_logs() -> String {
    let lifecycle = sniper_lifecycle_slot().lock().unwrap();
    let logs = sniper_log_slot().lock().unwrap();
    lifecycle
        .iter()
        .chain(logs.iter())
        .cloned()
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn record_sniper_lifecycle(line: impl AsRef<str>) {
    let line = line.as_ref().trim_end().to_string();
    if line.is_empty() {
        return;
    }

    eprintln!("{line}");
    let mut events = sniper_lifecycle_slot().lock().unwrap();
    events.push(line);
    if events.len() > 1_000 {
        let drain = events.len() - 1_000;
        events.drain(0..drain);
    }
}

pub fn sniper_service_lifecycle() -> String {
    let events = sniper_lifecycle_slot().lock().unwrap();
    events.join("\n")
}

fn drain_complete_lines(buffer: &mut Vec<u8>) {
    while let Some(pos) = buffer.iter().position(|byte| *byte == b'\n') {
        let mut line = buffer.drain(..=pos).collect::<Vec<u8>>();
        if line.last() == Some(&b'\n') {
            line.pop();
        }
        emit_log_line(String::from_utf8_lossy(&line));
    }
}

struct CrawlerLogWriter;

struct CrawlerLogWriterGuard {
    buffer: Vec<u8>,
}

impl io::Write for CrawlerLogWriterGuard {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buffer.extend_from_slice(buf);
        drain_complete_lines(&mut self.buffer);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        if !self.buffer.is_empty() {
            emit_log_line(String::from_utf8_lossy(&self.buffer));
            self.buffer.clear();
        }
        Ok(())
    }
}

impl Drop for CrawlerLogWriterGuard {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}

impl<'a> fmt::MakeWriter<'a> for CrawlerLogWriter {
    type Writer = CrawlerLogWriterGuard;

    fn make_writer(&'a self) -> Self::Writer {
        CrawlerLogWriterGuard { buffer: Vec::new() }
    }
}

#[no_mangle]
pub unsafe extern "C" fn crawler_set_log_callback(callback: LogCallback) {
    *log_callback_slot().lock().unwrap() = callback;
}

pub fn init_tracing() -> Result<(), Box<dyn std::error::Error>> {
    let _ = tracing_log::LogTracer::init();
    eprintln!("crawler tracing: installing subscriber");
    let result = tracing_subscriber::fmt()
        .with_ansi(false)
        .with_writer(CrawlerLogWriter)
        .without_time()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("hyper::client::trace=trace".parse()?)
                .add_directive("hyper::client::connect=trace".parse()?)
                .add_directive("hyper::client::connect::http=off".parse()?)
                .add_directive("hyper::proto=off".parse()?)
                .add_directive("nostr_sdk::relay=off".parse()?)
                .add_directive("nostr_relay_pool=off".parse()?)
                .add_directive("nostr_relay_pool::relay::inner=off".parse()?),
        )
        .try_init();
    if let Err(error) = result {
        eprintln!("crawler tracing: subscriber init failed: {}", error);
    } else {
        eprintln!("crawler tracing: subscriber installed");
    }
    Ok(())
}
