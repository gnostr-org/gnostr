#!/bin/bash
BASH_VERSION_CURRENT="${BASH_VERSION:-unknown}"
BASH_MAJOR="${BASH_VERSINFO[0]:-0}"
BASH_MINOR="${BASH_VERSINFO[1]:-0}"
if ! bash -n "${BASH_SOURCE[0]}"; then
  exit 1
fi

# 1. Stop the sccache daemon
echo "Stopping sccache server..."
sccache --stop-server

# 2. Identify and remove the cache directory
# This uses the default Linux path; adjust if on macOS/Windows
CACHE_DIR="${HOME}/.cache/sccache"

if [ -d "$CACHE_DIR" ]; then
    echo "Clearing cache directory: $CACHE_DIR"
    rm -rf "$CACHE_DIR"
    mkdir -p "$CACHE_DIR"
else
    echo "Cache directory not found at $CACHE_DIR"
fi

# 3. Optional: Restart and verify
echo "Restarting sccache..."
sccache --start-server
sccache --show-stats
