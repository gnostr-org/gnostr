# Workspace `#[ignore]` test audit

Date: 2026-10-11

## How this was run

```bash
# List every test marked #[ignore] in the workspace
cargo test --workspace -- --ignored --list

# Run all ignored tests without stopping at the first failure
# (stopped after ~11 min because several network tests hang indefinitely)
cargo test --workspace --no-fail-fast -- --ignored

# Individual packages / tests were also run to confirm isolation issues.
```

## TL;DR

Most `#[ignore]` tests are **network-dependent** (relays, WebSocket, libp2p,
screenshot/TUI). They fail or hang when run as part of a full workspace run
because they rely on external relays, peer discovery, or a GUI context.

A smaller group in `vendor/gitui/asyncgit` fails deterministically on this
machine because the `gix` status fallback does not see untracked files inside
sub-directories. Those tests are marked `#[ignore]` because of that
environment-specific behaviour.

When the non-network root-crate ignored tests are run **individually**, most of
them pass. They only fail in the full parallel run due to shared global state
(config/cache directories, env vars, tracing subscriber).

---

## 1. `vendor/gitui/asyncgit` — untracked sub-directory status failures

### Failing tests

| Test | Location |
|------|----------|
| `sync::diff::tests::test_untracked_subfolder` | `vendor/gitui/asyncgit/src/sync/diff.rs:435` |
| `sync::reset::tests::test_reset_untracked_in_subdir` | `vendor/gitui/asyncgit/src/sync/reset.rs:149` |
| `sync::reset::tests::test_reset_folder` | `vendor/gitui/asyncgit/src/sync/reset.rs:176` |
| `sync::reset::tests::test_reset_untracked_in_subdir_and_index` | `vendor/gitui/asyncgit/src/sync/reset.rs:223` |
| `sync::reset::tests::test_reset_untracked_in_subdir_with_cwd_in_subdir` | `vendor/gitui/asyncgit/src/sync/reset.rs:288` |
| `sync::reset::tests::test_reset_untracked_subdir` | `vendor/gitui/asyncgit/src/sync/reset.rs:319` |
| `sync::status::tests::test_get_status_with_workdir` | `vendor/gitui/asyncgit/src/sync/status.rs:378` |
| `sync::commit::tests::test_commit` | `vendor/gitui/asyncgit/src/sync/commit.rs:225` |
| `sync::commit::tests::test_commit_in_empty_repo` | `vendor/gitui/asyncgit/src/sync/commit.rs:248` |

### Why they fail

The test helpers create a `RepoPath` from a plain string:

```rust
let repo_path: &RepoPath =
    &root.as_os_str().to_str().unwrap().into();
```

`RepoPath::from(&str)` produces `RepoPath::Path(...)`, whose `workdir()` returns
`None`. `get_status()` therefore takes the **gix** backend path instead of the
**git2** path (`vendor/gitui/asyncgit/src/sync/status.rs:213`).

On this machine the gix dir-walk does not emit untracked files that live inside
newly created sub-directories. Example trace:

```text
gix_dir::walk::function statistics=Outcome {
    read_dir_calls: 1,
    returned_entries: 0,
    seen_entries: 2
}
```

It sees the `foo/` directory but returns zero entries, so `get_statuses()`
returns `(0, 0)` instead of the expected counts. System `git status` run inside
the same temp directory correctly reports the untracked files, confirming it is
a `gix` backend limitation/version issue, not a missing-file problem.

### Fix options

