use anyhow::{bail, Result};
use clap::{Args, Parser, Subcommand};
use std::{env, io::Write};

use crate::agent::ToolFilter;
use crate::openai::{
    LlmClient, OLLAMA_BASE_URL, OLLAMA_DEFAULT_MODEL, OPENAI_BASE_URL, OPENAI_DEFAULT_MODEL,
};
use crate::agent;

#[derive(Parser)]
#[command(name = "github-mcp-agent")]
#[command(about = "🐙 Gnostr MCP Agent — explore GitHub repos with natural language")]
#[command(arg_required_else_help = true)]
#[command(after_help = "Any GitHub MCP tool name can also be used as a subcommand, e.g. `search_code '{\"query\":\"rust\"}'`. Run `tools` to list available tool names.")]
#[command(disable_help_subcommand = true)]
struct Cli {
    /// GitHub Personal Access Token (or set GITHUB_TOKEN env var)
    #[arg(long, global = true)]
    github_token: Option<String>,

    /// OpenAI API Key — optional (or set OPENAI_API_KEY env var)
    #[arg(long, global = true)]
    openai_key: Option<String>,

    /// LLM base URL — e.g. http://localhost:11434/v1 (or set LLM_BASE_URL env var)
    #[arg(long, global = true)]
    llm_url: Option<String>,

    /// Model name override (or set LLM_MODEL env var)
    #[arg(long, global = true)]
    model: Option<String>,

    /// Launch the Docker container TUI (oxker) to view containers
    #[arg(long, global = true)]
    tui: bool,

    /// Force use of docker CLI when execing into containers (passed to the TUI)
    #[arg(long, global = true)]
    use_cli: bool,

    /// MCP server URL for HTTP transport (instead of Docker stdio)
    #[arg(long, global = true)]
    mcp_url: Option<String>,

    /// List all available MCP tools grouped by category (no LLM required)
    #[arg(long)]
    list_tools: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

/// Shared args for all query subcommands
#[derive(Args)]
struct QueryArgs {
    /// Repository (owner/repo) — required for most toolsets
    #[arg(short, long, default_value = "gnostr-org/gnostr")]
    repo: String,

    /// Natural language query
    #[arg(short, long)]
    query: Option<String>,

    /// List tools available for this subcommand (no LLM required)
    #[arg(long)]
    list_tools: bool,

