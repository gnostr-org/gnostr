//! A small terminal web browser that views the crawler's on-disk buckets.
//!
//! It starts the crawler's local Axum API server (without the network-heavy
//! watch/sniper sidecars) so the on-disk buckets are served over HTTP, then
//! opens the `./tui` terminal browser pointed at that server. Quit the browser
//! (Ctrl+Q) to stop the server and exit.

use std::time::Duration;

use anyhow::Result;
use gnostr_crawler::api::{run_api_server_with_shutdown, ServeOptions};
use gnostr_tui::browser;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let port: u16 = std::env::var("GNOSTR_CRAWLER_BROWSER_PORT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(3000);

    let options = ServeOptions {
        port,
        no_watch: true,
        no_sniper: true,
        shitlist: None,
    };

    // Run the API server in the background. It is aborted when the browser exits.
    let server_task = tokio::spawn(async move {
        let _ = run_api_server_with_shutdown(options, std::future::pending::<()>()).await;
    });

    // Give the server a moment to bind and prepare the on-disk index.html.
    tokio::time::sleep(Duration::from_millis(500)).await;

    let url = format!("http://127.0.0.1:{}", port);
    let browser_result = browser::run_url(Some(url)).await;

    server_task.abort();
    let _ = server_task.await;

    browser_result
}
