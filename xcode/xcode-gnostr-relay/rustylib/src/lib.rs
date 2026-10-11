use std::{
    ffi::CStr,
    os::raw::c_char,
    sync::{Mutex, OnceLock},
    thread,
    time::SystemTime,
};

use tokio::sync::oneshot;

uniffi::setup_scaffolding!();

struct CrawlerServiceState {
    port: u16,
    started_at: SystemTime,
    ready: bool,
    stopping: bool,
    shutdown: Option<oneshot::Sender<()>>,
    thread: Option<thread::JoinHandle<()>>,
}

struct SniperServiceState {
    started_at: SystemTime,
    requested: bool,
    stopping: bool,
    shutdown: Option<oneshot::Sender<()>>,
    thread: Option<thread::JoinHandle<()>>,
}

static CRAWLER_SERVICE: OnceLock<Mutex<Option<CrawlerServiceState>>> = OnceLock::new();
static SNIPER_SERVICE: OnceLock<Mutex<Option<SniperServiceState>>> = OnceLock::new();
static CRAWLER_LOGS: OnceLock<Mutex<Vec<String>>> = OnceLock::new();

fn crawler_service_slot() -> &'static Mutex<Option<CrawlerServiceState>> {
    CRAWLER_SERVICE.get_or_init(|| Mutex::new(None))
}

fn sniper_service_slot() -> &'static Mutex<Option<SniperServiceState>> {
    SNIPER_SERVICE.get_or_init(|| Mutex::new(None))
}

fn crawler_log_slot() -> &'static Mutex<Vec<String>> {
    CRAWLER_LOGS.get_or_init(|| Mutex::new(Vec::new()))
}

fn filtered_logs(slot: &'static Mutex<Vec<String>>, prefixes: &[&str]) -> String {
    let logs = slot.lock().unwrap();
    logs.iter()
        .filter(|line| prefixes.iter().any(|prefix| line.starts_with(prefix)))
        .cloned()
        .collect::<Vec<_>>()
        .join("\n")
}

fn push_crawler_log(line: impl AsRef<str>) {
    let line = line.as_ref().trim_end().to_string();
    if line.is_empty() {
        return;
    }

    eprintln!("{line}");
    let mut logs = crawler_log_slot().lock().unwrap();
    logs.push(line);
    if logs.len() > 1_000 {
        let drain = logs.len() - 1_000;
        logs.drain(0..drain);
    }
}

unsafe extern "C" fn crawler_log_callback(line: *const c_char) {
    if line.is_null() {
        return;
    }
    let line = CStr::from_ptr(line).to_string_lossy().into_owned();
    push_crawler_log(line);
}

fn ensure_crawler_tracing() {
    static LOG_CALLBACK_SET: OnceLock<()> = OnceLock::new();
    LOG_CALLBACK_SET.get_or_init(|| {
        unsafe {
            gnostr_crawler::crawler_set_log_callback(Some(crawler_log_callback));
        }
        push_crawler_log("crawler service: log callback registered");
    });
}

fn format_started_at(when: SystemTime) -> String {
    when.duration_since(SystemTime::UNIX_EPOCH)
        .map(|duration| format!("{}", duration.as_secs()))
        .unwrap_or_else(|_| "unknown".to_string())
}

fn crawler_service_status_string() -> String {
    let mut slot = crawler_service_slot().lock().unwrap();
    if let Some(state) = slot.as_ref() {
        if state.thread.as_ref().is_some_and(thread::JoinHandle::is_finished) {
            slot.take();
            push_crawler_log("crawler service: background thread exited");
            return "crawler service stopped".to_string();
        }

        if state.stopping {
            return format!(
                "crawler service stopping on 127.0.0.1:{} (started_at={})",
                state.port,
                format_started_at(state.started_at)
            );
        }

        if state.ready {
            return format!(
                "crawler service running on 127.0.0.1:{} (started_at={})",
                state.port,
                format_started_at(state.started_at)
            );
        }

        return format!(
            "crawler service starting on 127.0.0.1:{} (started_at={})",
            state.port,
            format_started_at(state.started_at)
        );
    }

    "crawler service stopped".to_string()
}

