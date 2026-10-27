use anyhow::Result;
use serde_json::{json, Value};

use crate::mcp::{McpClient, Tool};
use crate::openai::{LlmClient, Message};

/// Which GitHub toolset (and tool-name filter) to activate for a subcommand.
#[derive(Debug, Clone)]
pub enum ToolFilter {
    // Multi-toolset — needs client-side filtering
    Search,
    // Single-toolset — server filters, client passes all through
    Actions,
    CodeQuality,
    CodeSecurity,
    Context,
    Copilot,
    Dependabot,
    Discussions,
    Gists,
    Git,
    Issues,
    Labels,
    Notifications,
    Orgs,
    Projects,
    PullRequests,
    Repository,
    SecretProtection,
    SecurityAdvisories,
    Stargazers,
    Users,
}

impl ToolFilter {
    /// The `GITHUB_TOOLSETS` value to pass to the MCP server.
    pub fn toolsets(&self) -> &'static str {
        match self {
            Self::Search => "repos,issues,pull_requests",
            Self::Actions => "actions",
            Self::CodeQuality => "code_quality",
            Self::CodeSecurity => "code_security",
            Self::Context => "context",
            Self::Copilot => "copilot",
            Self::Dependabot => "dependabot",
            Self::Discussions => "discussions",
            Self::Gists => "gists",
            Self::Git => "git",
            Self::Issues => "issues",
            Self::Labels => "labels",
            Self::Notifications => "notifications",
            Self::Orgs => "orgs",
            Self::Projects => "projects",
            Self::PullRequests => "pull_requests",
            Self::Repository => "repos",
            Self::SecretProtection => "secret_protection",
            Self::SecurityAdvisories => "security_advisories",
            Self::Stargazers => "stargazers",
            Self::Users => "users",
        }
    }

    pub fn matches(&self, name: &str) -> bool {
        // For Search (multi-toolset load), filter client-side to search_* only.
        // All single-toolset filters let the server do the filtering.
        match self {
            Self::Search => name.starts_with("search_"),
            _ => true,
        }
    }
}

/// Full tool definition with complete inputSchema — used for actual tool calls.
fn tool_to_openai(tool: &Tool) -> Value {
    json!({
        "type": "function",
        "function": {
            "name": tool.name,
            "description": tool.description,
            "parameters": slim_schema(&tool.input_schema)
        }
    })
}

/// Strip per-property descriptions from a JSON schema to reduce token usage.
/// Keeps type/required/enum so the LLM still knows how to call each tool.
fn slim_schema(schema: &Value) -> Value {
    let mut s = schema.clone();
    if let Some(props) = s.get_mut("properties").and_then(|p| p.as_object_mut()) {
        for prop in props.values_mut() {
            if let Some(obj) = prop.as_object_mut() {
                obj.remove("description");
                obj.remove("examples");
                // Recurse into nested objects
                if let Some(inner) = obj.get("properties") {
                    let slimmed = slim_schema(inner);
                    obj.insert("properties".into(), slimmed);
                }
            }
        }
    }
    s
}