    /// Restrict to specific tools (comma-separated, validated against --list-tools)
    /// Pass without a value to list available tools for this subcommand.
    #[arg(long, num_args(0..), value_delimiter = ',')]
    tools: Option<Vec<String>>,
}

#[derive(Subcommand)]
enum Command {
    /// GitHub Actions workflows and CI/CD operations
    Actions(QueryArgs),
    /// Call a single MCP tool directly
    #[command(name = "call")]
    Call {
        tool: Option<String>,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Code quality tools
    #[command(name = "code-quality")]
    CodeQuality(QueryArgs),
    /// Code security — GitHub Code Scanning
    #[command(name = "code-security")]
    CodeSecurity(QueryArgs),
    /// Current user and GitHub context (strongly recommended for context)
    Context(QueryArgs),
    /// Copilot related tools
    Copilot(QueryArgs),
    /// Dependabot tools
    Dependabot(QueryArgs),
    /// GitHub Discussions
    Discussions(QueryArgs),
    /// Docker container TUI (requires --features docker-tui)
    #[cfg(feature = "docker-tui")]
    #[command(name = "docker-tui", disable_help_flag = true)]
    DockerTui {
        /// Args forwarded to oxker (use --help to see oxker's own help)
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// GitHub Gists
    Gists(QueryArgs),
    /// Low-level Git API operations
    Git(QueryArgs),
    /// Show help/schema for a tool
    #[command(name = "help")]
    Help {
        tool: Option<String>,
    },
    /// GitHub Issues
    Issues(QueryArgs),
    /// GitHub Labels
    Labels(QueryArgs),
    /// GitHub Notifications
    Notifications(QueryArgs),
    /// GitHub Organizations
    Orgs(QueryArgs),
    /// GitHub Projects
    Projects(QueryArgs),
    /// GitHub Pull Requests
    #[command(name = "prs")]
    PullRequests(QueryArgs),
    /// Repository files, branches, commits, releases, tags
    #[command(name = "repo")]
    Repository(QueryArgs),
    /// Search code, commits, issues, PRs, or repositories
    Search(QueryArgs),
    /// Secret protection — GitHub Secret Scanning
    #[command(name = "secret-protection")]
    SecretProtection(QueryArgs),
    /// Security advisories
    #[command(name = "security-advisories")]
    SecurityAdvisories(QueryArgs),
    /// GitHub Stargazers
    Stargazers(QueryArgs),
    /// List all available MCP tools grouped by category (no LLM required)
    #[command(name = "tools")]
    Tools,
    /// Docker container TUI to view containers
    #[command(name = "tui")]
    Tui,

    /// Docker helpers (TUI)
    #[command(name = "docker")]
    Docker,
    /// GitHub Users
    Users(QueryArgs),
    /// Pass-through to docker for unrecognized subcommands
    #[command(external_subcommand)]
    External(Vec<String>),
}

impl Command {
    fn filter(&self) -> ToolFilter {
        match self {
            Self::Actions(_) => ToolFilter::Actions,
            Self::CodeQuality(_) => ToolFilter::CodeQuality,
            Self::CodeSecurity(_) => ToolFilter::CodeSecurity,
            Self::Context(_) => ToolFilter::Context,
            Self::Copilot(_) => ToolFilter::Copilot,
            Self::Dependabot(_) => ToolFilter::Dependabot,
            Self::Discussions(_) => ToolFilter::Discussions,
            Self::Gists(_) => ToolFilter::Gists,
            Self::Git(_) => ToolFilter::Git,
            Self::Issues(_) => ToolFilter::Issues,
            Self::Labels(_) => ToolFilter::Labels,
            Self::Notifications(_) => ToolFilter::Notifications,
            Self::Orgs(_) => ToolFilter::Orgs,
            Self::Projects(_) => ToolFilter::Projects,
            Self::PullRequests(_) => ToolFilter::PullRequests,
            Self::Repository(_) => ToolFilter::Repository,
            Self::SecretProtection(_) => ToolFilter::SecretProtection,
            Self::SecurityAdvisories(_) => ToolFilter::SecurityAdvisories,
            Self::Search(_) => ToolFilter::Search,
            Self::Stargazers(_) => ToolFilter::Stargazers,
            Self::Users(_) => ToolFilter::Users,
            Self::Tools => unreachable!(),
            Self::Tui => unreachable!(),
            Self::Docker => unreachable!(),
            Self::Call { .. } => unreachable!(),
            Self::Help { .. } => unreachable!(),
            Self::External(_) => unreachable!(),
            #[cfg(feature = "docker-tui")]
            Self::DockerTui { .. } => unreachable!(),
        }
    }

    fn args(self) -> QueryArgs {
        match self {
            Self::Actions(a)
            | Self::CodeQuality(a)
            | Self::CodeSecurity(a)
            | Self::Context(a)
            | Self::Copilot(a)
            | Self::Dependabot(a)
            | Self::Discussions(a)
            | Self::Gists(a)
            | Self::Git(a)
            | Self::Issues(a)
            | Self::Labels(a)
            | Self::Notifications(a)
            | Self::Orgs(a)
            | Self::Projects(a)
            | Self::PullRequests(a)
            | Self::Repository(a)
            | Self::SecretProtection(a)
            | Self::SecurityAdvisories(a)
            | Self::Search(a)
            | Self::Stargazers(a)
            | Self::Users(a) => a,
            Self::Tools => unreachable!(),
            Self::Tui => unreachable!(),
            Self::Docker => unreachable!(),
            Self::Call { .. } => unreachable!(),
            Self::Help { .. } => unreachable!(),
            Self::External(_) => unreachable!(),
            #[cfg(feature = "docker-tui")]
            Self::DockerTui { .. } => unreachable!(),
        }
    }

    fn tools_flag(&self) -> Option<&Vec<String>> {
        match self {
            Self::Actions(a) | Self::CodeQuality(a) | Self::CodeSecurity(a)
            | Self::Context(a) | Self::Copilot(a) | Self::Dependabot(a)
            | Self::Discussions(a) | Self::Gists(a) | Self::Git(a)
            | Self::Issues(a) | Self::Labels(a) | Self::Notifications(a)
            | Self::Orgs(a) | Self::Projects(a) | Self::PullRequests(a)
            | Self::Repository(a) | Self::SecretProtection(a)
            | Self::SecurityAdvisories(a) | Self::Search(a)
            | Self::Stargazers(a) | Self::Users(a) => a.tools.as_ref(),
            Self::Tools => None,
            Self::Tui => None,
            Self::Docker => None,
            Self::Call { .. } => None,
            Self::Help { .. } => None,
            Self::External(_) => None,
            #[cfg(feature = "docker-tui")]
            Self::DockerTui { .. } => None,
        }
    }

    fn list_tools_flag(&self) -> bool {
        match self {
            Self::Actions(a)
            | Self::CodeQuality(a)
            | Self::CodeSecurity(a)
            | Self::Context(a)
            | Self::Copilot(a)
            | Self::Dependabot(a)
            | Self::Discussions(a)
            | Self::Gists(a)
            | Self::Git(a)
            | Self::Labels(a)
            | Self::Notifications(a)
            | Self::Orgs(a)
            | Self::Projects(a)
            | Self::PullRequests(a)
            | Self::Repository(a)
            | Self::SecretProtection(a)
            | Self::SecurityAdvisories(a)
            | Self::Search(a)
            | Self::Stargazers(a)
            | Self::Users(a)
            | Self::Issues(a) => a.list_tools,
            Self::Tools => false,
            Self::Tui => false,
            Self::Docker => false,
            Self::Call { .. } => false,
            Self::Help { .. } => false,
            Self::External(_) => false,
            #[cfg(feature = "docker-tui")]
            Self::DockerTui { .. } => false,
        }
    }
}

pub async fn run() -> Result<()> {
    run_from(std::env::args().collect()).await
}

pub async fn run_from(mut raw_args: Vec<String>) -> Result<()> {
    // Ensure a program name is present for clap parsing.
    if raw_args.is_empty() || !raw_args[0].contains("gnostr") && !raw_args[0].contains("docker") && !raw_args[0].contains("mcp") {
        raw_args.insert(0, "gnostr-docker".to_string());
    }

    // Pre-handle `call <tool> --help|-h|help` so the tool schema is shown
    // instead of the generic clap help.
    let raw_args = raw_args;
    if let Some(call_pos) = raw_args.iter().position(|a| a == "call") {
        if raw_args.len() > call_pos + 2 {
            let tool = &raw_args[call_pos + 1];
            let flag = &raw_args[call_pos + 2];
            if flag == "--help" || flag == "-h" || flag == "help" {
                let github_token = raw_args
                    .iter()
                    .position(|a| a == "--github-token")
                    .and_then(|i| raw_args.get(i + 1))
                    .cloned()
                    .unwrap_or_default();
                if !github_token.is_empty() {
                    if let Ok(mut mcp) = crate::mcp::McpClient::new(&github_token, "all").await {
                        if let Ok(tools) = mcp.list_tools().await
                            && let Some(t) = tools.iter().find(|t| t.name == *tool)
                        {
                            println!("{}", serde_json::to_string_pretty(&t.input_schema).unwrap_or_default());
                            return Ok(());
                        }
                    }
                }
            }
        }
    }

    let cli = Cli::parse_from(raw_args.clone());

    let github_token = cli
        .github_token
        .or_else(|| env::var("GITHUB_TOKEN").ok())
        .unwrap_or_default();

    if let Some(url) = &cli.mcp_url {
        crate::mcp::set_mcp_url(url.clone());
    }

    println!("🐙 Gnostr MCP Agent");

    let is_tools_cmd = matches!(cli.command, Some(Command::Tools));
    #[cfg(feature = "docker-tui")]
    let is_docker_tui = matches!(cli.command, Some(Command::DockerTui { .. }));
    #[cfg(not(feature = "docker-tui"))]
    let is_docker_tui = false;

    let is_tui = matches!(cli.command, Some(Command::Tui));
    let is_docker = matches!(cli.command, Some(Command::Docker));
    let is_external = matches!(cli.command, Some(Command::External(_)));
    let is_call = matches!(cli.command, Some(Command::Call { .. }));
    let is_help = matches!(cli.command, Some(Command::Help { .. }));

    if is_external {
        if let Some(Command::External(args)) = cli.command {
            // If the first token is "help", list all tools.
            if !github_token.is_empty() && !args.is_empty() && args[0] == "help" {
                let _ = list_tools(&github_token, None).await;
                return Ok(());
            }
            // If the first token matches an MCP tool name, call it directly.
            if !github_token.is_empty() && !args.is_empty() {
                if let Ok(mut mcp) = crate::mcp::McpClient::new(&github_token, "all").await {
                    if let Ok(tools) = mcp.list_tools().await {
                        if tools.iter().any(|t| t.name == args[0]) {
                            if args.len() >= 2 && (args[1] == "--help" || args[1] == "-h" || args[1] == "help") {
                                if let Ok(mut mcp2) = crate::mcp::McpClient::new(&github_token, "all").await {
                                    if let Ok(tools) = mcp2.list_tools().await
                                        && let Some(t) = tools.iter().find(|t| t.name == args[0])
                                    {
                                        println!("{}", serde_json::to_string_pretty(&t.input_schema).unwrap_or_default());
                                        return Ok(());
                                    }
                                }
                            }
                            let raw = args[1..].join(" ");
                            let mut arguments = serde_json::Value::Object(serde_json::Map::new());
                            if !raw.trim().is_empty() {
                                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
                                    arguments = v;
                                } else {
                                    let mut map = serde_json::Map::new();
                                    for pair in &args[1..] {
                                        if let Some((k, v)) = pair.split_once('=') {
                                            map.insert(k.to_string(), serde_json::Value::String(v.to_string()));
                                        }
                                    }
                                    if !map.is_empty() {
                                        arguments = serde_json::Value::Object(map);
                                    }
                                }
                            }
                            match mcp.call_tool(&args[0], arguments).await {
                                Ok(result) => {
                                    println!("{}", result);
                                    return Ok(());
                                }
                                Err(e) => {
                                    eprintln!("{}", e);
                                    return Ok(());
                                }
                            }
                        }
                    }
                }
            }
            let status = std::process::Command::new("docker")
                .args(&args)
                .status();
            match status {
                Ok(s) => std::process::exit(s.code().unwrap_or(0)),
                Err(e) => {
                    eprintln!("Failed to run docker: {e}");
                    std::process::exit(1);
                }
            }
        }
    }

    if is_help {
        if github_token.is_empty() {
            bail!("GitHub token required. Set GITHUB_TOKEN env var or use --github-token.");
        }
        if let Some(Command::Help { tool }) = cli.command {
            let tool = match tool {
                Some(t) => t,
                None => {
                    list_tools(&github_token, None).await?;
                    return Ok(());
                }
            };
            let mut mcp = crate::mcp::McpClient::new(&github_token, "all").await?;
            let tools = mcp.list_tools().await?;
            if let Some(t) = tools.iter().find(|t| t.name == tool) {
                println!("{}", serde_json::to_string_pretty(&t.input_schema).unwrap_or_default());
            } else {
                println!("Tool '{}' not found.", tool);
            }
        }
        return Ok(());
    }

    if is_call {
        if let Some(Command::Call { tool, args }) = cli.command {
            let Some(tool) = tool else {
                list_tools(&github_token, None).await?;
                return Ok(());
            };
            if tool == "help" {
                list_tools(&github_token, None).await?;
                return Ok(());
            }
            if github_token.is_empty() {
                bail!("GitHub token required. Set GITHUB_TOKEN env var or use --github-token.");
            }
            if args == ["help"] || args == ["--help"] || args == ["-h"] {
                let mut mcp = crate::mcp::McpClient::new(&github_token, "all").await?;
                let tools = mcp.list_tools().await?;
                if let Some(t) = tools.iter().find(|t| t.name == tool) {
                    println!("{}", serde_json::to_string_pretty(&t.input_schema).unwrap_or_default());
                } else {
                    println!("Tool '{}' not found.", tool);
                }
                return Ok(());
            }
            let raw = args.join(" ");
            let mut arguments = serde_json::Value::Object(serde_json::Map::new());
            if !raw.trim().is_empty() {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
                    arguments = v;
                } else {
                    // Parse key=value pairs
                    let mut map = serde_json::Map::new();
                    for pair in args {
                        if let Some((k, v)) = pair.split_once('=') {
                            map.insert(k.to_string(), serde_json::Value::String(v.to_string()));
                        }
                    }
                    if !map.is_empty() {
                        arguments = serde_json::Value::Object(map);
                    }
                }
            }
            let mut mcp = crate::mcp::McpClient::new(&github_token, "all").await?;
            match mcp.call_tool(&tool, arguments).await {
                Ok(result) => {
                    println!("{}", result);
                }
                Err(e) => {
                    eprintln!("{}", e);
                }
            }
        }
        return Ok(());
    }

