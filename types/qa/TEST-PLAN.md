# Test Plan — gnostr-types

**Version**: 1.0
**Date**: 2026-10-30
**Scope**: `types/` crate (shared Nostr / git / key types)
**Baseline**: 423 existing tests across 58 source files

## 1. Objectives

Raise unit-test coverage of the `gnostr-types` crate with a focus on the
previously **zero-coverage** modules, following the Arrange–Act–Assert (AAA)
pattern and Google Testing Standards priority classification.

## 2. Scope

### In scope (zero-coverage modules targeted this cycle)

| Module | File | Category | Network |
|---|---|---|---|
| IPFS types | `src/ipfs.rs` | UNIT | No |
| File slice reader | `src/utils/file_slice.rs` | UNIT | No |
| Relay usage bitflags | `src/nostr/relay_usage.rs` | UNIT | No |
| NIP-65 relay list | `src/nostr/relay_list.rs` | UNIT | No |
| NIP-09 deletion | `src/nostr/nip9.rs` | UNIT | No |
| Event mining / signer | `src/nostr/signer.rs` | SEC | No |
| Block height | `src/blockheight.rs` | NET | Yes (ignored) |
| Block hash | `src/blockhash.rs` | NET | Yes (ignored) |
| Weeble | `src/weeble.rs` | NET | Yes (ignored) |
| Wobble | `src/wobble.rs` | NET | Yes (ignored) |

### Out of scope

- Modules already covered by the existing 423 tests (serialization,
  versioned events/tags, NIPs with existing suites)
- Live-relay integration testing (tracked separately under `relay/`)

## 3. Test Categories & Priorities

- **P0**: cryptographic correctness, event signing/verification
- **P1**: protocol data-structure round trips (relay lists, deletion events)
- **P2**: pure helper logic (bitflags, file slices, IPFS value types)
- **P3**: network smoke tests (marked `#[ignore]`, run manually or in CI matrix)

## 4. Quality Gates

| Gate | Target | Status |
|---|---|---|
| New tests pass (`cargo test -p gnostr-types`) | 100% | pending |
| `cargo clippy -p gnostr-types --all-targets` | no new warnings | pending |
| `cargo fmt -p gnostr-types --check` | clean | pending |
| Network tests | `#[ignore]`, never block default suite | enforced |
| Real private keys in test code | none (use `PrivateKey::mock` / public test keys) | enforced |

## 5. Execution

```bash
# Fast unit suite (default)
cargo test -p gnostr-types

# Including network smoke tests
cargo test -p gnostr-types -- --ignored

# Tracking
# Results are recorded in TEST-EXECUTION-TRACKING.csv after each run.
```

## 6. Bug Tracking

Failures discovered while writing/executing these tests are filed in
`BUG-TRACKING-TEMPLATE.csv` with severity P0–P4 per the skill's
classification rules (P0 = security/data loss … P4 = cosmetic).
