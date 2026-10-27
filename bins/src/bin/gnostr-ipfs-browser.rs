//! Search for a running IPFS (Kubo) WebUI service and view it with the
//! `./tui` terminal browser.
//!
//! If no service is found, this binary re-executes itself in "serve" mode to
//! start an embedded Kubo node (via `kubo-ffi`) in a separate process, then
//! opens that node's WebUI in the browser. The node is stopped when the
//! browser exits.

use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use anyhow::{anyhow, Context, Result};

fn default_repo() -> String {
    std::env::var("GNOSTR_IPFS_REPO").unwrap_or_else(|_| {
        std::env::temp_dir()
            .join("gnostr-ipfs-browser")
            .to_string_lossy()
            .to_string()
    })
}

fn parse_host_port(multiaddr: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = multiaddr.split('/').collect();
    let mut host = None;
    let mut port = None;
    let mut i = 0;
    while i < parts.len() {
        if parts[i] == "ip4" && i + 1 < parts.len() {
            host = Some(parts[i + 1].to_string());
        }
        if parts[i] == "tcp" && i + 1 < parts.len() {
            port = Some(parts[i + 1].to_string());
        }
        i += 1;
    }
    Some((host?, port?))
}

fn serve_mode() -> Result<()> {
    let repo = default_repo();
    let _ = std::fs::create_dir_all(&repo);
    let _ = kubo_ffi::init_repo(&repo);

    let mut node = kubo_ffi::Node::start(&repo, true);
    if node.is_err() {
        let _ = std::fs::remove_file(std::path::Path::new(&repo).join("repo.lock"));
        node = kubo_ffi::Node::start(&repo, true);
    }
    let node = node.map_err(|e| anyhow!("failed to start Kubo node: {}", e))?;

    let api_multiaddr = node
        .start_api("/ip4/127.0.0.1/tcp/0")
        .ok_or_else(|| anyhow!("failed to start Kubo API: {:?}", kubo_ffi::last_error()))?;

    let (host, port) = parse_host_port(&api_multiaddr)
        .ok_or_else(|| anyhow!("could not parse Kubo API multiaddr: {}", api_multiaddr))?;

    let url = format!("http://{}:{}/webui", host, port);
    println!("IPFS_WEBUI_URL={}", url);
    use std::io::Write;
    let _ = std::io::stdout().flush();

    // Keep the node alive until killed.
    loop {
        std::thread::sleep(Duration::from_secs(3600));
    }
}

async fn wait_for_webui(url: &str, timeout: Duration) -> bool {
    let probe = match reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .redirect(reqwest::redirect::Policy::none())
        .build()
    {
        Ok(c) => c,
        Err(_) => return false,
    };
    let deadline = tokio::time::Instant::now() + timeout;
    while tokio::time::Instant::now() < deadline {
        for candidate in [url.to_string(), format!("{}/", url.trim_end_matches('/'))] {
            match probe.get(&candidate).send().await {
                Ok(resp) if resp.status().as_u16() < 400 => return true,
                _ => {}
            }
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    false
}

#[tokio::main]
async fn main() -> Result<()> {
    if std::env::var("GNOSTR_IPFS_BROWSER_SERVE").as_deref() == Ok("1") {
        return serve_mode();
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .context("failed to build HTTP client")?;

    let mut candidates: Vec<String> = Vec::new();

    if let Ok(value) = std::env::var("IPFS_WEBUI") {
        let value = value.trim().to_string();
        if !value.is_empty() {
            candidates.push(value);
        }
    }

    if let Ok(value) = std::env::var("IPFS_API") {
        let base = value.trim().trim_end_matches('/').to_string();
        if !base.is_empty() {
            candidates.push(format!("{}/webui", base));
        }
    }

    for port in 5001u16..=5010 {
        for host in ["127.0.0.1", "localhost"] {
            candidates.push(format!("http://{}:{}/webui", host, port));
        }
    }

    let mut seen = std::collections::HashSet::new();
    candidates.retain(|url| seen.insert(url.clone()));

    // Search for an already-running IPFS WebUI service.
    for url in &candidates {
        match client.get(url).send().await {
            Ok(response) if response.status().as_u16() < 400 => {
                println!("Found IPFS WebUI: {}", url);
                if !wait_for_webui(url, Duration::from_secs(10)).await {
                    println!("IPFS WebUI not ready yet, polling...");
                }
                return gnostr_tui::browser::run_url(Some(url.clone())).await;
            }
            _ => {}
        }
    }

    // None found: start an embedded node in a child process.
    println!("No running IPFS WebUI found; starting an embedded Kubo node...");

    let repo = default_repo();
    let exe = std::env::current_exe().context("could not locate current executable")?;

    let mut child: Child = Command::new(&exe)
        .env("GNOSTR_IPFS_BROWSER_SERVE", "1")
        .env("GNOSTR_IPFS_REPO", &repo)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .context("failed to spawn embedded IPFS node")?;

    let stdout = child.stdout.take().context("no stdout from child")?;
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();
    let mut url: Option<String> = None;
    loop {
        line.clear();
        let n = reader.read_line(&mut line).context("reading child stdout")?;
        if n == 0 {
            break;
        }
        let trimmed = line.trim().to_string();
        if let Some(rest) = trimmed.strip_prefix("IPFS_WEBUI_URL=") {
            url = Some(rest.to_string());
            break;
        }
        if !trimmed.is_empty() {
            println!("{}", trimmed);
        }
    }

    let url = url.ok_or_else(|| anyhow!("embedded IPFS node did not report a WebUI URL"))?;

    if !wait_for_webui(&url, Duration::from_secs(60)).await {
        let _ = child.kill();
        return Err(anyhow!("embedded IPFS WebUI did not become ready: {}", url));
    }
    println!("Embedded IPFS WebUI ready: {}", url);

    let browser_result = gnostr_tui::browser::run_url(Some(url)).await;

    let _ = child.kill();
    let _ = child.wait();

    browser_result
}
