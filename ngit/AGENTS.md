# AGENTS.md

Documentation for AI agents and automated tools working with the ngit codebase.

## Project Overview

**ngit** is a nostr plugin for git that enables decentralized code collaboration using the nostr protocol.

- **Language**: Rust
- **Type**: Command-line tool with git integration
- **Architecture**: Two main binaries (`ngit` and `git-remote-nostr`) with shared library code
- **Key Dependencies**: nostr-sdk, git2, clap, tokio

### Core Concepts

1. **Nostr Integration**: Uses nostr for repository identification, discovery, state management, and collaboration (PRs, issues)
2. **Git Server Agnostic**: Still requires git servers for data storage, but they're interchangeable
3. **Decentralized**: Multiple relays and git servers can be used for redundancy
4. **URL Format**: `nostr://<npub|nip05-address>/<identifier>`

## Networking

ngit sits on two network layers: a Nostr relay/WebSocket layer for repository
metadata and collaboration events, and a standard git transport layer for object
transfer.

### Nostr client (`src/lib/client.rs`)

`Client` wraps `nostr_sdk::Client`.

- **Builder** (`client.rs:202`): uses `ClientBuilder` with
  `RelayLimits::disable()` and `verify_subscriptions(true)`.
- **Custom transport** (`src/lib/transport.rs`): `HappyEyeballsTransport`
  implements RFC 8305 for direct WebSocket — IPv6 gets a 250 ms head start, then
  IPv4 is raced. Proxy/Tor modes fall back to the default async-wsocket
  implementation.
- **Relay sets** (`client.rs:1082`):
  - `relay_default_set` — default read relays.
  - `blaster_relays` — used when publishing repo announcements.
  - `fallback_signer_relays` — for remote signer (NostrConnect).
  - `grasp_default_set` — default grasp servers.
- **Timeouts** (`client.rs:918`): `long_timeout() = 45 s`, `short_timeout() = 7 s`
  (1 s under `NGITTEST`). Fetch uses an adaptive timeout: once ≥50% of relays
  succeed, remaining relays switch to the short timeout.
- **Connection logic** (`client.rs:934`): `get_events_of` retries connection with
  exponential backoff (2 s → ×1.5) until `long_timeout`, then fetches with
  `ReqExitPolicy::ExitOnEOSE`.
- **Session skip list** (`client.rs:122`): relays that hit connection/timeout
  errors are skipped for the rest of the session.
- **Signer/auth** (`client.rs:232`, `1187`): supports `NostrSigner` (local keys
  or NostrConnect). NIP-05 lookup uses a synchronous `reqwest` HTTP client
  (`client.rs:1253`).
- **Caching** (`client.rs:1322`): events are saved to a per-repo LMDB cache
  (`.git/nostr-cache.lmdb`) and a global LMDB cache in the OS config dir.

### `git-remote-nostr` protocol (`src/lib/git_remote_nostr.rs`)

`src/bin/git_remote_nostr/main.rs:10` calls
`gnostr_ngit::git_remote_nostr::run()`.

- **Argument processing** (`git_remote_nostr.rs:221`): reads `GIT_DIR`, parses
  the remote URL with `NostrUrlDecoded::parse_and_resolve`, and opens the repo.
- **Initial nostr fetch** (`git_remote_nostr.rs:130`, `264`): fetches repo
  announcement, state, patches, PRs, and contributor profiles from relays, then
  resolves the canonical `RepoRef`.
- **Command loop** (`git_remote_nostr.rs:146`) speaks git's remote-helper
  protocol:
  - `capabilities` → advertises `option`, `push`, `fetch`, `push-options`.
  - `option` → handles `verbosity` and `push-option`.
  - `list` / `list for-push` → delegates to `list::run_list`.
  - `fetch <oid> <ref>` → delegates to `fetch::run_fetch`.
  - `push <refspec>` → delegates to `push::run_push`.
- **URL mapping**: the `nostr://` URL resolves to a `Nip19Coordinate`
  (`kind:GitRepoAnnouncement`). Actual git objects are fetched from/pushed to the
  `clone` URLs embedded in the repo announcement event.

### `nostr://` URL parsing (`src/lib/git/nostr_url.rs`)

`NostrUrlDecoded::parse_and_resolve` (`nostr_url.rs:105`):