/// Ask the LLM (with just names + descriptions, no schemas) which tools are
/// relevant to the query. Returns a subset of tools.
async fn select_tools<'a>(query: &str, tools: &'a [Tool], llm: &LlmClient) -> Vec<&'a Tool> {
    let catalogue: Vec<String> = tools
        .iter()
        .map(|t| format!("- {}: {}", t.name, t.description.as_deref().unwrap_or("")))
        .collect();

    let prompt = format!(
        "Given this query: \"{query}\"\n\n\
         Available tools:\n{}\n\n\
         List only the tool names (comma-separated, no explanation) that are \
         needed to answer the query. Include no more than 8 tools.",
        catalogue.join("\n")
    );

    let msgs = vec![Message {
        role: "user".into(),
        content: Some(prompt),
        tool_calls: None,
        tool_call_id: None,
    }];

    // No tools in this call — just text response
    if let Ok(resp) = llm.chat(&msgs, &[]).await {
        if let Some(text) = resp.content {
            let selected: std::collections::HashSet<&str> =
                text.split(',').map(str::trim).collect();
            let filtered: Vec<&Tool> = tools
                .iter()
                .filter(|t| selected.contains(t.name.as_str()))
                .collect();
            if !filtered.is_empty() {
                println!(
                    "🎯 Selected {} relevant tools: {}",
                    filtered.len(),
                    filtered
                        .iter()
                        .map(|t| t.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                return filtered;
            }
        }
    }

    // Fallback: return first 10 tools
    tools.iter().take(10).collect()
}

pub async fn run(
    query: &str,
    github_token: &str,
    llm: LlmClient,
    filter: ToolFilter,
    explicit_tools: &[String],
) -> Result<String> {
    println!("🔌 Connecting to GitHub MCP server via Docker…");
    let mut mcp = McpClient::new(github_token, filter.toolsets()).await?;

    println!("📋 Loading available tools…");
    let all_tools = mcp.list_tools().await?;
    println!("✅ {} tools loaded", all_tools.len());

    // If the caller pinned specific tools, validate then use them directly.
    let relevant: Vec<&Tool> = if !explicit_tools.is_empty() {
        let available: std::collections::HashMap<&str, &Tool> =
            all_tools.iter().map(|t| (t.name.as_str(), t)).collect();

        let unknown: Vec<&str> = explicit_tools
            .iter()
            .filter(|n| !available.contains_key(n.as_str()))
            .map(String::as_str)
            .collect();

        if !unknown.is_empty() {
            let valid: Vec<&str> = all_tools
                .iter()
                .filter(|t| filter.matches(&t.name))
                .map(|t| t.name.as_str())
                .collect();
            anyhow::bail!(
                "Unknown tool(s): {}\n\nAvailable tools for this subcommand:\n  {}",
                unknown.join(", "),
                valid.join("\n  ")
            );
        }

        explicit_tools
            .iter()
            .filter_map(|n| available.get(n.as_str()).copied())
            .collect()
    } else {
        // Auto-select: pre-filter by category then let LLM narrow down if needed
        let category_tools: Vec<&Tool> = all_tools
            .iter()
            .filter(|t| filter.matches(&t.name))
            .collect();

        if category_tools.len() <= 8 {
            category_tools
        } else {
            select_tools(query, &all_tools, &llm).await
        }
    };
    let openai_tools: Vec<Value> = relevant.iter().map(|t| tool_to_openai(t)).collect();

    let mut messages = vec![
        Message {
            role: "system".into(),
            content: Some(
                "You are a GitHub assistant. Help users explore repositories and their activity.\n\
                 - Provide organized, concise insights about the repository\n\
                 - Focus on facts and data from the GitHub API\n\
                 - Use markdown formatting for better readability\n\
                 - Present numerical data in tables when appropriate\n\
                 - Include links to relevant GitHub pages when helpful"
                    .into(),
            ),
            tool_calls: None,
            tool_call_id: None,
        },
        Message {
            role: "user".into(),
            content: Some(query.into()),
            tool_calls: None,
            tool_call_id: None,
        },
    ];

    // Phase 2: agentic loop with the slim tool set
    loop {
        println!("🤔 Thinking…");
        let response = llm.chat(&messages, &openai_tools).await?;

        match response.tool_calls.as_deref() {
            Some(calls) if !calls.is_empty() => {
                let n = calls.len();
                messages.push(response.clone());

                for (i, tc) in calls.iter().enumerate() {
                    println!("🔧 [{}/{}] {}", i + 1, n, tc.function.name);
                    let args: Value =
                        serde_json::from_str(&tc.function.arguments).unwrap_or(json!({}));
                    let result = mcp
                        .call_tool(&tc.function.name, args)
                        .await
                        .unwrap_or_else(|e| format!("Tool error: {e}"));

                    messages.push(Message {
                        role: "tool".into(),
                        content: Some(result),
                        tool_calls: None,
                        tool_call_id: Some(tc.id.clone()),
                    });
                }
            }
            _ => return Ok(response.content.unwrap_or_default()),
        }
    }
}
