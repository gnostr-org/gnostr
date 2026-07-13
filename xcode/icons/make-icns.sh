#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

rsvg-convert -w 1024 -h 1024 "$SCRIPT_DIR/gnostr.svg" -o "$SCRIPT_DIR/app/gnostr.png"
rsvg-convert -w 1024 -h 1024 "$SCRIPT_DIR/background.svg" -o "$SCRIPT_DIR/app/background.png"

cd "$SCRIPT_DIR"
exec ./scripts/make-icns.sh "$@"
