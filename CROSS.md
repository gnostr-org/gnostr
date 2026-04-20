# Cross builds

This repo supports a small, explicit cross-build matrix that matches the current
CI workflow and `Cross.toml`.

## Supported targets

| Name | Rust target | Builder |
| --- | --- | --- |
| `linux-x64` | `x86_64-unknown-linux-gnu` | `cargo` |
| `linux-x64-musl` | `x86_64-unknown-linux-musl` | `cross` |
| `linux-arm64` | `aarch64-unknown-linux-gnu` | `cross` |
| `linux-arm64-musl` | `aarch64-unknown-linux-musl` | `cross` |
| `windows-x64` | `x86_64-pc-windows-msvc` | `cargo` |
| `macos-x64` | `x86_64-apple-darwin` | `cargo` |
| `macos-arm64` | `aarch64-apple-darwin` | `cargo` |

Linux cross targets use Docker-backed images configured in `Cross.toml`.

## Files

- `Cross.toml`: target-specific `cross` configuration
- `docker/cross/*.Dockerfile`: custom images for Linux cross targets
- `scripts/cross.sh`: convenience wrapper that builds every target the current
  host can realistically attempt

## Prerequisites

For Linux cross targets:

- `cross`
- a running container engine: `docker` or `podman`

For native targets:

- standard Rust toolchain via `rustup`

## Usage

Build everything this host can attempt:

```bash
./scripts/cross.sh
```

List what the current host can attempt:

```bash
./scripts/cross.sh --list
```

Dry-run the generated commands:

```bash
./scripts/cross.sh --dry-run
```

Build selected targets only:

```bash
./scripts/cross.sh --target linux-arm64 --target linux-arm64-musl
./scripts/cross.sh --target x86_64-apple-darwin
```

Build a different package or the whole workspace:

```bash
./scripts/cross.sh --package gnostr-relay
./scripts/cross.sh --workspace
```

Pass cargo feature/profile flags through:

```bash
./scripts/cross.sh --profile dist --all-features
./scripts/cross.sh --no-default-features --features blossom-tui
```

## Docker-backed cross images

`Cross.toml` points these targets at custom Dockerfiles:

- `docker/cross/aarch64-unknown-linux-gnu.Dockerfile`
- `docker/cross/aarch64-unknown-linux-musl.Dockerfile`
- `docker/cross/x86_64-unknown-linux-musl.Dockerfile`

These images extend the upstream `cross-rs` base images and bake in the extra
packages this repo needs, instead of reinstalling them on every run.

## Apple Silicon caveat

On `aarch64-apple-darwin`, `cross 0.2.5` is known to fail for the Linux
container-backed targets by trying to install a non-host toolchain such as
`stable-x86_64-unknown-linux-gnu` before it reaches the Docker build.

`scripts/cross.sh` detects that broken combination and skips those targets by
default so the script still builds the targets that are actually possible on the
current machine.

If you intentionally want to try those targets anyway:

```bash
GNOSTR_CROSS_FORCE=1 ./scripts/cross.sh
```

or:

```bash
./scripts/cross.sh --force-broken-cross
```

If you want those Linux `cross` targets to work reliably on Apple Silicon, the
best next step is to upgrade `cross` once the upstream host/toolchain issue is
resolved.
