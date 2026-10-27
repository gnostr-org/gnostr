use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const OPENAI_BASE_URL: &str = "https://api.openai.com/v1";
pub const OLLAMA_BASE_URL: &str = "http://localhost:11434/v1";
pub const GITHUB_MODELS_BASE_URL: &str = "https://models.inference.ai.azure.com";
pub const OPENAI_DEFAULT_MODEL: &str = "gpt-4o-mini";
pub const OLLAMA_DEFAULT_MODEL: &str = "llama3.2";
pub const GITHUB_MODELS_DEFAULT_MODEL: &str = "gpt-4o-mini";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub function: FunctionCall,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

pub struct LlmClient {
    http: Client,
    base_url: String,
    /// None → no Authorization header (e.g. local Ollama)
    api_key: Option<String>,
    pub model: String,
}

impl LlmClient {
    pub fn new(
        base_url: impl Into<String>,
        api_key: Option<String>,
        model: impl Into<String>,
    ) -> Self {
        Self {
            http: Client::new(),
            base_url: base_url.into(),
            api_key,
            model: model.into(),
        }
    }

    /// Check whether the endpoint is reachable and the key (if any) is accepted.
    /// Returns Ok(()) on success, Err with status text on failure.
    pub async fn probe_result(&self) -> Result<()> {
        let url = format!("{}/models", self.base_url);
        let mut req = self.http.get(&url);
        if let Some(key) = &self.api_key {
            req = req.header("Authorization", format!("Bearer {key}"));
        }
        let resp = req
            .send()
            .await
            .with_context(|| format!("Cannot reach {url}"))?;
        if resp.status().is_success() {
            Ok(())
        } else {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("{status}: {body}");
        }
    }

    /// Lightweight reachability check (no auth) — used for Ollama.
    pub async fn probe(&self) -> bool {
        self.http
            .get(format!("{}/models", self.base_url))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    pub async fn chat(&self, messages: &[Message], tools: &[Value]) -> Result<Message> {
        let body = if tools.is_empty() {
            json!({ "model": self.model, "messages": messages })
        } else {
            json!({
                "model": self.model,
                "messages": messages,
                "tools": tools,
                "tool_choice": "auto"
            })
        };

        let url = format!("{}/chat/completions", self.base_url);
        let mut req = self.http.post(&url).json(&body);
        if let Some(key) = &self.api_key {
            req = req.header("Authorization", format!("Bearer {key}"));
        }

        let resp = req
            .send()
            .await
            .with_context(|| format!("Failed to reach LLM at {url}"))?;

        let status = resp.status();
        let text = resp.text().await?;

        if !status.is_success() {
            anyhow::bail!("LLM API error {status}: {text}");
        }

        let data: Value = serde_json::from_str(&text)?;
        let choice = &data["choices"][0]["message"];
        let msg: Message = serde_json::from_value(choice.clone())
            .context("Failed to parse LLM response message")?;
        Ok(msg)
    }
}