fn sniper_service_status_string() -> String {
    let mut slot = sniper_service_slot().lock().unwrap();
    if let Some(state) = slot.as_ref() {
        if state.thread.as_ref().is_some_and(thread::JoinHandle::is_finished) {
            slot.take();
            push_crawler_log("sniper service: background thread exited");
            return "sniper service stopped".to_string();
        }

        if state.stopping {
            return format!(
                "sniper service stopping (started_at={})",
                format_started_at(state.started_at)
            );
        }

        return format!(
            "sniper service running (started_at={})",
            format_started_at(state.started_at)
        );
    }

    "sniper service stopped".to_string()
}
 
#[uniffi::export]
fn rust_hello() -> String {
    "Hello from Rust!".to_string()
}

#[uniffi::export]
pub fn rust_add(a: u32, b: u32) -> u32 {
    a + b
}

#[uniffi::export]
pub fn p2p_network_start() -> String {
    gnostr_p2p::embedded_network::start()
}

#[uniffi::export]
pub fn p2p_network_status() -> String {
    gnostr_p2p::embedded_network::status()
}

#[uniffi::export]
pub fn p2p_network_stop() -> String {
    gnostr_p2p::embedded_network::stop()
}

#[uniffi::export]
pub fn p2p_network_logs() -> String {
    gnostr_p2p::embedded_network::logs()
}

#[uniffi::export]
pub fn crawler_service_logs() -> String {
    filtered_logs(
        crawler_log_slot(),
        &["crawler service:", "run_api_server:", "starting crawler service"],
    )
}

#[uniffi::export]
pub fn crawler_service_status() -> String {
    crawler_service_status_string()
}

#[uniffi::export]
pub fn sniper_service_logs() -> String {
    gnostr_crawler::sniper_service_logs()
}

#[uniffi::export]
pub fn sniper_service_lifecycle() -> String {
    gnostr_crawler::sniper_service_lifecycle()
}

#[uniffi::export]
pub fn sniper_service_status() -> String {
    sniper_service_status_string()
}

#[uniffi::export]
pub fn crawler_service_start(port: u16) -> String {
    ensure_crawler_tracing();

    let mut slot = crawler_service_slot().lock().unwrap();
    if let Some(state) = slot.as_ref() {
        let status = if state.thread.as_ref().is_some_and(thread::JoinHandle::is_finished) {
            "stopped"
        } else if state.stopping {
            "stopping"
        } else if state.ready {
            "running"
        } else {
            "starting"
        };
        return format!(
            "crawler service already {} on 127.0.0.1:{} (started_at={})",
            status,
            state.port,
            format_started_at(state.started_at)
        );
    }

    let (ready_tx, ready_rx) = std::sync::mpsc::channel::<Result<(), String>>();
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    push_crawler_log(format!("crawler service: starting on 127.0.0.1:{port}"));

    *slot = Some(CrawlerServiceState {
        port,
        started_at: SystemTime::now(),
        ready: false,
        stopping: false,
        shutdown: Some(shutdown_tx),
        thread: None,
    });
    drop(slot);

    let thread = thread::spawn(move || {
        let runtime = match tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(2)
            .thread_name("gnostr-crawler")
            .build()
        {
            Ok(runtime) => runtime,
            Err(error) => {
                let _ = ready_tx.send(Err(error.to_string()));
                push_crawler_log(format!("crawler service: runtime build failed: {error}"));
                return;
            }
        };

        let result = runtime.block_on(async move {
            gnostr_crawler::run_api_server_with_shutdown_and_ready(
                port,
                async move {
                    let _ = shutdown_rx.await;
                },
                Some(ready_tx),
            )
            .await
        });

        if let Err(error) = result {
            push_crawler_log(format!("crawler service: server exited with error: {error}"));
        } else {
            push_crawler_log("crawler service: server stopped");
        }
    });

    let ready_watcher = thread::spawn(move || match ready_rx.recv() {
        Ok(Ok(())) => {
            let mut slot = crawler_service_slot().lock().unwrap();
            if let Some(state) = slot.as_mut() {
                if state.port == port {
                    state.ready = true;
                    push_crawler_log(format!("crawler service: listening on 127.0.0.1:{port}"));
                }
            }
        }
        Ok(Err(error)) => {
            push_crawler_log(format!("crawler service: failed to start on 127.0.0.1:{port}: {error}"));
            let mut slot = crawler_service_slot().lock().unwrap();
            if slot.as_ref().is_some_and(|state| state.port == port) {
                slot.take();
            }
        }
        Err(error) => {
            push_crawler_log(format!("crawler service: startup channel closed on 127.0.0.1:{port}: {error}"));
            let mut slot = crawler_service_slot().lock().unwrap();
            if slot.as_ref().is_some_and(|state| state.port == port) {
                slot.take();
            }
        }
    });

    let mut slot = crawler_service_slot().lock().unwrap();
    if let Some(state) = slot.as_mut() {
        state.thread = Some(thread);
    }
    drop(slot);

    drop(ready_watcher);
    format!("crawler service starting on 127.0.0.1:{port}")
}

