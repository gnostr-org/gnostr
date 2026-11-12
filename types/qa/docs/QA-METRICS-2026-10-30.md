# QA Metrics — gnostr-types test development cycle

**Date**: 2026-10-30
**Command**: `cargo test -p gnostr-types --lib`
**Result**: 296 passed; 0 failed; 7 ignored — in 14.12s

## Execution Summary

| Metric | Value |
|---|---|
| New test functions added | 41 |
| … running in default suite | 34 |
| … network smoke tests (`#[ignore]`) | 7 |
| Previously zero-coverage modules addressed | 10 / 10 planned |
| Failures introduced | 0 |
| Bugs filed | 0 (no defects found while writing tests) |

## Coverage by Module

| Module | New tests | Status |
|---|---|---|
| `src/ipfs.rs` | 4 | ✅ pass |
| `src/utils/file_slice.rs` | 4 | ✅ pass |
| `src/nostr/relay_usage.rs` | 9 | ✅ pass |
| `src/nostr/relay_list.rs` | 5 | ✅ pass |
| `src/nostr/nip9.rs` | 4 | ✅ pass (incl. signature verification) |
| `src/nostr/signer.rs` | 4 | ✅ pass (EventMining validation) |
| `src/blockheight.rs` | 4 + 1 ignored | ✅ pass |
| `src/blockhash.rs` | 2 ignored | ⏭ manual/CI |
| `src/weeble.rs` | 2 ignored | ⏭ manual/CI |
| `src/wobble.rs` | 2 ignored | ⏭ manual/CI |

## Quality Gates

| Gate | Target | Result |
|---|---|---|
| Unit tests pass | 100% | ✅ 296/296 |
| New code `cargo fmt --check` | clean | ✅ (only pre-existing `protocol_slice.rs` intentional-format diffs remain) |
| New test code clippy | no new warnings | ✅ (3 pre-existing `clippy::string_slice` errors in `nip34.rs` / `versioned/event3.rs` predate this change) |
| Network tests isolated | `#[ignore]` | ✅ 7 ignored, default suite stays offline |
| Test keys | mock/public only | ✅ `PrivateKey::mock()` throughout |

## Security Notes (OWASP-relevant checks)

- NIP-09 deletion events: id hash + schnorr signature verified on every
  generated event (TC-TYPES-020).
- `EventMining::prefix` input validation: rejects empty, >64 chars, non-hex
  (TC-TYPES-022) — bounds checked before mining.
- No real private keys in test code; repo placeholder key policy respected.

## Known Pre-existing Issues (not introduced by this cycle)

1. `cargo clippy` fails on `types/src/nostr/nip34.rs:308,321` and
   `types/src/nostr/versioned/event3.rs:158` due to
   `#![deny(clippy::string_slice)]` firing on newer rustc (1.99.0). Exists on
   the unmodified tree.
2. `cargo fmt --check` reports diffs in `types/src/utils/protocol_slice.rs`;
   the odd indentation there is intentional (in-code comments say do not change).
