use serde::Serialize;
use std::path::PathBuf;
use warp::http::StatusCode;
use warp::Reply;
use std::fs;
use std::path::Path;

use crate::utils::detach::{
    capture_detached_pid, existing_detached_pid, kill_process_by_pid, listener_pid_on_port,
    relay_port_is_listening, spawn_detached_named_with_env,
};
use gnostr_relay::cli::RelayCli;

const RELAY_PID_NAME: &str = "gnostr-js-relay";
const RELAY_BINARY_NAME: &str = "gnostr-js-relay";
const RELAY_PORT: u16 = 8080;
const DETACHED_ENV: &str = "GNOSTR_JS_RELAY_DETACHED";

#[derive(Clone, Debug, Serialize)]
pub struct RelayProcessState {
    pub running: bool,
    pub pid: Option<u32>,
    pub message: String,
    pub disk_usage_bytes: Option<u64>,
}

fn relay_config_path() -> String {
    RelayCli::default().config_file_path
}

fn directory_usage_bytes(path: &Path) -> anyhow::Result<u64> {
    let mut total = 0u64;
    if !path.exists() {
        return Ok(0);
    }
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            total = total.saturating_add(directory_usage_bytes(&entry.path())?);
        } else {
            total = total.saturating_add(metadata.len());
        }
    }
    Ok(total)
}

fn relay_disk_usage_bytes() -> anyhow::Result<u64> {
    let config_file_path = relay_config_path();
    let config_path = Path::new(&config_file_path);
    let usage_path = config_path.parent().unwrap_or(config_path);
    directory_usage_bytes(usage_path)
}

fn relay_spawn_target() -> anyhow::Result<(PathBuf, bool)> {
    let current_exe = std::env::current_exe()?;
    let current_name = current_exe
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("")
        .to_string();
    if current_name == "gnostr-js" || current_name == "gnostr-js-relay" {
        return Ok((current_exe, current_name == "gnostr-js"));
    }

    let binary = current_exe
        .parent()
        .map(|dir| dir.join(RELAY_BINARY_NAME))
        .ok_or_else(|| anyhow::anyhow!("failed to resolve relay binary directory"))?;
    Ok((binary, false))
}

pub fn relay_status() -> anyhow::Result<RelayProcessState> {
    if let Some(pid) = existing_detached_pid(RELAY_PID_NAME)? {
        return Ok(RelayProcessState {
            running: true,
            pid: Some(pid),
            message: format!("relay already running with pid {pid}"),
            disk_usage_bytes: Some(relay_disk_usage_bytes()?),
        });
    }

    if relay_port_is_listening(RELAY_PORT) {
        let pid = listener_pid_on_port(RELAY_PORT)?;
        return Ok(RelayProcessState {
            running: true,
            pid,
            message: match pid {
                Some(pid) => format!("relay already listening on 127.0.0.1:{RELAY_PORT} with pid {pid}"),
                None => format!("relay already listening on 127.0.0.1:{RELAY_PORT}"),
            },
            disk_usage_bytes: Some(relay_disk_usage_bytes()?),
        });
    }

    Ok(RelayProcessState {
        running: false,
        pid: None,
        message: "relay stopped".to_string(),
        disk_usage_bytes: Some(relay_disk_usage_bytes()?),
    })
}

pub fn start_relay() -> anyhow::Result<RelayProcessState> {
    let status = relay_status()?;
    if status.running {
        return Ok(status);
    }

    let config_file_path = relay_config_path();
    let (relay_binary, needs_subcommand) = relay_spawn_target()?;
    let pid = if needs_subcommand {
        spawn_detached_named_with_env(
            relay_binary,
            Some(RELAY_PID_NAME),
            [
                "relay",
                "--logging",
                "info",
                "--config-file-path",
                config_file_path.as_str(),
            ],
            [(DETACHED_ENV, "1")],
        )?
    } else {
        spawn_detached_named_with_env(
            relay_binary,
            Some(RELAY_PID_NAME),
            [
                "--logging",
                "info",
                "--config-file-path",
                config_file_path.as_str(),
            ],
            [(DETACHED_ENV, "1")],
        )?
    };
    let _ = capture_detached_pid(RELAY_PID_NAME, pid)?;

    Ok(RelayProcessState {
        running: true,
        pid: Some(pid),
        message: format!("spawned detached relay pid {pid}"),
        disk_usage_bytes: Some(relay_disk_usage_bytes()?),
    })
}

pub fn stop_relay() -> anyhow::Result<RelayProcessState> {
    let status = relay_status()?;
    let Some(pid) = status.pid else {
        if status.running {
            return Err(anyhow::anyhow!(
                "relay is listening on 127.0.0.1:{RELAY_PORT} but its PID could not be determined"
            ));
        }
        return Ok(RelayProcessState {
            running: false,
            pid: None,
            message: "relay already stopped".to_string(),
            disk_usage_bytes: Some(relay_disk_usage_bytes()?),
        });
    };

    kill_process_by_pid(pid)?;

    Ok(RelayProcessState {
        running: false,
        pid: Some(pid),
        message: format!("stopped relay pid {pid}"),
        disk_usage_bytes: Some(relay_disk_usage_bytes()?),
    })
}

pub fn response(status: RelayProcessState, code: StatusCode) -> impl Reply {
    warp::reply::with_status(warp::reply::json(&status), code)
}
