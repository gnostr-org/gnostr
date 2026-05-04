#!/usr/bin/env bash
set -euo pipefail

if ! bash -n "${BASH_SOURCE[0]}"; then
  exit 1
fi

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

export RUST_LOG="${RUST_LOG:+$RUST_LOG,}ureq=off,serial_test=off,mio=off,tungstenite=off,tokio_tungstenite=off"

TEST_FLAGS=(--nocapture)
TEST_NAME=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --ignored)
      TEST_FLAGS+=(--ignored)
      ;;
    --test)
      shift
      [[ $# -gt 0 ]] || { echo "--test requires a test name" >&2; exit 1; }
      TEST_NAME="$1"
      ;;
    --help|-h)
      cat <<'EOF'
Usage: gnostr-chat-tests.sh [--test NAME] [--ignored] [--help]

Runs chat tests with --nocapture enabled by default.
EOF
      exit 0
      ;;
    *)
      echo "Unsupported flag: $1" >&2
      exit 1
      ;;
  esac
  shift
done

if [[ -n "$TEST_NAME" ]]; then
  cargo test -p gnostr-chat "$TEST_NAME" -- --exact "${TEST_FLAGS[@]}"
else
  cargo test -p gnostr-chat -- "${TEST_FLAGS[@]}"
fi
