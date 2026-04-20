# Docker

This repo currently uses Docker in two distinct ways:

1. **Runtime containers** for Blossom services and tools
2. **Build containers** for `cross`-based Linux target builds

## Files

| Path | Purpose |
| --- | --- |
| `docker/Dockerfile.blossom-server` | Container image for upstream `blossom-server` |
| `docker/Dockerfile.blossom-git` | Tools image with `git`, `git-lfs`, `blossom-cli`, and `blossom-lfs` |
| `docker/blossom-server-entrypoint.sh` | Env-driven entrypoint for the Blossom server image |
| `docker-compose.blossom.yml` | Local compose stack for Blossom server and tools |
| `docker/cross/*.Dockerfile` | Custom images used by `cross` for Linux cross-target builds |
| `Cross.toml` | Wires `cross` targets to those Dockerfiles |

## Blossom containers

### `blossom-server`

`docker/Dockerfile.blossom-server` builds an image around upstream
`blossom-server` and starts it through `docker/blossom-server-entrypoint.sh`.

Default behavior:

- listens on `0.0.0.0:3000`
- persists data under `/var/lib/blossom`
- stores blobs under `/var/lib/blossom/blobs`
- stores SQLite metadata in `/var/lib/blossom/blossom.db`

Supported env vars:

| Variable | Default | Meaning |
| --- | --- | --- |
| `BLOSSOM_BIND` | `0.0.0.0:3000` | Server bind address |
| `BLOSSOM_BASE_URL` | `http://localhost:3000` | Public base URL |
| `BLOSSOM_DATA_DIR` | `/var/lib/blossom/blobs` | Blob storage path |
| `BLOSSOM_DB_PATH` | `/var/lib/blossom/blossom.db` | SQLite DB path |
| `BLOSSOM_LOG_LEVEL` | `info` | Log level |
| `BLOSSOM_EXTRA_ARGS` | empty | Extra raw CLI flags appended to `blossom-server` |

### `blossom-git`

`docker/Dockerfile.blossom-git` is a convenience image for client-side Blossom
and Git workflows. It includes:

- `git`
- `git-lfs`
- `blossom-cli`
- `blossom-lfs`

It is intended for interactive use against the local compose stack or another
reachable Blossom server.

## Compose stack

`docker-compose.blossom.yml` defines:

- `blossom-server`: the persisted Blossom API server
- `blossom-git`: an optional tools container behind the `tools` profile

### Start the server

```bash
docker compose -f docker-compose.blossom.yml up --build blossom-server
```

### Run the tools container

```bash
docker compose -f docker-compose.blossom.yml run --rm blossom-git blossom-cli --help
```

### Run the LFS daemon inside the tools container

```bash
docker compose -f docker-compose.blossom.yml run --rm --service-ports blossom-git blossom-lfs daemon
```

### Compose env vars

| Variable | Default | Meaning |
| --- | --- | --- |
| `BLOSSOM_PORT` | `3000` | Published host port for `blossom-server` |
| `BLOSSOM_BASE_URL` | `http://localhost:3000` | Public URL passed to the server |
| `BLOSSOM_LOG_LEVEL` | `info` | Server log level |
| `BLOSSOM_SERVER_EXTRA_ARGS` | empty | Extra flags passed through to the server |
| `BLOSSOM_SERVER_URL` | `http://blossom-server:3000` | Server URL inside `blossom-git` |
| `NOSTR_PRIVATE_KEY` | empty | Optional auth key for Blossom client operations |
| `BLOSSOM_DAEMON_PORT` | `31921` | Port used by `blossom-lfs daemon` |

### Persistence

The compose stack uses one named volume:

- `blossom_server_data` → `/var/lib/blossom`

That volume contains both blob data and the default SQLite database.

## Cross build images

The repo also uses Docker for `cross` builds of Linux targets.

These images live under `docker/cross/`:

- `aarch64-unknown-linux-gnu.Dockerfile`
- `aarch64-unknown-linux-musl.Dockerfile`
- `x86_64-unknown-linux-musl.Dockerfile`

They extend upstream `cross-rs` base images and bake in the extra packages this
workspace needs for bindgen, OpenSSL, musl, and related native dependencies.

`Cross.toml` points the corresponding targets at these Dockerfiles, and
`scripts/cross.sh` is the local entrypoint for those builds.

For cross-build usage details, see [`CROSS.md`](CROSS.md).

## Notes

- `docker compose` and legacy `docker-compose` both work for the current
  Blossom compose file.
- `cross` additionally requires a working container runtime such as Docker or
  Podman.
- On Apple Silicon, `cross 0.2.5` has a known host/toolchain issue for some
  Linux cross targets; that behavior is documented in `CROSS.md`.
