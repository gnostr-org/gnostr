# Plan: Remove `./vendor/gitui` Dependencies

## Goal

Eliminate all dependencies on the vendored `gitui` fork (`vendor/gitui`,
`vendor/gitui/asyncgit`, `vendor/gitui/scopetime`) so the directory can be
deleted, while preserving the Git TUI / hooks / profiling functionality that
currently relies on it.

## Current dependency graph

| Consumer | Dependency | Usage |
|----------|------------|-------|
| root `Cargo.toml` `[patch.crates-io]` | `gitui = { path = "vendor/gitui" }` | crate-level patch (line 981) |
| root `Cargo.toml` `[dependencies]` | `gitui = "0.28.1"` | appears unused in Rust source; verify/remove (line 651) |
| `asyncgit/Cargo.toml` | `gitui_asyncgit = { package = "asyncgit", path = "../vendor/gitui/asyncgit" }` | re-exported as `upstream_sync`; used only in `asyncgit/src/lib/sync/hooks.rs` |
| `scopetime/Cargo.toml` | `gitui_scopetime = { package = "scopetime", path = "../vendor/gitui/scopetime" }` | backing implementation of `gnostr-scopetime` profiling macros |

## Impact analysis

1. **Hooks (`asyncgit/src/lib/sync/hooks.rs`)**: The only consumer of
   `gitui_asyncgit`. Uses `hooks_pre_push`, `hooks_pre_commit`,
   `hooks_post_commit`, `hooks_commit_msg`, plus `RepoPath`, `PrePushTarget`,
   `HookResult`, and `cred::BasicAuthCredential`. Reimplementing these in terms
   of `git2` + `std::process` (running `.git/hooks/*`) is feasible and keeps
   the public `gnostr-asyncgit` hooks API intact.

2. **Profiling (`scopetime`)**: `scope_time!` is invoked pervasively across
   `asyncgit`. Options:
   - Replace `gitui_scopetime` with a tiny in-tree macro (no-op when disabled,
     `std::time::Instant`-based timing when `enabled`), preserving the
     `scopetime::scope_time!` call sites.
   - Or depend on the published `scopetime` crate from crates.io if available.
   - Or gate all `scope_time!` calls behind `#[cfg(feature = "timing")]` and
     drop the dependency entirely.

3. **Root `gitui` crate dep**: Confirm unused; if any feature references it,
   remove the feature wiring.

## Step-by-step plan

### Phase 1 — Audit & isolate
- [ ] Run `cargo metadata` / `cargo tree -p gnostr-asyncgit` to confirm the only
      path from root to `vendor/gitui` is via the three entries above.
- [ ] Grep the whole workspace (excluding `vendor/`) for `gitui` crate usage to
      ensure no other crate pulls it transitively through our patch.
- [ ] Decide hooks strategy: reimplement in `gnostr-asyncgit` (preferred) vs.
      drop hooks support. Document the decision.

### Phase 2 — Remove `gitui_asyncgit` (hooks)
- [ ] Implement hook execution directly in `asyncgit/src/lib/sync/hooks.rs` using
      `git2` to resolve hook paths and `std::process::Command` to run them,
      matching gitui semantics (pre-push, pre-commit, commit-msg, post-commit).
- [ ] Remove the `pub(crate) use gitui_asyncgit::sync as upstream_sync;` line in
      `asyncgit/src/lib/mod.rs`.
- [ ] Remove `gitui_asyncgit` from `asyncgit/Cargo.toml`.
- [ ] Update `asyncgit/src/lib/sync/hooks.rs` to use the new local helpers.
- [ ] Add/adjust unit tests for hooks (existing hooks tests in asyncgit).

### Phase 3 — Replace `scopetime` backing crate
- [ ] Implement `gnostr-scopetime` without `gitui_scopetime`: provide
      `scope_time!` macro that is a no-op unless `enabled` feature is on, and
      when enabled logs elapsed time via `log`.
- [ ] Remove `gitui_scopetime` from `scopetime/Cargo.toml`.
- [ ] Keep the `enabled` feature flag and `scopetime/enabled` wiring in root
      `Cargo.toml`.
- [ ] Verify `cargo build -p gnostr-scopetime --features enabled`.

### Phase 4 — Remove patch & root dependency
- [ ] Remove `gitui = { path = "vendor/gitui" }` from `[patch.crates-io]`.
- [ ] Remove `gitui = "0.28.1"` from `[dependencies]` if unused (confirm via
      `cargo build`).
- [ ] Remove the `##gitui` comment block if desired.

### Phase 5 — Delete vendor directory & validate
- [ ] Delete `vendor/gitui/`.
- [ ] Run `cargo build --workspace` (or at least `gnostr`, `gnostr-asyncgit`,
      `gnostr-scopetime`, `gnostr-bins`).
- [ ] Run `cargo test -p gnostr-asyncgit` and hook-related tests.
- [ ] Run `cargo clippy --workspace` and `cargo fmt --all --check`.
- [ ] Update `AGENTS.md` / docs referencing vendor/gitui.

## Risks & mitigations
- Hook semantics (exit codes, stdin/stdout, env) must match git's hook contract;
  reimplement carefully and keep tests.
- Profiling macro must remain cheap when disabled to avoid perf regressions.
- Removing the patch may affect other crates if `gitui` was pulled transitively;
  verify with `cargo tree`.

## Success criteria
- `cargo build --workspace` succeeds with `vendor/gitui` deleted.
- No references to `vendor/gitui` remain in `Cargo.toml` files.
- Existing hook and asyncgit tests pass.
