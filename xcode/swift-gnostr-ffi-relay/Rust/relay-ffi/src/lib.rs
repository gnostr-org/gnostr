use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
    path::PathBuf,
    sync::{Mutex, OnceLock},
};

use actix_web::dev::ServerHandle;
use gnostr_relay::{cli::RelayCli, spawn_app_with_endpoint, App};
use serde::Serialize;
use tokio::runtime::{Builder, Runtime};

#[derive(Serialize)]
struct Envelope<T> {
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

impl<T> Envelope<T> {
    fn ok(data: T) -> Self {
        Self { ok: true, data: Some(data), error: None }
    }

    fn err(error: impl Into<String>) -> Self {
        Self { ok: false, data: None, error: Some(error.into()) }
    }
}

#[derive(Serialize)]
struct RelayCliView {
    logging: String,
    config_file_path: String,
}

#[derive(Serialize)]
struct RelayListenView {
    endpoint: String,
}

#[derive(Serialize, Clone)]
struct RelayProcessState {
    running: bool,
    pid: Option<u32>,
    message: String,
    disk_usage_bytes: Option<u64>,
}

struct RelayRuntimeState {
    handle: Option<ServerHandle>,
    state: RelayProcessState,
}

impl Default for RelayRuntimeState {
    fn default() -> Self {
        Self {
            handle: None,
            state: RelayProcessState {
                running: false,
                pid: None,
                message: "relay stopped".to_string(),
                disk_usage_bytes: None,
            },
        }
    }
}

static RELAY_RUNTIME: OnceLock<Runtime> = OnceLock::new();
static RELAY_STATE: OnceLock<Mutex<RelayRuntimeState>> = OnceLock::new();

fn to_c_string(json: String) -> *mut c_char {
    CString::new(json)
        .unwrap_or_else(|_| CString::new(r#"{"ok":false,"error":"embedded nul"}"#).unwrap())
        .into_raw()
}

fn encode<T: Serialize>(envelope: &Envelope<T>) -> *mut c_char {
    let json = serde_json::to_string(envelope)
        .unwrap_or_else(|error| format!(r#"{{"ok":false,"error":"failed to serialize response: {error}"}}"#));
    to_c_string(json)
}

fn relay_runtime() -> &'static Runtime {
    RELAY_RUNTIME.get_or_init(|| {
        Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to build relay runtime")
    })
}

fn relay_state() -> &'static Mutex<RelayRuntimeState> {
    RELAY_STATE.get_or_init(|| Mutex::new(RelayRuntimeState::default()))
}

fn relay_process_state(running: bool, message: impl Into<String>) -> RelayProcessState {
    RelayProcessState {
        running,
        pid: None,
        message: message.into(),
        disk_usage_bytes: None,
    }
}

fn current_setting_path() -> Option<PathBuf> {
    let current_dir = std::env::current_dir().ok()?;
    let config_path = current_dir.join(".gnostr/relay.toml");
    if config_path.exists() {
        Some(config_path)
    } else {
        None
    }
}

fn create_relay_app() -> Result<App, String> {
    let setting_path = current_setting_path();
    let app = App::create(
        setting_path.as_deref(),
        true,
        Some("NOSTR".to_owned()),
        None,
    )
    .map_err(|error| error.to_string())?;
    app.setting.write().add_nip(34);
    Ok(app)
}

fn start_relay_runtime() -> Result<RelayProcessState, String> {
    let mut guard = relay_state().lock().map_err(|error| error.to_string())?;
    if guard.state.running {
        return Ok(guard.state.clone());
    }

    let app = create_relay_app()?;
    let handle = relay_runtime()
        .block_on(async { spawn_app_with_endpoint(app).await.map_err(|error| error.to_string()) })?;
    guard.handle = Some(handle);
    guard.state = relay_process_state(true, "relay server started");
    Ok(guard.state.clone())
}

fn stop_relay_runtime() -> Result<RelayProcessState, String> {
    let handle = {
        let mut guard = relay_state().lock().map_err(|error| error.to_string())?;
        if !guard.state.running {
            return Ok(guard.state.clone());
        }
        guard.state = relay_process_state(false, "relay server stopping");
        guard.handle.take()
    };

    if let Some(handle) = handle {
        relay_runtime().block_on(async move {
            handle.stop(true).await;
        });
    }

    let mut guard = relay_state().lock().map_err(|error| error.to_string())?;
    guard.state = relay_process_state(false, "relay server stopped");
    Ok(guard.state.clone())
}

fn restart_relay_runtime() -> Result<RelayProcessState, String> {
    let _ = stop_relay_runtime()?;
    start_relay_runtime()
}

fn read_relay_runtime_status() -> Result<RelayProcessState, String> {
    let guard = relay_state().lock().map_err(|error| error.to_string())?;
    Ok(guard.state.clone())
}

unsafe fn read_c_string<'a>(ptr: *const c_char) -> Result<&'a str, String> {
    if ptr.is_null() {
        return Err("null pointer".to_string());
    }
    CStr::from_ptr(ptr).to_str().map_err(|error| error.to_string())
}

#[no_mangle]
pub unsafe extern "C" fn relay_string_free(ptr: *mut c_char) {
    if !ptr.is_null() {
        drop(CString::from_raw(ptr));
    }
}

#[no_mangle]
pub unsafe extern "C" fn relay_default_configuration_json(_unused: *const c_char) -> *mut c_char {
    let defaults = RelayCli::default();
    let view = RelayCliView {
        logging: defaults.logging,
        config_file_path: defaults.config_file_path,
    };
    encode(&Envelope::ok(view))
}

#[no_mangle]
pub unsafe extern "C" fn relay_listen_endpoint_json(host: *const c_char, port: u16) -> *mut c_char {
    let host = match read_c_string(host) {
        Ok(value) => value,
        Err(error) => return encode(&Envelope::<String>::err(error)),
    };

    let normalized_host = match host.parse::<std::net::IpAddr>() {
        Ok(addr) if addr.is_unspecified() => "127.0.0.1".to_string(),
        Ok(addr) => addr.to_string(),
        Err(_) if host.is_empty() => "127.0.0.1".to_string(),
        Err(_) => host.to_string(),
    };

    encode(&Envelope::ok(RelayListenView {
        endpoint: format!("ws://{normalized_host}:{port}"),
    }))
}

#[no_mangle]
pub unsafe extern "C" fn relay_status_json(_unused: *const c_char) -> *mut c_char {
    encode_process_state(read_relay_runtime_status())
}

#[no_mangle]
pub unsafe extern "C" fn relay_start_json(_unused: *const c_char) -> *mut c_char {
    encode_process_state(start_relay_runtime())
}

#[no_mangle]
pub unsafe extern "C" fn relay_stop_json(_unused: *const c_char) -> *mut c_char {
    encode_process_state(stop_relay_runtime())
}

#[no_mangle]
pub unsafe extern "C" fn relay_restart_json(_unused: *const c_char) -> *mut c_char {
    encode_process_state(restart_relay_runtime())
}

fn encode_process_state(result: Result<RelayProcessState, String>) -> *mut c_char {
    match result {
        Ok(state) => encode(&Envelope::ok(state)),
        Err(error) => encode(&Envelope::<RelayProcessState>::err(error)),
    }
}
