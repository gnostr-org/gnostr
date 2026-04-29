use serde::Serialize;
use std::path::PathBuf;
use warp::http::StatusCode;
use warp::Reply;

use crate::utils::detach::{
    capture_detached_pid, existing_detached_pid, relay_port_is_listening,
    spawn_detached_current_exe_named,
};

const CRAWLER_PID_NAME: &str = "gnostr-js-crawler";

#[derive(Clone, Debug, Serialize)]
pub struct CrawlerProcessState {
    pub running: bool,
    pub pid: Option<u32>,
    pub message: String,
}

pub fn crawler_status(port: u16) -> anyhow::Result<CrawlerProcessState> {
    if let Some(pid) = existing_detached_pid(CRAWLER_PID_NAME)? {
        return Ok(CrawlerProcessState {
            running: true,
            pid: Some(pid),
            message: format!("crawler already running with pid {pid}"),
        });
    }

    if relay_port_is_listening(port) {
        return Ok(CrawlerProcessState {
            running: true,
            pid: None,
            message: format!("crawler already listening on 127.0.0.1:{port}"),
        });
    }

    Ok(CrawlerProcessState {
        running: false,
        pid: None,
        message: "crawler stopped".to_string(),
    })
}

pub fn start_crawler(port: u16) -> anyhow::Result<CrawlerProcessState> {
    let status = crawler_status(port)?;
    if status.running {
        return Ok(status);
    }

    let port = port.to_string();
    let pid = spawn_detached_current_exe_named(
        Some("gnostr-js"),
        ["crawler", "--port", port.as_str()],
    )?;
    let _ = capture_detached_pid(CRAWLER_PID_NAME, pid)?;

    Ok(CrawlerProcessState {
        running: true,
        pid: Some(pid),
        message: format!("spawned detached crawler pid {pid}"),
    })
}

pub fn response(status: CrawlerProcessState, code: StatusCode) -> impl Reply {
    warp::reply::with_status(warp::reply::json(&status), code)
}
