#!/usr/bin/env bash
set -euo pipefail

if ! bash -n "${BASH_SOURCE[0]}"; then
  exit 1
fi

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

if ! command -v gnostr >/dev/null 2>&1; then
  echo "gnostr is required on PATH" >&2
  exit 1
fi

test_config_home="$(mktemp -d "${TMPDIR:-/tmp}/gnostr-p2p-test.XXXXXX")"
cleanup() {
  rm -rf "$test_config_home"
}
trap cleanup EXIT

export XDG_CONFIG_HOME="$test_config_home"
export RUST_LOG="${RUST_LOG:+$RUST_LOG,}ureq=off,serial_test=off,mio=off,tungstenite=off,tokio_tungstenite=off"

printf 'CRAWLER_BUCKETS\nQUIT\n' | \
  cargo run -p gnostr-p2p --bin gnostr-p2p -- \
    --secret-key-seed "$(gnostr --hash "")"