1. Change the test helpers to construct `RepoPath::Workdir { gitdir, workdir }`
   so `get_status()` uses the git2 path (consistent with gitui's normal usage).
2. Fix/debug the gix status path so it respects `status.showUntrackedFiles=all`
   and recurses into untracked directories.
3. Leave them `#[ignore]` until the gix backend catches up.

---

## 2. Root `gnostr` crate — network & integration tests

Location: `src/lib/sub_commands/{dm,query,server}.rs`,
`src/lib/test_utils/tests/`, `src/lib/utils/screenshot.rs`

| Test / group | Why ignored / what happens |
|--------------|----------------------------|
| `dm::dm_tests::test_dm_command_success_bech32_recipient` | Passes when run alone; fails in full parallel run due to shared relay pool / cache state. |
| `dm::dm_tests::test_dm_command_success` | Sends a real DM through public relays (relies on `wss://relay.damus.io`, `wss://blossom.gnostr.cloud`). |
| `dm::dm_tests::test_dm_command_decryption_success` | Same as above. |
| `query::tests::test_real_network_roundtrip_kind44_with_default_key` | Hits live relays; can hang if relays are slow. |
| `query::tests::test_real_network_roundtrip_kind4_with_default_key` | Same. |
| `query::tests::test_launch_no_panic_with_all_bootstrap_relays` | Tries to connect to every bootstrap relay; often hangs. |
| `server::tests::blossom_remote_push_list_and_fetch_round_trip` | Spins up a local Blossom server; passes when run alone but heavy. |
| `utils::screenshot::tests::test_make_screenshot_cross_platform` | macOS screen-capture; passes alone, fails in parallel because the screen/TUI state is shared. |
| `test_utils::tests::test_screenshot::test_capture_tmux` | Requires tmux. |
| `test_utils::tests::chat_simulation::test_chat_simulation` | Requires libp2p network setup. |
| `test_utils::tests::gitminer::test_mine_commit_success` | Commented-out `#[ignore]`; heavy PoW. |
| `test_utils::tests::make_just_script::*` | Generates/runs just scripts; environment-specific. |

Many of these are marked `#[ignore]` because they are **not hermetic**: they
mutate `~/.config/gnostr`, open ephemeral ports, or contact the public internet.
Running them in parallel makes the failures worse.

---

## 3. `gnostr-ngit` integration tests

Location: `ngit/tests/{ngit_init,ngit_list,ngit_login,ngit_send,git_notes}.rs`

Roughly 90+ tests are `#[ignore]`. They exercise the full `ngit` workflow:

- mock relay setup
- nostr identity / login prompts
- `git-remote-nostr` push/fetch/list
- PR creation and state events
- git-notes round-trips

These require the mock relay infrastructure in `ngit/test_utils/`. They are
slow and network-dependent, so they are skipped by default. Running them all at
once frequently hangs on relay connection timeouts.

---

## 4. `gnostr-chat` / `gnostr-p2p` integration tests

Location: `chat/src/tests/p2p_chat_integration_tests.rs`,
`p2p/src/{crawler_broadcast,relay_bridge}.rs`

These tests need a real libp2p swarm, mDNS peers, or a running relay bridge.
They are inherently flaky outside of a controlled CI environment and are marked
`#[ignore]` / `long_tests`.

---

## 5. `gnostr-asyncgit` syntax-highlight test

Location: `asyncgit/src/lib/tui/shared/syntax_parser/mod.rs:164`

`test_highlight` is ignored because the tree-sitter grammar build is
platform-sensitive and may not produce stable highlight output on every host.

---

## 6. Benchmarks

Location: `types/src/nostr/{nip44/nip44.md,types.md}`

`bench_encryption_inner` tests are true benchmarks and are ignored during normal
unit-test runs.

---

## Recommendations

1. **Do not run `cargo test --workspace -- --ignored` blindly.** Several tests
   will hang for minutes waiting for relays / peers.
2. **Run ignored tests per crate** with `--nocapture` and preferably
   `RUST_TEST_THREADS=1` for anything that touches global state.
3. **Fix the `vendor/gitui/asyncgit` tests first.** They are deterministic and
   local. The right fix is probably constructing `RepoPath::Workdir` in the test
   helpers so the git2 status path is exercised.
4. **Convert network tests to use the existing mock relay / test-utils**
   infrastructure where possible, or gate them behind a `network_tests` feature
   instead of `#[ignore]` so the intent is explicit.
