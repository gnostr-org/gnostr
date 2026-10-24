# [Legit](https://github.com/RandyMcMillan/legit.git) [![legit](https://github.com/RandyMcMillan/legit/actions/workflows/automate.yml/badge.svg)](https://github.com/RandyMcMillan/legit/actions/workflows/automate.yml)

#### Legit adds Proof of Work (PoW) to a git commit hash prefix.

#### install rustup:

```
curl -sSf https://static.rust-lang.org/rustup.sh | sh
```

#### cargo:

```
cargo install legit
```

#### `Example`

```
git log | grep "0000006"
```

`commit` [000000615b90566ae8559dd45852190edea79a8c](httpshttps://github.com/RandyMcMillan/legit/commit/000000615b90566ae8559dd45852190edea79a8c)

---

## Usage

The standalone `gnostr-legit` binary uses workspace Clap for argument parsing:

```bash
gnostr-legit --repo . --prefix 000 --threads 2 -m "subject" -m "body"
gnostr-legit --help
gnostr-legit --version
```

Without flags, it uses the current repository, prefix `000`, available CPU
parallelism, and `default commit message`. Thread counts must be positive.

Use `--depth N` to append up to `N` existing first-parent commits to the
commit body, starting with the current HEAD and continuing newest-first through
its first parents. Merged branches are not traversed. Ordinary commits use one
full hash per line; merge commits list every direct parent as
`<full parent hash> <full merge commit hash>`, one line per parent in Git's parent
order. A merge counts as one depth step regardless of its number of parents.
These lines form the final paragraph before the blank line and mining nonce:

```text
subject

body

<full HEAD commit hash>
<full HEAD first-parent commit hash>

01-00000122
```

For example, when HEAD is a merge commit, `--depth 1` includes:

```text
<full first parent hash> <full HEAD merge commit hash>
<full second parent hash> <full HEAD merge commit hash>
```

For example, `gnostr-legit --depth 3 -m "subject" -m "body"`. The default is
`--depth 0`, which leaves the message unchanged. Negative and non-integer depths
are rejected; requests beyond the available history stop at the root. This flag
applies to mining, not `compose`. Library callers use `Gitminer::with_depth(N)`.

Use `--nsec "$NOSTR_PRIVATE_KEY"` to sign the published NIP-34 patch event with
your own key. It accepts a 64-character hex private key (such as a SHA-256 digest)
or a bech32 `nsec1...` private key. Invalid keys are rejected before mining.
Only the public key is logged, never the private key.

Without `--nsec`, the legacy signing key is derived from the mined commit hash.
That hash is public, so the derived key is **not a secure identity**; use an
explicit private key for authenticated publishing. CLI arguments may be visible
to other local processes; avoid literal private keys in shell history.

When `--nsec` is omitted, the miner first publishes a kind-0 profile metadata
event signed by the key derived from the zero-padded commit hash. Its `name`
and `display_name` are both `gnostr@<commit short hash>`, using the first seven
hexadecimal characters of the mined commit hash (for example, `gnostr@000dba5`). The profile
uses the gnostr avatar and banner from `gnostr-types`, then publishes the NIP-34
patch with the same key to the same relays. Both event IDs are logged.
Metadata publication errors are reported and stop publication of the patch.
With an explicit `--nsec`, no profile metadata is published or overwritten.
`gnostr_types::nostr::Metadata::default()` and `Metadata::new()` supply these
avatar/banner URLs. Deserializing existing profiles preserves missing or null
artwork fields rather than inserting the defaults.
All legit-generated events include the Nostr hashtag tag `["t","gnostr"]`,
including kind-0 profiles and NIP-34 patches, so clients can discover `#gnostr`.
Mined profiles, patches, and kind-1 announcements also carry
`["commit-nonce","<worker>-<counter>"]`, copied exactly from the commit's final
nonce line (for example `01-00000122`). This is Git mining proof, not the
decimal `nonce` tag used for Nostr NIP-13 event proof-of-work.
After a patch is sent, a kind-1 announcement is sent to the same relays with the
same signing key. Its content is `nostr:note1...` referencing the patch event,
and its tags include `["e","<patch event ID>","","mention"]` and `["t","gnostr"]`.
The announcement event ID is logged separately. Announcement errors propagate;
the already-published patch is not rolled back.

The patch and announcement also include a `p` tag for the parent commit's public
key, derived from its zero-padded Git hash. They reference the parent's actual
patch event with `["e","<parent patch event ID>","","reply"]` when known.
Event IDs are cached under the repository's Git directory in
`gnostr/patch-events/<commit hash>` immediately after successful patch sending.
If the parent is not cached, a bounded relay lookup checks its signature,
commit tag, kind, and derived author, choosing the oldest matching event
(event ID breaks timestamp ties). Missing parents or failed lookups are logged;
publication continues as a chain root with the parent `p` but no parent `e`.
Corrupted caches and cache I/O failures are reported as errors.
Parent public keys are deterministic; Nostr event IDs are not derivable solely
from Git hashes. An explicit `--nsec` can sign with a different public key, so
relay discovery by the derived identity cannot find that parent's patch without
the local cache.

## Compose NIP-34 events without mining

`gnostr-legit compose` prints one signed event as JSON. It does not mine, change
Git, publish metadata or announcements, or contact relays unless `--publish` is
specified. Publication also requires an explicit `--relay`; repeat it for
multiple destinations. Relay logs confirm WebSocket writes, not acceptance.

Use the global `--nsec "$NOSTR_PRIVATE_KEY"` to choose the signer. Without it,
composition uses the key derived from the current Git HEAD, without creating a
commit. This publicly derivable identity is not secure. With an explicit key,
composition works outside a Git repository.

| Subcommand | Kind | Required event inputs |
|---|---|---|
| `repo-announcement` | 30617 | `--identifier`, `--root-commit` |
| `repo-state` | 30618 | `--identifier`; optional repeated `--ref NAME=VALUE` |
| `patch` | 1617 | `--repository`, `--commit`, format-patch content |
| `pull-request` | 1618 | `--repository`, `--subject`, `--commit`, `--clone-url` |
| `pull-request-update` | 1619 | `--repository`, `--event`, `--author`, `--commit`, `--clone-url` |
| `issue` | 1621 | `--repository`, content |
| `reply` | 1111 | `--repository`, `--event`, `--author`, `--root-kind`, content |
| `reply --legacy` | 1622 | Same inputs; deprecated Git reply format |
| `status open/applied/closed/draft` | 1630-1633 | `--repository`, `--event`, `--author` |
| `grasp-list` | 10317 | Optional repeated `--server` |

`--repository` accepts `30617:<owner public key>:<identifier>` or `naddr1...`.
Event references accept hex or `note1...`; public keys accept hex or `npub1...`.
Commit hashes must be full 40-character Git hashes. Use `--content` for literal
text or `--content-file PATH` for UTF-8 files (`-` reads stdin).
Every event includes `["t","gnostr"]`; repeat `--hashtag` for additional labels.
For manually composed events, `--commit-nonce 01-00000122` attaches the same
validated Git mining nonce tag without treating it as NIP-13 proof-of-work.
Use each subcommand's `--help` for optional fields.

```bash
# Compose only: stdout contains signed JSON and its full event ID.
gnostr-legit --nsec "$NOSTR_PRIVATE_KEY" compose issue \
  --repository "30617:$OWNER_PUBKEY:gnostr" \
  --subject "Issue title" --content "Issue description"

# Publish only to explicitly chosen relays.
gnostr-legit --nsec "$NOSTR_PRIVATE_KEY" compose --publish \
  --relay wss://relay.example.com issue \
  --repository "30617:$OWNER_PUBKEY:gnostr" --content "Issue description"

# Compose a patch for HEAD without mining or publishing.
git format-patch -1 --stdout HEAD | gnostr-legit compose patch \
  --repository "30617:$OWNER_PUBKEY:gnostr" \
  --commit "$(git rev-parse HEAD)" \
  --parent-commit "$(git rev-parse HEAD^)" --content-file -
```

Patch composition accepts `--previous-patch` for the actual parent/revision
event ID and derives a parent `p` tag from `--parent-commit`. It performs no
automatic relay lookup. Replies use current NIP-22 root/parent tags by default;
`--reply-event`, `--reply-author`, and `--reply-kind` together target a nested
reply. Applied statuses support `--merge-commit`, repeated `--applied-commit`,
and repeated `--applied-patch`; these fields are rejected for other statuses.
Repository announcement `--monitor-relay` values describe monitored relays,
independently of `--relay` publication destinations.

The focused runner uses existing cached Rust dependencies without invoking
Cargo, and prints verbose progress banners:

```bash
./scripts/run-legit-chain-tests.py
```

## Debug logging

Enable debug diagnostics for repository discovery, CPU/thread selection, mining,
storage, and NIP-34 publication with:

```bash
RUST_LOG=gnostr_legit=debug gnostr-legit --repo . --prefix 000 -m "subject"
```

Debug output goes to stderr; the successful commit summary stays on stdout.
Each selected NIP-34 relay is logged with its full URL, position, and commit hash.
Use `RUST_LOG=gnostr_legit=trace` for per-candidate sizes and full hashes; trace
logging is extremely verbose and slows mining. Library callers configure their
own logger.

To include signed NIP-34 event JSON (full event ID, public key, tags, content,
timestamp, and signature) before relay connections, enable the publishing helper:

```bash
RUST_LOG=gnostr_legit=debug,gnostr_types::nostr::patch=debug,gnostr_types::nostr::client=info gnostr-legit --repo . -m "subject"
```

Successful publication logs the full event ID alongside the commit hash.
Client info logs list each successful relay URL and event ID, followed by a
sent/total summary. These confirm WebSocket writes, not relay `OK` acceptance.
Publication errors propagate instead of reporting success. Event JSON contains
the patch content, but never the signing private key.

To create a commit with a subject and a multi-line body, use the `-m` flag multiple times. The first instance of `-m` will be the commit subject, and each subsequent instance will be a new line in the commit body.

### Command Syntax

```bash

gnostr legit -m "<subject>" -m "<body_line_1>" -m "<body_line_2>" ...

```

### Example

This is the command used to successfully create a commit:

```bash

cargo run --bin gnostr -- legit \

  -m "fix(legit): improve error handling and argument parsing" \

  -m "Replaced panics with graceful error handling in gnostr-legit." \

  -m "Improved directory creation logic to prevent errors." \

  -m "Enabled multi-line commit messages with multiple -m flags."

```

This resulted in the following commit message:

```

fix(legit): improve error handling and argument parsing



Replaced panics with graceful error handling in gnostr-legit.

Improved directory creation logic to prevent errors.

Enabled multi-line commit messages with multiple -m flags.

```

### Important Note

The command-line parser currently has a limitation: **body lines cannot begin with a hyphen (`-`)**. If a line starts with a hyphen, the parser will mistakenly interpret it as a command-line flag and the command will fail.