- Accepts `nostr://naddr1...`, `nostr://<npub|nip05>/<identifier>`, or
  `nostr://<npub|nip05>/<relay-hints...>/<identifier>`.
- Optional query params: `relay=`, `protocol=ssh|https|http|git`,
  `ssh_key_file=`.
- Optional path prefix protocol: `nostr://ssh/npub123/...`.
- NIP-05 identifiers are resolved over HTTP via `nip05_query` and cached in git
  config (`nostr_url.rs:196`, `client.rs:1253`).
- Result: `Nip19Coordinate { kind: GitRepoAnnouncement, public_key, identifier,
  relays }` plus protocol/ssh-key overrides.

### Grasp servers (`src/lib/repo_ref.rs`)

Grasp servers are nostr-aware git hosting endpoints that expose both a WebSocket
relay and an HTTPS git smart-HTTP endpoint under the same origin.

- **Clone URL format**: `https://<host>/<npub>/<repo-name>.git`
  (`repo_ref.rs:750`).
- **Validation** (`repo_ref.rs:750`): must be `http(s)://`, end in `.git`,
  contain a valid npub in the path, and have a non-empty repo name after
  `/<npub>/`.
- **Normalization** (`repo_ref.rs:688`): `normalize_grasp_server_url` strips the
  scheme down to `host:port/path`, removes everything from `npub1` onward, and
  downgrades `wss`/`ws`/`https` to host-only while preserving `http://` for
  `ws`/`http` inputs.
- **Relay conversion** (`repo_ref.rs:794`):
  `format_grasp_server_url_as_relay_url` turns `https://host` → `wss://host` and
  `http://host` → `ws://host`.
- **Clone URL generation** (`repo_ref.rs:802`): builds
  `https://<host>/<npub>/<identifier>.git`.
- **Detection** (`repo_ref.rs:630`): `detect_existing_grasp_servers` finds grasp
  URLs whose normalized root also appears in `repo_ref.relays`.
- **Infrastructure application** (`repo_ref.rs:837`):
  `apply_grasp_infrastructure` prepends grasp-derived relay URLs and adds
  grasp-derived clone URLs.

### Git server network behavior (`src/lib/list.rs`, `fetch.rs`, `push.rs`, `utils.rs`)

- Listing/fetching/pushing operate over standard git transports via `git2`
  (libgit2): HTTPS, HTTP, SSH, `git://`, FTP, and local filesystem
  (`utils.rs:280`).
- Protocol preferences are stored in git config (`nostr.protocol-fetch`,
  `nostr.protocol-push`) and used to order the attempt list (`utils.rs:397`).
- Read protocols (`utils.rs:280`): for plain servers defaults to unauthenticated
  HTTPS → SSH → authenticated HTTPS; for `git://` servers tries `git://` first;
  for grasp servers only unauthenticated HTTPS/HTTP.
- Write protocols (`utils.rs:333`): defaults to SSH → HTTPS; grasp servers use
  only unauthenticated HTTPS/HTTP.
- Authentication (`list.rs:545`, `fetch.rs:147`, `push.rs:165`): uses git
  credential helper for `USER_PASS_PLAINTEXT`, explicit SSH key file, or SSH
  agent for `SSH_KEY`.
- Nostr state ↔ git-server reconciliation: `list.rs/run_list` compares per-relay
  nostr state events with refs actually present on git servers and only
  advertises OIDs that are resolvable on at least one server or locally
  (`list.rs:55`).

### Push flow specifics (`src/bin/git_remote_nostr/push.rs`, `src/lib/push.rs`)

- `run_push` builds an updated nostr state map from refspecs, signs/publishes a
  state event (`kind:30618`) plus PR/PR-update/patch/merge-status events
  (`push.rs:56`).
- Events are broadcast to repo relays plus the user's write relays
  (`client.rs:2552`).
- If state-event publishing succeeds, actual git refs are pushed to each
  configured git server; if all pushes fail, the new state event is rolled back
  from the local cache (`push.rs:252`).
- For proposal branches (`refs/heads/pr/*`),
  `select_servers_push_refs_and_generate_pr_or_pr_update_event` pushes commit
  data to the user's fork or to repo grasp servers and then signs a PR event
  (`push.rs:421`).

## Project Structure

