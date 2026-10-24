#!/usr/bin/env bash
set -euo pipefail

echo "=== Checking for cargo-hack ==="
if ! command -v cargo-hack &> /dev/null; then
    echo "cargo-hack not found. Installing..."
    cargo install cargo-hack
fi

echo "=== Running cargo hack check across workspace ==="
cargo hack check --workspace --each-feature --no-dev-deps

echo "=== Feature check complete ==="