#[uniffi::export]
pub fn crawler_service_stop() -> String {
    let mut slot = crawler_service_slot().lock().unwrap();
    let Some(state) = slot.as_mut() else {
        return "crawler service already stopped".to_string();
    };

    if state.thread.as_ref().is_some_and(thread::JoinHandle::is_finished) {
        slot.take();
        return "crawler service stopped".to_string();
    }

    push_crawler_log(format!(
        "crawler service: stopping 127.0.0.1:{}",
        state.port
    ));
    state.stopping = true;
    if let Some(shutdown) = state.shutdown.take() {
        let _ = shutdown.send(());
    }
    push_crawler_log("crawler service: stopping requested");
    "crawler service stopping".to_string()
}

#[uniffi::export]
pub fn sniper_service_start() -> String {
    ensure_crawler_tracing();

    let mut slot = sniper_service_slot().lock().unwrap();
    if let Some(state) = slot.as_ref() {
        if state.thread.as_ref().is_some_and(thread::JoinHandle::is_finished) {
            slot.take();
        } else {
            let status = if state.stopping {
                "stopping"
            } else if state.requested {
                "start requested"
            } else {
                "running"
            };
            return format!(
                "sniper service already {} (started_at={})",
                status,
                format_started_at(state.started_at)
            );
        }
    }

    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    gnostr_crawler::record_sniper_lifecycle("sniper lifecycle: start requested");
    gnostr_crawler::record_sniper_log("sniper service: start requested");
    gnostr_crawler::record_sniper_lifecycle("sniper lifecycle: starting");
    gnostr_crawler::record_sniper_log("sniper service: starting");

    *slot = Some(SniperServiceState {
        started_at: SystemTime::now(),
        requested: true,
        stopping: false,
        shutdown: Some(shutdown_tx),
        thread: None,
    });
    drop(slot);

    let thread = thread::spawn(move || {
        let runtime = match tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(2)
            .thread_name("gnostr-sniper")
            .build()
        {
            Ok(runtime) => runtime,
            Err(error) => {
                    gnostr_crawler::record_sniper_lifecycle(format!(
                        "sniper lifecycle: runtime build failed: {}",
                        error
                    ));
                    gnostr_crawler::record_sniper_log(format!(
                        "sniper service: runtime build failed: {error}"
                    ));
                    return;
                }
            };

            gnostr_crawler::record_sniper_lifecycle("sniper lifecycle: worker spawned");
            let client = reqwest::Client::new();
            {
                let mut slot = sniper_service_slot().lock().unwrap();
                if let Some(state) = slot.as_mut() {
                    state.requested = false;
            }
        }
        runtime.block_on(async move {
            gnostr_crawler::run_sniper_service_with_shutdown(client, shutdown_rx).await;
        });
        gnostr_crawler::record_sniper_log("sniper service: server stopped");
    });

    let mut slot = sniper_service_slot().lock().unwrap();
    if let Some(state) = slot.as_mut() {
        state.thread = Some(thread);
    }
    drop(slot);
    gnostr_crawler::record_sniper_log("sniper service: running");

    "sniper service start requested".to_string()
}

#[uniffi::export]
pub fn sniper_service_stop() -> String {
    let mut slot = sniper_service_slot().lock().unwrap();
    let Some(state) = slot.as_mut() else {
        return "sniper service already stopped".to_string();
    };

    if state.thread.as_ref().is_some_and(thread::JoinHandle::is_finished) {
        slot.take();
        return "sniper service stopped".to_string();
    }

    gnostr_crawler::record_sniper_lifecycle("sniper lifecycle: stop requested");
    gnostr_crawler::record_sniper_log("sniper service: stopping");
    state.stopping = true;
    if let Some(shutdown) = state.shutdown.take() {
        let _ = shutdown.send(());
    }
    gnostr_crawler::record_sniper_log("sniper service: stopping requested");
    "sniper service stopping".to_string()
}
