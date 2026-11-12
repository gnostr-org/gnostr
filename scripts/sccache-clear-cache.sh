#!/bin/bash
set -euo pipefail
BASH_VERSION_CURRENT="${BASH_VERSION:-unknown}"
BASH_MAJOR="${BASH_VERSINFO[0]:-0}"
BASH_MINOR="${BASH_VERSINFO[1]:-0}"
if ! bash -n "${BASH_SOURCE[0]}"; then
  exit 1
fi

INTERACTIVE=false
CACHE_DIR="${SCCACHE_DIR:-${HOME}/.cache/sccache}"
while [[ $# -gt 0 ]]; do
    case "$1" in
        --interactive|-i) INTERACTIVE=true ;;
        --cache-dir)
            [[ $# -ge 2 ]] || { echo "--cache-dir requires a value" >&2; exit 1; }
            CACHE_DIR="$2"
            shift
            ;;
        --help|-h)
            echo "Usage: sccache-clear-cache.sh [--interactive] [--cache-dir PATH]"
            echo "Interactive mode offers stats, stop, restart, or confirmed cache clearing."
            exit 0
            ;;
        *) echo "Unknown option: $1" >&2; exit 1 ;;
    esac
    shift
done
if [[ "$INTERACTIVE" == true ]]; then
    # shellcheck source=scripts/lib/script-ui.sh
    source "$(dirname "${BASH_SOURCE[0]}")/lib/script-ui.sh"
    ui_choose "sccache operation" stats clear stop restart
    case "$UI_VALUE" in
        stats) exec sccache --show-stats ;;
        stop) ui_confirm "Stop the sccache server?"; exec sccache --stop-server ;;
        restart)
            ui_confirm "Restart the sccache server?"
            sccache --stop-server
            exec sccache --start-server
            ;;
    esac
    ui_prompt "Cache directory" "$CACHE_DIR"
    CACHE_DIR="$UI_VALUE"
    if [[ -d "$CACHE_DIR" ]]; then du -sh "$CACHE_DIR"; fi
    ui_confirm "Clear $CACHE_DIR and restart sccache?"
fi
if [[ -L "$CACHE_DIR" ]]; then
    echo "Error: refusing to clear a symlinked cache directory." >&2
    exit 1
fi
if [[ -d "$CACHE_DIR" ]]; then
    CACHE_DIR="$(cd "$CACHE_DIR" && pwd -P)"
    repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
    case "$CACHE_DIR" in
        /|/Users|/Users/Shared|/tmp|/private/tmp|/var|/var/tmp|/private/var/tmp|"$HOME"|"$repo_root")
            echo "Error: refusing to clear a broad directory: $CACHE_DIR" >&2
            exit 1
            ;;
    esac
fi

echo "Stopping sccache server..."
sccache --stop-server

# 2. Identify and remove the cache directory
# This uses the default Linux path; adjust if on macOS/Windows
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