```
ngit/
├── src/
│   ├── bin/
│   │   ├── git_remote_nostr/    # Git remote helper implementation
│   │   │   ├── capabilities.rs
│   │   │   ├── fetch.rs
│   │   │   ├── list.rs
│   │   │   ├── main.rs
│   │   │   └── push.rs
│   │   └── ngit/                # Main CLI tool
│   │       ├── main.rs
│   │       └── sub_commands/
│   └── lib/                     # Shared library code
│       ├── git/                 # Git operations
│       ├── login/               # User authentication
│       ├── client.rs            # Nostr client
│       ├── fetch.rs             # Fetch operations
│       ├── git_events.rs        # Git event handling
│       ├── list.rs              # List operations
│       ├── push.rs              # Push operations
│       ├── repo_ref.rs          # Repository references
│       ├── repo_state.rs        # Repository state
│       └── utils.rs             # Utilities
├── tests/                       # Integration tests
├── test_utils/                  # Test utilities and helpers
└── git_hooks/                   # Git hooks for development
```

## Key Files and Their Purposes

### Core Library (`src/lib/`)

- **`repo_ref.rs`**: Repository reference handling, URL parsing, grasp server detection
  - Functions: `is_grasp_server_in_list()`, `is_grasp_server_clone_url()`, `normalize_grasp_server_url()`
  - Critical for identifying and validating repository URLs
  
- **`repo_state.rs`**: Manages repository state (refs, commits, etc.)

- **`client.rs`**: Nostr client wrapper and relay communication

- **`git_events.rs`**: Handles git-related nostr events

- **`push.rs`**: Push operations to git servers and nostr relays

- **`fetch.rs`**: Fetch operations from git servers

### Binaries

- **`git-remote-nostr`**: Git remote helper that enables git to work with nostr:// URLs
- **`ngit`**: Main CLI tool for managing nostr repositories

## Development Guidelines

### Code Style

- Follow Rust standard formatting (`rustfmt.toml` is configured)
- Run `cargo fmt` before committing
- Run `cargo clippy` to catch common issues
- Use `anyhow::Result` for error handling
- Prefer explicit error messages with context

### Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Run integration tests only
cargo test --test ngit_init
```

### Important Testing Notes

1. **URL Comparison**: When comparing git server URLs, be aware of normalization:
   - `is_grasp_server_in_list()`: Compares full URLs (including repo path)
   - `normalize_grasp_server_url()`: Extracts server part only (strips npub and path)

2. **Test Structure**: Integration tests are in `tests/`, unit tests are in module files

### Building

```bash
# Development build
cargo build

# Release build
cargo build --release

# Binaries will be in target/debug/ or target/release/
# - ngit
# - git-remote-nostr
```

## Common Tasks for Agents

### Adding New Features

1. **Identify the correct module**: 
   - Git operations → `src/lib/git/`
   - Nostr operations → `src/lib/client.rs`, `src/lib/git_events.rs`
   - CLI commands → `src/bin/ngit/sub_commands/`
   - Remote helper → `src/bin/git_remote_nostr/`

2. **Add tests**: Always add tests for new functionality

3. **Update error handling**: Use `anyhow::Context` for error messages

4. **Check dependencies**: Avoid adding unnecessary dependencies

### Debugging Issues

1. **Check test failures**: Run `cargo test` to identify failing tests

2. **Review error context**: Look at the full error chain with `.context()`

3. **Examine URL handling**: Many issues relate to URL parsing/normalization
   - Check `repo_ref.rs` for URL-related functions
   - Verify grasp server detection logic

4. **Trace nostr events**: Check event creation and publishing in `git_events.rs`

### Refactoring

1. **Maintain backward compatibility**: This is a CLI tool users depend on

2. **Update all call sites**: Use `cargo check` to find all references

3. **Run full test suite**: Ensure nothing breaks

4. **Check integration tests**: These test real-world scenarios

## Architecture Patterns

### Error Handling

```rust
use anyhow::{Context, Result};

