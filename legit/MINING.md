# Mining process

`gnostr-legit` mines a **Git commit hash**, then publishes signed Nostr events
describing that commit. Mining a Nostr event ID is a separate operation.
This document describes the current implementation, including its limitations.

## Preparing a commit

Stage the intended changes before starting:

```bash
git add path/to/files
gnostr-legit --repo . --prefix 00000 --threads 4 --depth 3 \
  -m "subject" -m "commit body"
```

The repository needs an existing HEAD and Git `user.name` / `user.email`.
The miner checks for unstaged changes, writes a tree from the index, and uses
the existing HEAD as the new commit's single parent. It prepares the author,
committer, timestamp, and message once for all workers.

Repeated `-m` arguments produce a subject followed by a blank line and the
remaining message arguments joined with newlines. Optional `--depth N` history
is appended before the final blank line and mining nonce. Depth follows the
first-parent chain; a merge contributes one `<parent> <merge commit>` line for
each direct parent but still counts as one depth step.

**Keep the worktree clean and stage only intended changes.** Once a match is
found, the implementation moves HEAD with `git reset --hard`.

## Iterating candidates

Each worker has its own zero-based `u32` counter and a distinct decimal worker
ID. The final commit-message line is:

```text
<worker ID, minimum two decimal digits>-<counter, eight hexadecimal digits>
```

For example:

| Worker | Counter 0 | Counter 1 | Counter 2 |
| --- | --- | --- | --- |
| 0 | `00-00000000` | `00-00000001` | `00-00000002` |
| 4 | `04-00000000` | `04-00000001` | `04-00000002` |

Workers can use the same counter without hashing the same candidate: their
worker IDs make the commit contents different.

For every counter value, the worker rebuilds the raw commit content, resets
its SHA-1 digest, and computes the normal Git object hash:

```text
SHA1("commit " + decimal_byte_length(raw_commit) + NUL + raw_commit)
```

The tree, parent, author, committer, timestamp, and prepared message stay fixed;
the nonce line changes. Changing even one digit produces an unpredictable
hash, not the next hash in numerical order. A candidate wins when its lowercase
hexadecimal hash starts with the requested `--prefix`.

Use lowercase hexadecimal prefixes. The current matcher performs a literal
prefix comparison; it does not normalize uppercase or reject impossible
prefixes.

### Reading progress logs

Debug progress is emitted every 50,000 counter values, including zero.
The logged counter is decimal, while the nonce stored in the commit is
hexadecimal:

```text
[Worker 0] Still mining; iteration value 1550000 for target 00000
[Worker 0] Target hash found! Hash: 000006c50aa905907f72024421862a197e29a459, Value: 1558008
```

Here worker 0 found the matching commit at nonce `00-0017c5f8`.
Because counting starts at zero, it examined 1,558,009 candidates through that
match. A trace-level log additionally prints every candidate and hash.

### Difficulty and available iterations

For a prefix of `k` zero hexadecimal digits, each attempt has probability
`1 / 16^k` of succeeding. Expected work is `16^k` attempts **across all workers**,
not per worker:

| Prefix | Leading zero bits | Expected total attempts |
| --- | --- | --- |
| `000` | 12 | 4,096 |
| `00000` | 20 | 1,048,576 |
| `0000000` | 28 | 268,435,456 |

These are averages, not limits or guarantees. Approximately three times the
expected work gives a 95% chance of at least one match. For example, 18 workers
at about 3,650,000 attempts each have tried about 65,700,000 candidates, giving
only a 21.7% chance of finding seven leading hexadecimal zeros by then.
Adding workers increases throughput, not the success probability per hash.

A `u32` provides 4,294,967,296 counter values per worker. Counter exhaustion is
not explicitly handled: the next increment can panic with overflow checks or
wrap and repeat candidates without them. There is no automatic timestamp
refresh or unlimited search space.

## Selecting and persisting the winner

The first worker to find a match sends its raw commit content and full hash to
the coordinator, then stops. The coordinator receives the first result and:

