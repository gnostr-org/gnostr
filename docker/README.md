# 🐙 Gnostr MCP Agent

A Rust CLI that lets you explore any GitHub repository using natural language. It connects to the official [`ghcr.io/github/github-mcp-server`](https://github.com/github/github-mcp-server) Docker image via the [Model Context Protocol](https://modelcontextprotocol.io/) and uses an LLM to decide which GitHub API tools to call on your behalf.

> **Also includes** the original Python/Streamlit version (`github_agent.py`).

---

## How It Works

```
You (natural language query)
        │
        ▼
┌───────────────┐   JSON-RPC/stdio   ┌──────────────────────────────┐
│  Rust Agent   │ ◄────────────────► │  Docker: github-mcp-server   │
│  (agent.rs)   │                    │  (official GitHub image)     │
└──────┬────────┘                    └──────────────┬───────────────┘
       │  HTTP                                      │ GitHub REST API
       ▼                                            ▼
┌───────────────┐                          github.com/your-repo
│  LLM Backend  │
│  (openai.rs)  │
└───────────────┘
```

1. **Tool selection** — The LLM reads tool names and descriptions and picks the ≤8 most relevant tools for your query.
2. **Agentic loop** — The LLM calls tools (e.g. `list_issues`, `search_code`), receives JSON results from the GitHub API via the MCP server, and iterates until it can produce a final answer.
3. **Answer** — The LLM synthesises the API results into a readable response.

The LLM never touches GitHub directly and never sees your PAT — all API calls go through the MCP Docker container.

---

## Requirements

| Requirement | Notes |
|---|---|
| [Rust](https://rustup.rs) 1.75+ | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| [Docker](https://www.docker.com/get-started) | Must be running; image pulled automatically |
| GitHub PAT | [Create one](https://github.com/settings/tokens) with `repo` scope |
| LLM backend | One of: OpenAI key, [Ollama](https://ollama.com) locally, or GitHub Models (free via PAT) |

---

## Build

```bash
git clone https://github.com/Shubhamsaboo/awesome-llm-apps.git
cd awesome-llm-apps/mcp_ai_agents/github_mcp_agent
cargo build --release
# binary: ./target/release/github-mcp-agent
```

---

## LLM Backend

The agent tries backends in this order — first one that responds wins:

| Priority | Backend | How to activate |
|---|---|---|
| 1 | Custom endpoint | `--llm-url https://…` or `LLM_BASE_URL` |
| 2 | OpenAI | `--openai-key sk-…` or `OPENAI_API_KEY` |
| 3 | Ollama (local) | Start Ollama: `ollama pull llama3.2` |
| 4 | GitHub Models | Automatic — uses your `--github-token`; requires a [fine-grained PAT](https://github.com/settings/personal-access-tokens/new) with **Account permissions → Models → Read** |

---

## Usage

```
github-mcp-agent [OPTIONS] <COMMAND>

Options:
  --github-token <TOKEN>   GitHub PAT  (or GITHUB_TOKEN env var)
  --openai-key <KEY>       OpenAI key  (or OPENAI_API_KEY env var)
  --llm-url <URL>          LLM base URL (or LLM_BASE_URL env var)
  --model <MODEL>          Model name  (or LLM_MODEL env var)
  --list-tools             List all tools grouped by category
```

### Subcommands

Every subcommand loads **only** the tools for that category, keeping the
LLM context small and tool selection accurate.

| Command | GitHub Toolset | Description |
|---|---|---|
| `actions` | `actions` | Workflows, runs, jobs, artifacts |
| `code-quality` | `code_quality` | Code quality tools |
| `code-security` | `code_security` | Code Scanning alerts |
| `context` | `context` | Current user & GitHub context |
| `copilot` | `copilot` | Copilot tools |
| `dependabot` | `dependabot` | Dependabot alerts & PRs |
| `discussions` | `discussions` | GitHub Discussions |
| `gists` | `gists` | GitHub Gists |
| `git` | `git` | Low-level Git API |
| `issues` | `issues` | Issues & sub-issues |
| `labels` | `labels` | Labels |
| `notifications` | `notifications` | Notifications & threads |
| `orgs` | `orgs` | Organizations & teams |
| `projects` | `projects` | GitHub Projects |
| `prs` | `pull_requests` | Pull requests & reviews |
| `repo` | `repos` | Files, branches, commits, releases, tags |
| `search` | `repos,issues,pull_requests` | Code, commit, issue, PR, repo search |
| `secret-protection` | `secret_protection` | Secret Scanning |
| `security-advisories` | `security_advisories` | Security advisories |
| `stargazers` | `stargazers` | Stargazers |
| `users` | `users` | User profiles & relationships |
| `tools` | `all` | List all tools (no token or LLM needed) |

Each subcommand accepts:
```
-r, --repo <owner/repo>   Repository to query  [default: gnostr-org/gnostr]
-q, --query <text>        Natural language query (prompted if omitted)
    --list-tools          Show tools for this category and exit
```

---

## Examples

```bash
# Explore issues
github-mcp-agent --github-token TOKEN issues -r torvalds/linux -q "show open bugs"

# Recent merged PRs
github-mcp-agent --github-token TOKEN prs -r rust-lang/rust -q "merged last week"

# CI failures
github-mcp-agent --github-token TOKEN actions -r owner/repo -q "failing workflows"

# Search code
github-mcp-agent --github-token TOKEN search -r owner/repo -q "function parse_token"

# Dependabot alerts
github-mcp-agent --github-token TOKEN dependabot -r owner/repo -q "critical alerts"

# List tools for a category (no LLM needed)
github-mcp-agent issues --list-tools
github-mcp-agent actions --list-tools

# List every available tool (no token needed)
github-mcp-agent --list-tools

# Use Ollama instead of GitHub Models
github-mcp-agent --github-token TOKEN --llm-url http://localhost:11434/v1 --model llama3.2 \
  issues -r owner/repo -q "oldest unresolved issues"

# Use OpenAI
OPENAI_API_KEY=sk-... github-mcp-agent --github-token TOKEN \
  repo -r owner/repo -q "summarise recent commits"
```

---

## Environment Variables

| Variable | Equivalent flag |
|---|---|
| `GITHUB_TOKEN` | `--github-token` |
| `OPENAI_API_KEY` | `--openai-key` |
| `LLM_BASE_URL` | `--llm-url` |
| `LLM_MODEL` | `--model` |

---

## Project Structure

```
src/
  main.rs      CLI parsing, subcommand dispatch, tool listing, LLM resolution
  agent.rs     Agentic loop: tool selection, slim schemas, tool call execution
  mcp.rs       MCP JSON-RPC client over Docker stdio
  openai.rs    OpenAI-compatible LLM client (OpenAI / Ollama / GitHub Models)
  docker.rs    Cross-platform Docker detection, daemon check, image pull
```

---

## CLI Reference

### Subcommands

| Command | Description |
|---|---|
| `actions`, `issues`, `prs`, `repo`, … | Natural-language query using a toolset (requires an LLM) |
| `tools` | List all available MCP tools grouped by category |
| `call <tool> [args]` | Call a single MCP tool directly (`help` lists tools) |
| `help <tool>` | Print a tool's input schema |
| `tui` / `--tui` | Launch the Docker container TUI (oxker) |
| `docker-tui` | Docker TUI wrapper (requires `--features docker-tui`) |

### Global flags

| Flag | Description |
|---|---|
| `--github-token` | GitHub PAT (or `GITHUB_TOKEN`) |
| `--openai-key` | OpenAI API key (or `OPENAI_API_KEY`) |
| `--llm-url` | Custom LLM base URL |
| `--model` | Model name override |
| `--mcp-url` | Use an MCP server over HTTP instead of Docker stdio |
| `--use-cli` | Force Docker CLI for TUI exec |
| `--tui` | Launch the TUI |

### TUI

- Select a container, press **`t`** / **`T`** to exec into it.
- `--use-cli` uses native `docker exec -it`.
- Custom image with a shell: `GITHUB_MCP_IMAGE=github-mcp-server:shell` (build via `Dockerfile.mcp`).

### Direct tool calls

```bash
# List tools
github-mcp-agent tools

# Call a tool
github-mcp-agent call search_code '{"query":"rust","owner":"gnostr-org"}'

# Tool schema
github-mcp-agent help list_branches
github-mcp-agent call list_branches --help
```

---

## Python / Streamlit version

The original Streamlit app is still available:

```bash
pip install -r requirements.txt
streamlit run github_agent.py
```

### docker-tui derived from:

[OXKER.md](OXKER.md)
