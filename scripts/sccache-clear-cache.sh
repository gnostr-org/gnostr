#!/bin/bash
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