1. Extracts the exact nonce from the final message paragraph.
2. Writes the raw commit content to `/tmp/<commit hash>.tmp`,
   `.gnostr/blobs/<commit hash>`, and `.gnostr/reflog/<commit hash>`.
3. Stores the object with `git hash-object -t commit -w --stdin`, then moves
   HEAD to it with `git reset --hard`.
4. Publishes the associated Nostr events.

**Other Git workers are not currently cancelled when the first worker wins.**
They can keep emitting progress during persistence and publication, until each
finds its own match or the process exits. Later matches are not selected by the
coordinator for this run. This is unnecessary work, not a second commit being
published.

Publication happens after the Git commit is written. A publication failure
does not roll back that commit or events already sent.

## Publishing the commit's events

Without `--nsec`, the signer is derived from the mined Git hash left-padded to
64 hexadecimal characters. This is a publicly recoverable key, **not a secure
account**. An explicit `--nsec` selects a different signer and skips automatic
profile, follow-list, and relay-list publication.

The default publication order is:

| Order | Kind | Purpose | Required success |
| --- | --- | --- | --- |
| 1 | 0 | `gnostr@<first seven commit characters>` profile and default artwork | At least one matching relay `OK true` |
| 2 | 3 | Follow six verified empty-hash identities and the first-parent profile | At least one WebSocket write |
| 3 | 10002 | NIP-65 outbox list containing only relays accepting kind 0 | At least one matching relay `OK true` |
| 4 | 1617 | NIP-34 patch for the mined commit | At least one WebSocket write |
| 5 | 1 | Announcement referencing the patch event | At least one WebSocket write |

The relay list uses `r` tags marked `write`; it does not claim verified read
access. It is sent only to the relays that explicitly accepted the metadata.
Rejections, unrelated acknowledgements, timeouts, and closed connections do not
qualify. WebSocket writes alone do not prove relay acceptance.

Every generated event carries `["t","gnostr"]` and the exact
`["commit-nonce","<worker>-<counter>"]`. Patch IDs are cached under the actual Git
directory at `gnostr/patch-events/<commit hash>` before the announcement.
Parent patch IDs come from that cache or a validated relay lookup; a Git hash
alone cannot determine a Nostr event ID. Missing parent events retain the
derived parent public key but omit the parent event reference.

## Git proof versus Nostr event-ID proof

The Git nonce and its `commit-nonce` tag prove the Git commit's prefix search.
They do **not** claim NIP-13 proof of work on any Nostr event ID.

The working Nostr miner is
[`Signer::sign_event_with_pow`](../types/src/nostr/signer.rs). It changes the
decimal NIP-13 `["nonce","<counter>","<target bits>"]` tag, hashes the unsigned
event with SHA-256, counts leading zero **bits**, and signs the winning event.
It uses `u64` counters, multiple threads, and a shared cancellation flag.
[`generate_git_note_event_with_pow`](../types/src/nostr/nip34.rs) is an existing
NIP-34 caller. [`generate_pow_event`](../types/src/nostr/nip13.rs) is another,
single-threaded implementation that signs each candidate.

**`EventBuilder::to_pow_event` is currently a TODO:** despite accepting a
difficulty argument, it signs without mining. Neither the standalone Git
mining path nor `compose` currently requests Nostr event-ID mining.

## Source and focused checks

The candidate loop and nonce formatting live in
[`src/worker.rs`](src/worker.rs). Preparation, first-result selection,
persistence, and publication live in [`src/gitminer.rs`](src/gitminer.rs).

The existing test runner uses matching cached Rust dependencies without
invoking Cargo:

```bash
./scripts/run-legit-chain-tests.py depth
./scripts/run-legit-chain-tests.py profile
```

Depth tests cover first-parent traversal, merge-parent rows, and nonce
placement. Profile tests cover deterministic empty-hash signers, follow
deduplication, relay acknowledgements, and signed NIP-65 relay lists.
