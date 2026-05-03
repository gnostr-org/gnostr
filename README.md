## gnostr:a git+nostr workflow utility

![](https://raw.githubusercontent.com/gnostr-org/.github/refs/heads/master/1024x1024.svg)

## README

- [legit/README.md](legit/README.md)
- [src/lib/types/README.md](src/lib/types/README.md)
- [gh/README.md](gh/README.md)
- [app/kv/README.md](app/kv/README.md)
- [asyncgit/README.md](asyncgit/README.md)
- [crawler/README.md](crawler/README.md)
- [filetreelist/README.md](filetreelist/README.md)
- [git2-hooks/README.md](git2-hooks/README.md)
- [git2-testing/README.md](git2-testing/README.md)
- [gnit/bencode/README.md](gnit/bencode/README.md)
- [gnit/buffers/README.md](gnit/buffers/README.md)
- [gnit/clone_to_owned/README.md](gnit/clone_to_owned/README.md)
- [gnit/dht/README.md](gnit/dht/README.md)
- [gnit/html/README.md](gnit/html/README.md)
- [gnit/librqbit_core/README.md](gnit/librqbit_core/README.md)
- [gnit/librqbit/README.md](gnit/librqbit/README.md)
- [gnit/network-interface/README.md](gnit/network-interface/README.md)
- [gnit/peer_binary_protocol/README.md](gnit/peer_binary_protocol/README.md)
- [gnit/README.md](gnit/README.md)
- [gnit/sha1w/README.md](gnit/sha1w/README.md)
- [gnit/statics/README.md](gnit/statics/README.md)
- [gnit/upnp/README.md](gnit/upnp/README.md)
- [invalidstring/README.md](invalidstring/README.md)
- [qr/README.md](qr/README.md)
- [query/README.md](query/README.md)
- [relay/extensions/README.md](relay/extensions/README.md)
- [relay/README.md](relay/README.md)
- [scopetime/README.md](scopetime/README.md)
- [src/bin/README.md](src/bin/README.md)
- [src/lib/legit/README.md](src/lib/legit/README.md)
- [src/lib/p2p/chat/README.md](src/lib/p2p/chat/README.md)
- [src/lib/p2p/README.md](src/lib/p2p/README.md)
- [src/lib/remote/README.md](src/lib/remote/README.md)
- [src/lib/sub_commands/README.md](src/lib/sub_commands/README.md)
- [src/lib/utils/README.md](src/lib/utils/README.md)
- [ssh/README.md](ssh/README.md)
- [types/README.md](types/README.md)
- [vendor/bitcoin/README.md](vendor/bitcoin/README.md)
- [xq/fuzz/README.md](xq/fuzz/README.md)
- [xq/README.md](xq/README.md)

--

### [CHANGELOG](CHANGELOG.md)

# Temp for CI
# Docker

Blossom server and Git tooling containers are available with:

```bash
docker compose -f docker-compose.blossom.yml up --build blossom-server
docker compose -f docker-compose.blossom.yml run --rm blossom-git blossom-cli --help
docker compose -f docker-compose.blossom.yml run --rm --service-ports blossom-git blossom-lfs daemon
```

The `blossom-git` tools image includes `git`, `git-lfs`, `blossom-cli`, and
`blossom-lfs`. The `blossom-server` image wraps upstream `blossom-server`
defaults and persists data under the `blossom_server_data` volume.

## Test wrappers

The repo keeps a few shell wrappers for the most common test paths:

```bash
./scripts/gnostr-tests.sh [--list] [--test <name>] [--ignored] [--nocapture]
./scripts/asyncgit-tests.sh [--quiet] [--release] [--locked] [--offline] [--target-dir <dir>] [--target-tmpdir] [--target-tmpdir-clean] [--ignored] [--nocapture]
./scripts/gnostr-asyncgit-tests.sh [--quiet] [--release] [--locked] [--offline] [--target-dir <dir>] [--target-tmpdir] [--target-tmpdir-clean] [--ignored] [--nocapture]
./scripts/gnostr-ngit-tests.sh [--features <list>] [--all-features] [--no-default-features] [--ignored] [--nocapture]
./scripts/cargo-install-feature-variants.sh [--features <list>] [--allow-all] [--no-default-features]
```

`gnostr-tests.sh` runs the workspace test suite, `gnostr-asyncgit-tests.sh`
adds the asyncgit DM/NIP regressions and queries the real event ids it emits,
including the shared NIP-34 repo URL vector regression and the extended
plain/mined commit, plain/mined note, and plain/PoW matrix, and
`gnostr-ngit-tests.sh` exercises the ngit library with optional cargo feature
switches.
`cargo-install-feature-variants.sh` runs `cargo install --path .` for the
default feature set, `--all-features`, and `--no-default-features`.

For the workspace runner, a handy targeted example is:

```bash
./scripts/gnostr-tests.sh --test blossom_remote_push_list_and_fetch_round_trip -- --nocapture
```

The matrix workflow `./.github/workflows/gnostr-test-matrix.yml` runs the
asyncgit and ngit wrappers on the stable toolchain, which keeps the script
paths covered in CI.

## JS web app relay control

The `js` crate now exposes relay process control through the web server:

```bash
cargo run -p gnostr-js --bin gnostr-js -- web
cargo run -p gnostr-js --bin gnostr-js -- relay
cargo run -p gnostr-js --bin gnostr-js -- relay --detach
cargo run -p gnostr-js --bin gnostr-js-relay --detach
```

The web UI uses these backend endpoints to manage the local relay:

- `GET /api/relay/status`
- `POST /api/relay/start`
- `POST /api/relay/stop`

Detached launches write their PID to `.gnostr/gnostr-js-relay.pid`, and the
startup path refuses to spawn a duplicate relay when port `8080` is already in
use.

The `web` server now starts the detached local relay automatically on boot, so
the browser UI comes up with the relay already available.
