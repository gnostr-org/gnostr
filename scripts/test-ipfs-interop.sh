#!/usr/bin/env bash
set -uo pipefail
cd "$(dirname "$0")/.."
echo "=== p2p (ipfs) ==="
cargo test -p gnostr-p2p --features ipfs 2>&1 | tail -8
echo "=== chat (ipfs) ==="
cargo test -p gnostr-chat --features ipfs 2>&1 | tail -8
echo "=== web (ipfs) ==="
cargo test -p gnostr-web --features ipfs 2>&1 | tail -8
echo "DONE"