fn example() -> Result<()> {
    something()
        .context("descriptive error message")?;
    Ok(())
}
```

### Async Operations

- Uses `tokio` runtime for async operations
- Nostr operations are async
- Git operations are mostly sync

### Configuration

- User config stored in platform-specific directories (via `directories` crate)
- Repository config in `.git/config` and `ngit.yaml`

## Important Invariants

### URL Handling

1. **Grasp Server URLs**: Must contain npub in path format: `/{npub}/`
2. **Nostr URLs**: Format: `nostr://<npub|nip05>/<identifier>`
3. **Normalization**: Different functions normalize differently:
   - Full URL comparison: Use direct string comparison with `trim_end_matches('/')`
   - Server identification: Use `normalize_grasp_server_url()`

### Git Integration

1. **Remote helper protocol**: Must follow git's remote helper protocol exactly
2. **Ref naming**: PRs use `pr/` prefix
3. **State sync**: Must keep git refs in sync with nostr events

## Dependencies to Be Aware Of

### Critical Dependencies

- **nostr**: Core nostr protocol implementation
- **git2**: Git operations via libgit2
- **tokio**: Async runtime
- **anyhow**: Error handling

### Optional Features

Check `Cargo.toml` for feature flags and optional dependencies.

## Testing Strategy

### Unit Tests

- Located in same file as code (`#[cfg(test)] mod tests`)
- Test individual functions in isolation
- Example: `is_grasp_server_in_list` tests in `repo_ref.rs`

### Integration Tests

- Located in `tests/` directory
- Test full workflows (init, push, fetch, etc.)
- Require test utilities from `test_utils/`

### Test Utilities

- Mock relay implementation
- Git repository setup helpers
- Located in `test_utils/`

## Common Pitfalls

### 1. URL Normalization

**Problem**: Using wrong normalization function leads to incorrect comparisons.

**Solution**: 
- For full URL comparison: Direct string comparison
- For server identification: Use `normalize_grasp_server_url()`

### 2. Async/Sync Boundaries

**Problem**: Mixing async and sync code incorrectly.

**Solution**: Use `tokio::runtime::Runtime` or proper async context.

### 3. Git State

**Problem**: Git state getting out of sync with nostr events.

**Solution**: Always update both git refs and publish nostr events atomically.

## Contribution Workflow

1. **Clone**: Use ngit itself to clone: `nostr://dan@gitworkshop.dev/relay.damus.io/ngit`
2. **Branch**: Create a branch with `pr/` prefix for pull requests
3. **Test**: Run `cargo test` and `cargo clippy`
4. **Commit**: Follow git commit message conventions (enforced by `git_hooks/commit-msg`)
5. **Push**: Push branch to submit PR via ngit

## Resources

- **Main Repository**: [gitworkshop.dev/dan@gitworkshop.dev/ngit](https://gitworkshop.dev/dan@gitworkshop.dev/ngit)
- **Homepage**: [gitworkshop.dev/ngit](https://gitworkshop.dev/ngit)
- **Source**: [codeberg.org/DanConwayDev/ngit-cli](https://codeberg.org/DanConwayDev/ngit-cli)
- **Nostr Protocol**: [github.com/nostr-protocol/nostr](https://github.com/nostr-protocol/nostr)

## Quick Reference

### Build Commands

```bash
cargo build                    # Development build
cargo build --release         # Release build
cargo test                    # Run tests
cargo clippy                  # Run linter
cargo fmt                     # Format code
```

### File Locations

```bash
src/lib/repo_ref.rs           # URL handling, grasp server detection
src/lib/client.rs             # Nostr client
src/lib/git_events.rs         # Git event handling
src/bin/ngit/main.rs          # Main CLI entry point
src/bin/git_remote_nostr/     # Git remote helper
tests/                        # Integration tests
```

### Key Functions

```rust
// URL handling
is_grasp_server_in_list(url, list)     // Check if URL in list (exact match)
is_grasp_server_clone_url(url)         // Check if URL is grasp server format
normalize_grasp_server_url(url)        // Extract server part from URL

// Repository operations
repo_ref.to_nostr_git_url()            // Convert to nostr URL
push_to_remote()                       // Push to git server
fetch_from_remote()                    // Fetch from git server
```

## Support and Questions

For questions about the codebase or contributions:
- Check existing issues on [gitworkshop.dev](https://gitworkshop.dev/dan@gitworkshop.dev/ngit)
- Review test files for usage examples
- Examine integration tests for workflow examples

---

**Last Updated**: 2025-10-20  
**Version**: 1.7.4  
**Maintainer**: DanConwayDev
