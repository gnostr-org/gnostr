use serde::Serialize;
use warp::http::StatusCode;
use warp::Reply;

use crate::utils::detach::{
    capture_detached_pid, existing_detached_pid, kill_process_by_pid, listener_pid_on_port,
    relay_port_is_listening, spawn_detached_current_exe_named,
};
use gnostr_relay::cli::RelayCli;

const RELAY_PID_NAME: &str = "gnostr-js-relay";
const RELAY_PORT: u16 = 8080;

#[derive(Clone, Debug, Serialize)]
pub struct RelayProcessState {
    pub running: bool,
    pub pid: Option<u32>,
    pub message: String,
}

fn relay_config_path() -> String {
    RelayCli::default().config_file_path
}

pub fn relay_status() -> anyhow::Result<RelayProcessState> {
    if let Some(pid) = existing_detached_pid(RELAY_PID_NAME)? {
        return Ok(RelayProcessState {
            running: true,
            pid: Some(pid),
            message: format!("relay already running with pid {pid}"),
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
        });
    }

    Ok(RelayProcessState {
        running: false,
        pid: None,
        message: "relay stopped".to_string(),
    })
}

pub fn start_relay() -> anyhow::Result<RelayProcessState> {
    let status = relay_status()?;
    if status.running {
        return Ok(status);
    }

    let config_file_path = relay_config_path();
    let pid = spawn_detached_current_exe_named(
        Some("gnostr-js"),
        [
            "relay",
            "--logging",
            "info",
            "--config-file-path",
            config_file_path.as_str(),
        ],
    )?;
    let _ = capture_detached_pid(RELAY_PID_NAME, pid)?;

    Ok(RelayProcessState {
        running: true,
        pid: Some(pid),
        message: format!("spawned detached relay pid {pid}"),
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
        });
    };

    kill_process_by_pid(pid)?;

    Ok(RelayProcessState {
        running: false,
        pid: Some(pid),
        message: format!("stopped relay pid {pid}"),
    })
}

pub fn response(status: RelayProcessState, code: StatusCode) -> impl Reply {
    warp::reply::with_status(warp::reply::json(&status), code)
}