    if is_docker {
        return crate::sub_commands::docker::run(cli.use_cli).await;
    }

    if is_tui {
        println!("🐙 Gnostr MCP Agent — Docker TUI");
        if !github_token.is_empty() {
            println!("🔌 Connecting to GitHub MCP server via Docker…");
            let docker = crate::docker::ensure_ready().await?;
            crate::docker::start_mcp_server(&docker, &github_token).await?;
        }
        let mut oxker_args = Vec::new();
        if cli.use_cli {
            oxker_args.push("--use-cli".to_string());
        }
        crate::setup_tracing();
        crate::run_with_args(&oxker_args, Some(crate::docker::CONTAINER_NAME.to_string())).await;
        return Ok(());
    }

    if cli.tui {
        println!("🐙 Gnostr MCP Agent — Docker TUI");
        if !github_token.is_empty() {
            println!("🔌 Connecting to GitHub MCP server via Docker…");
            let docker = crate::docker::ensure_ready().await?;
            crate::docker::start_mcp_server(&docker, &github_token).await?;
        }
        let mut oxker_args = Vec::new();
        if cli.use_cli {
            oxker_args.push("--use-cli".to_string());
        }
        crate::setup_tracing();
        crate::run_with_args(&oxker_args, Some(crate::docker::CONTAINER_NAME.to_string())).await;
        return Ok(());
    }

