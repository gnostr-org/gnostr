use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    path::PathBuf,
    process::Stdio,
    sync::{atomic::{AtomicU64, Ordering}, Mutex},
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStdin, Command},
};

use crate::docker;

static REQUEST_ID: AtomicU64 = AtomicU64::new(1);

static MCP_URL: Mutex<Option<String>> = Mutex::new(None);

pub fn set_mcp_url(url: String) {
    if let Ok(mut guard) = MCP_URL.lock() {
        *guard = Some(url);
    }
}

pub struct McpClient {
    transport: Transport,
}

enum Transport {
    Stdio {
        child: Child,
        stdin: ChildStdin,
        lines: tokio::io::Lines<BufReader<tokio::process::ChildStdout>>,
    },
    Http {
        url: String,
        http: reqwest::Client,
        token: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tool {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

impl McpClient {
    pub async fn new(github_token: &str, toolsets: &str) -> Result<Self> {
        let mcp_url = MCP_URL.lock().ok().and_then(|g| g.clone());
        if let Some(url) = mcp_url
            && !url.trim().is_empty()
        {
            let http = reqwest::Client::new();
            let mut client = Self {
                transport: Transport::Http {
                    url: url.trim_end_matches('/').to_string(),
                    http,
                    token: github_token.to_string(),
                },
            };
            client.initialize().await?;
            return Ok(client);
        }

        let docker: PathBuf = docker::ensure_ready().await?;

        let mut child = Command::new(&docker)
            .args([
                "run",
                "-i",
                "--rm",
                "-e",
                "GITHUB_PERSONAL_ACCESS_TOKEN",
                "-e",
                "GITHUB_TOOLSETS",
                &docker::image(),
                "stdio",
            ])
            .env("GITHUB_PERSONAL_ACCESS_TOKEN", github_token)
            .env("GITHUB_TOOLSETS", toolsets)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .with_context(|| format!("Failed to spawn {}", docker.display()))?;

        let stdin = child.stdin.take().context("Failed to get child stdin")?;
        let stdout = child.stdout.take().context("Failed to get child stdout")?;
        let lines = BufReader::new(stdout).lines();

        let mut client = Self {
            transport: Transport::Stdio { child, stdin, lines },
        };
        client.initialize().await?;
        Ok(client)
    }

    async fn send_request(&mut self, method: &str, params: Value) -> Result<Value> {
        match &mut self.transport {
            Transport::Stdio { stdin, lines, .. } => {
                let id = REQUEST_ID.fetch_add(1, Ordering::SeqCst);
                let request = json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params });
                let mut line = serde_json::to_string(&request)?;
                line.push('\n');
                stdin.write_all(line.as_bytes()).await?;
                stdin.flush().await?;

                loop {
                    let l = lines
                        .next_line()
                        .await?
                        .context("MCP server closed the connection unexpectedly")?;
                    if l.trim().is_empty() {
                        continue;
                    }
                    let resp: Value = serde_json::from_str(&l).context("Failed to parse MCP server response")?;
                    if resp.get("id").is_none() {
                        continue;
                    }
                    if resp["id"] == id {
                        if let Some(err) = resp.get("error") {
                            bail!("MCP error: {err}");
                        }
                        return Ok(resp["result"].clone());
                    }
                }
            }
            Transport::Http { url, http, token } => {
                let id = REQUEST_ID.fetch_add(1, Ordering::SeqCst);
                let request = json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params });
                let mut req = http.post(url.as_str()).json(&request);
                if !token.is_empty() {
                    req = req.header("Authorization", format!("Bearer {token}"));
                }
                let resp = req.send().await.context("Failed to send MCP request")?;
                let status = resp.status();
                let text = resp.text().await.context("Failed to read MCP response")?;
                if !status.is_success() {
                    bail!("MCP HTTP error {status}: {text}");
                }
                let parsed: Value = serde_json::from_str(&text).context("Failed to parse MCP response")?;
                if let Some(err) = parsed.get("error") {
                    bail!("MCP error: {err}");
                }
                Ok(parsed["result"].clone())
            }
        }
    }

    async fn send_notification(&mut self, method: &str) -> Result<()> {
        match &mut self.transport {
            Transport::Stdio { stdin, .. } => {
                let notif = json!({ "jsonrpc": "2.0", "method": method });
                let mut line = serde_json::to_string(&notif)?;
                line.push('\n');
                stdin.write_all(line.as_bytes()).await?;
                stdin.flush().await?;
                Ok(())
            }
            Transport::Http { url, http, token } => {
                let notif = json!({ "jsonrpc": "2.0", "method": method });
                let mut req = http.post(url.as_str()).json(&notif);
                if !token.is_empty() {
                    req = req.header("Authorization", format!("Bearer {token}"));
                }
                let _ = req.send().await;
                Ok(())
            }
        }
    }

    async fn initialize(&mut self) -> Result<()> {
        self.send_request(
            "initialize",
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": { "name": "github-mcp-agent-rs", "version": "0.1.0" }
            }),
        )
        .await?;
        self.send_notification("notifications/initialized").await
    }

    pub async fn list_tools(&mut self) -> Result<Vec<Tool>> {
        let result = self.send_request("tools/list", json!({})).await?;
        let tools: Vec<Tool> = serde_json::from_value(result["tools"].clone())
            .context("Failed to parse tools list from MCP server")?;
        Ok(tools)
    }

    pub async fn call_tool(&mut self, name: &str, arguments: Value) -> Result<String> {
        let result = self
            .send_request(
                "tools/call",
                json!({ "name": name, "arguments": arguments }),
            )
            .await?;

        if let Some(content) = result["content"].as_array() {
            let text = content
                .iter()
                .filter_map(|item| {
                    if item["type"] == "text" {
                        item["text"].as_str().map(str::to_owned)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
            Ok(text)
        } else {
            Ok(serde_json::to_string_pretty(&result)?)
        }
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        if let Transport::Stdio { child, .. } = &mut self.transport {
            let _ = child.start_kill();
        }
    }
}
