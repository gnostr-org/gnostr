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

Use `--nsec "$NOSTR_PRIVATE_KEY"` to sign the published NIP-34 patch event with
your own key. It accepts a 64-character hex private key (such as a SHA-256 digest)
or a bech32 `nsec1...` private key. Invalid keys are rejected before mining.
Only the public key is logged, never the private key.

Without `--nsec`, the legacy signing key is derived from the mined commit hash.
That hash is public, so the derived key is **not a secure identity**; use an
explicit private key for authenticated publishing. CLI arguments may be visible
to other local processes; avoid literal private keys in shell history.

When `--nsec` is omitted, the miner first publishes a kind-0 profile metadata
event signed by the key derived from the zero-padded commit hash. The profile
uses the gnostr avatar and banner from `gnostr-types`, then publishes the NIP-34
patch with the same key to the same relays. Both event IDs are logged.
Metadata publication errors are reported and stop publication of the patch.
With an explicit `--nsec`, no profile metadata is published or overwritten.
All legit-generated events include the Nostr hashtag tag `["t","gnostr"]`,
including kind-0 profiles and NIP-34 patches, so clients can discover `#gnostr`.
After a patch is sent, a kind-1 announcement is sent to the same relays with the
same signing key. Its content is `nostr:note1...` referencing the patch event,
and its tags include `["e","<patch event ID>"]` and `["t","gnostr"]`.
The announcement event ID is logged separately. Announcement errors propagate;
the already-published patch is not rolled back.

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