    if is_docker_tui {
        #[cfg(feature = "docker-tui")]
        if let Some(Command::DockerTui { args }) = cli.command {
            let is_help    = args.iter().any(|a| a == "--help"    || a == "-h");
            let is_version = args.iter().any(|a| a == "--version" || a == "-V");
            if is_version {
                println!("github-mcp-agent {} (docker-tui / oxker {})",
                    env!("CARGO_PKG_VERSION"), "0.13.2");
                return Ok(());
            }
            if is_help {
                let sep = "─".repeat(72);
                let cname = crate::docker::CONTAINER_NAME;
                println!(
                    "\n\
                     docker-tui — GitHub MCP Agent wrapper for the oxker Docker TUI\n\
                     {sep}\n\
                     Ensures the GitHub MCP server image is present, optionally starts\n\
                     a named '{cname}' container, then launches the oxker TUI.\n\
                     \n\
                     Usage:\n\
                     \\x20 github-mcp-agent [--github-token <TOKEN>] docker-tui [OPTIONS]\n\
                     \n\
                     Global flags (before docker-tui):\n\
                     {sep}\n\
                     \\x20 {:<36} Start '{cname}' container\n\
                     \\x20 {:<36} (or set GITHUB_TOKEN env var)\n\
                     \n\
                     Options:\n\
                     {sep}\n\
                     \\x20 {:<36} Docker update interval in ms [default: 1000]\n\
                     \\x20 {:<36} Remove timestamps from Docker logs\n\
                     \\x20 {:<36} Colorize logs (conflicts with -c)\n\
                     \\x20 {:<36} Raw logs, remove ansi formatting (conflicts with -c)\n\
                     \\x20 {:<36} Show self when running as a Docker container\n\
                     \\x20 {:<36} Don't draw GUI (debug mode)\n\
                     \\x20 {:<36} Docker host [default: /var/run/docker.sock]\n\
                     \\x20 {:<36} Do not include stderr output in logs\n\
                     \\x20 {:<36} Display log timestamps in given timezone [default: UTC]\n\
                     \\x20 {:<36} Directory for saving exported logs [default: $HOME]\n\
                     \\x20 {:<36} Path to a config file (TOML, JSONC, or JSON)\n\
                     \\x20 {:<36} Force use of docker CLI when execing into containers\n\
                     \\x20 {:<36} Print this help\n\
                     \\x20 {:<36} Print version\n\
                     \n\
                     TUI keybindings:\n\
                     {sep}\n\
                     \\x20 {:<36} Exec / log into selected container\n\
                     \\x20 {:<36} Quit\n\
                     \\x20 {:<36} Filter containers\n\
                     \\x20 {:<36} Inspect container details\n\
                     {sep}",
                    "--github-token <TOKEN>",
                    "",
                    "-d <ms>",
                    "-t",
                    "-c",
                    "-r",
                    "-s",
                    "-g",
                    "--host <HOST>",
                    "--no-stderr",
                    "--timezone <TIMEZONE>",
                    "--save-dir <DIR>",
                    "--config-file <PATH>",
                    "--use-cli",
                    "-h, --help",
                    "-V, --version",
                    "t / T",
                    "q",
                    "/",
                    "i",
                );
                return Ok(());
            }
            println!("🔌 Connecting to GitHub MCP server via Docker…");
            let docker = crate::docker::ensure_ready().await?;
            if !github_token.is_empty() {
                crate::docker::start_mcp_server(&docker, &github_token).await?;
            }
            let mut fwd = args.clone();
            if cli.use_cli {
                fwd.push("--use-cli".to_string());
            }
            crate::setup_tracing();
            crate::run_with_args(&fwd, Some(crate::docker::CONTAINER_NAME.to_string())).await;
            return Ok(());
        }
    }

