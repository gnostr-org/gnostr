# AGENTS.md — `gnostr-grammar`

Guide for AI agents working on the `gnostr-grammar` tree-sitter grammar repository crate.

## Overview

`gnostr-grammar` downloads the Helix editor's `languages.toml` and grammar sources at compile time, builds C/C++ parser and external-scanner objects, and generates:

- A `Language` enum for file-extension detection.
- A `Grammar` enum for highlight configuration lookup.
- Per-grammar `tree_sitter_language::LanguageFn`, highlight queries, and external-symbol references.

It is used by `gnostr-asyncgit`'s TUI syntax highlighter.

## Source map

| File | Purpose |
|------|---------|
| `src/lib.rs` | Includes generated registries and defines `HighlightConfigurationParams` |
| `build.rs` | Downloads Helix sources, compiles grammars, generates registries |

## Build behavior

1. Fetches `https://github.com/helix-editor/helix` at `GRAMMAR_REPOSITORY_REF`.
2. Parses `languages.toml` for language and grammar definitions.
3. For each grammar not in a blacklist, runs `tree-sitter generate` (if needed) and compiles `parser.c` plus optional external scanner sources with `cc`.
4. Generates `grammar.registry.rs`, `language.registry.rs`, and `grammar.defs.rs` into `$OUT_DIR`.

## Blacklists

Three blacklist lists in `build.rs` control which grammars are skipped:

- `BLACKLISTED_MODULES` — always excluded (platform-agnostic failures or unwanted languages).
- `MACOS_BLACKLISTED` — excluded on `cfg(target_os = "macos")`.
- `WINDOWS_BLACKLISTED` / `MUSL_BLACKLISTED` — platform-specific exclusions.

`is_blacklisted` normalizes hyphenated grammar names to underscores before checking, because Helix `languages.toml` uses hyphens (e.g. `haskell-persistent`) while linker symbols and the blacklist use underscores (`_tree_sitter_haskell_persistent`).

If a grammar fails to link on a platform (undefined `_tree_sitter_*` symbols), add it to the appropriate blacklist rather than disabling the whole TUI feature. On macOS in particular many C/C++ grammars do not link cleanly and are blacklisted in `build.rs`. The generated `language.registry.rs` also uses an explicit `&[&str]` cast for `RegexSet::new` so an all-blacklisted registry still compiles.

## Dynamic linking

Setting `TREE_SITTER_GRAMMAR_LIB_DIR` skips compilation and loads pre-built grammar libraries from the named directory. Useful for cross-compilation or avoiding the long Helix download.

## Common tasks for agents

### A grammar fails to compile or link on a platform

1. Find the grammar name as used in `languages.toml` (usually the snake_case symbol prefix).
2. Add it to the appropriate blacklist in `build.rs`.
3. Re-run `cargo build -p gnostr-grammar` to verify the symbol is no longer referenced.

### Updating the Helix pin

- Change `GRAMMAR_REPOSITORY_REF` in `build.rs`.
- Delete cached Helix sources so the new ref is fetched.
- Expect some grammars to move in/out of blacklists.

### Removing tree-sitter entirely

- Disable the `tui` feature in dependents (e.g., `gnostr-asyncgit`) instead of editing this crate.

## Pitfalls

- The build downloads a large Git repository and compiles many C/C++ files; first builds are slow.
- Generated files live in `$OUT_DIR`; edits must happen in `build.rs` or the Helix source/config.
- External scanner symbols must be linked; missing symbols usually mean the grammar is blacklisted incorrectly or the scanner source wasn't compiled.
- macOS requires `cargo::rustc-link-lib=c++` for C++ scanners; Linux needs `stdc++`.
