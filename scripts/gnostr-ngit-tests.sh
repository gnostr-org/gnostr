#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

TEST_FLAGS=()
while [[ $# -gt 0 ]]; do
  case "$1" in
    --nocapture)
      TEST_FLAGS+=(--nocapture)
      ;;
    --ignored)
      TEST_FLAGS+=(--ignored)
      ;;
    *)
      echo "Unsupported flag: $1" >&2
      exit 1
      ;;
  esac
  shift
done

if [[ ${#TEST_FLAGS[@]} -gt 0 ]]; then
  cargo test -p gnostr-ngit --lib --features nostr -- "${TEST_FLAGS[@]}"
else
  cargo test -p gnostr-ngit --lib --features nostr
fi