    if cli.list_tools || is_tools_cmd {
        return list_tools(&github_token, None).await;
    }

    let Some(cmd) = cli.command else {
        bail!("A subcommand is required. Use --help for usage.");
    };

    // Subcommand --list-tools OR bare --tools (no values) → show filtered tools
    if cmd.list_tools_flag() || matches!(cmd.tools_flag(), Some(t) if t.is_empty()) {
        let filter = cmd.filter();
        return list_tools(&github_token, Some(&filter)).await;
    }

    if github_token.is_empty() {
        bail!("GitHub token required. Set GITHUB_TOKEN env var or use --github-token.");
    }

    let openai_key = cli.openai_key.or_else(|| env::var("OPENAI_API_KEY").ok());
    let llm_url = cli.llm_url.or_else(|| env::var("LLM_BASE_URL").ok());
    let model = cli.model.or_else(|| env::var("LLM_MODEL").ok());
    let llm = resolve_llm(&github_token, openai_key, llm_url, model).await?;

    let filter = cmd.filter();
    let args = cmd.args();

    let query = match args.query {
        Some(q) => q,
        None => {
            print!("Query: ");
            std::io::stdout().flush()?;
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            input.trim().to_string()
        }
    };

    let full_query = if query.contains(&args.repo) {
        query
    } else {
        format!("{} in {}", query, args.repo)
    };

    println!("Repository : {}", args.repo);
    println!("Query      : {}", full_query);
    println!("{}", "─".repeat(60));

    let explicit_tools = args.tools.as_deref().unwrap_or(&[]);
    let result = agent::run(&full_query, &github_token, llm, filter, explicit_tools).await?;
    println!("\n### Results\n{}", result);

    Ok(())
}

async fn list_tools(github_token: &str, filter: Option<&ToolFilter>) -> Result<()> {
    let toolsets = filter.map(|f| f.toolsets()).unwrap_or("all");
    let label = filter.map(|f| match f {
        ToolFilter::Actions => "Actions",
        ToolFilter::CodeQuality => "Code Quality",
        ToolFilter::CodeSecurity => "Code Security",
        ToolFilter::Context => "Context",
        ToolFilter::Copilot => "Copilot",
        ToolFilter::Dependabot => "Dependabot",
        ToolFilter::Discussions => "Discussions",
        ToolFilter::Gists => "Gists",
        ToolFilter::Git => "Git",
        ToolFilter::Issues => "Issues",
        ToolFilter::Labels => "Labels",
        ToolFilter::Notifications => "Notifications",
        ToolFilter::Orgs => "Organizations",
        ToolFilter::Projects => "Projects",
        ToolFilter::PullRequests => "Pull Requests",
        ToolFilter::Repository => "Repository",
        ToolFilter::SecretProtection => "Secret Protection",
        ToolFilter::SecurityAdvisories => "Security Advisories",
        ToolFilter::Search => "Search",
        ToolFilter::Stargazers => "Stargazers",
        ToolFilter::Users => "Users",
    });

    println!("🔌 Connecting to GitHub MCP server via Docker…");
    let mut mcp = crate::mcp::McpClient::new(github_token, toolsets).await?;
    let all_tools = mcp.list_tools().await?;

    let tools: Vec<_> = match filter {
        Some(f) => all_tools.iter().filter(|t| f.matches(&t.name)).collect(),
        None => all_tools.iter().collect(),
    };

    let sep = "─".repeat(72);

    if let Some(cat) = label {
        println!("\n📋 {} tools\n{sep}", tools.len());
        println!("  {cat}");
        println!("  {}", "─".repeat(68));
        for t in &tools {
            let raw = t
                .description
                .as_deref()
                .unwrap_or("")
                .lines()
                .next()
                .unwrap_or("");
            let desc = if raw.len() > 57 {
                format!("{}…", &raw[..57])
            } else {
                raw.to_string()
            };
            println!("  {:<42} {}", t.name, desc);
        }
        println!("\n{sep}");
        return Ok(());
    }

    // All-tools grouped listing (GITHUB_TOOLSETS=all)
    println!("\n📋 {} tools\n{sep}", tools.len());
    // Group by common name patterns
    let groups: &[(&str, &[&str])] = &[
        (
            "Actions",
            &[
                "workflow", "run", "job", "artifact", "runner", "secret", "variable", "cache",
            ],
        ),
        ("Code Quality", &["code_quality", "autofix"]),
        ("Code Security", &["code_scanning", "alert"]),
        ("Context", &["get_me", "get_github_context"]),
        ("Copilot", &["copilot"]),
        ("Dependabot", &["dependabot"]),
        ("Discussions", &["discussion"]),
        ("Gists", &["gist"]),
        ("Git", &["git_"]),
        ("Issues", &["issue", "sub_issue"]),
        ("Labels", &["label"]),
        ("Notifications", &["notification", "thread"]),
        ("Organizations", &["org", "team", "member"]),
        ("Projects", &["project"]),
        (
            "Pull Requests",
            &["pull_request", "pending_review", "add_reply"],
        ),
        (
            "Repository",
            &[
                "repo",
                "branch",
                "commit",
                "release",
                "tag",
                "file",
                "fork",
                "push_files",
            ],
        ),
        ("Search", &["search_"]),
        ("Secret Protection", &["secret_scanning"]),
        ("Security Advisories", &["advisory", "cve"]),
        ("Stargazers", &["star"]),
        ("Users", &["user", "follow", "block"]),
    ];

    let mut assigned: std::collections::HashSet<&str> = Default::default();
    for (cat_label, prefixes) in groups {
        let group: Vec<_> = tools
            .iter()
            .filter(|t| {
                !assigned.contains(t.name.as_str()) && prefixes.iter().any(|p| t.name.contains(p))
            })
            .collect();
        if group.is_empty() {
            continue;
        }
        for t in &group {
            assigned.insert(t.name.as_str());
        }

        println!("  {cat_label}");
        println!("  {}", "─".repeat(68));
        for t in group {
            let raw = t
                .description
                .as_deref()
                .unwrap_or("")
                .lines()
                .next()
                .unwrap_or("");
            let desc = if raw.len() > 57 {
                format!("{}…", &raw[..57])
            } else {
                raw.to_string()
            };
            println!("  {:<42} {}", t.name, desc);
        }
        println!();
    }

    let rest: Vec<_> = tools
        .iter()
        .filter(|t| !assigned.contains(t.name.as_str()))
        .collect();
    if !rest.is_empty() {
        println!("  Other");
        println!("  {}", "─".repeat(68));
        for t in rest {
            let raw = t
                .description
                .as_deref()
                .unwrap_or("")
                .lines()
                .next()
                .unwrap_or("");
            let desc = if raw.len() > 57 {
                format!("{}…", &raw[..57])
            } else {
                raw.to_string()
            };
            println!("  {:<42} {}", t.name, desc);
        }
        println!();
    }

    println!("{sep}");
    Ok(())
}

async fn resolve_llm(
    _github_token: &str,
    api_key: Option<String>,
    base_url: Option<String>,
    model: Option<String>,
) -> Result<LlmClient> {
    if let Some(url) = base_url {
        let m = model.unwrap_or_else(|| OPENAI_DEFAULT_MODEL.into());
        println!("🔗 Using custom LLM endpoint: {url} ({m})");
        return Ok(LlmClient::new(url, api_key, m));
    }

    if let Some(key) = api_key {
        let m = model.unwrap_or_else(|| OPENAI_DEFAULT_MODEL.into());
        println!("🤖 Using OpenAI ({m})");
        return Ok(LlmClient::new(OPENAI_BASE_URL, Some(key), m));
    }

    let ollama_model = model.clone().unwrap_or_else(|| OLLAMA_DEFAULT_MODEL.into());
    let ollama = LlmClient::new(OLLAMA_BASE_URL, None, &ollama_model);
    if ollama.probe().await {
        println!("🦙 Using Ollama at {OLLAMA_BASE_URL} ({ollama_model})");
        return Ok(ollama);
    }

    // GitHub Models was retired on 2026-07-30; no longer available.
    bail!(
        "GitHub Models has been retired and is no longer available.\n  \
         Alternatives:\n  \
         • Set OPENAI_API_KEY (or --openai-key) to use OpenAI\n  \
         • Start Ollama: https://ollama.com  then: ollama pull gpt-4o-mini"
    );
}