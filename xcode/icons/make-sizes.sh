#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

rsvg-convert -w 1024 -h 1024 "$SCRIPT_DIR/gnostr.svg" -o "$SCRIPT_DIR/app/gnostr.png"
rsvg-convert -w 1024 -h 1024 "$SCRIPT_DIR/background.svg" -o "$SCRIPT_DIR/app/background.png"
rsvg-convert -w 3072 -h 1024 "$SCRIPT_DIR/banner.svg" -o "$SCRIPT_DIR/banner/icon3072x1024.png"

"$SCRIPT_DIR/scripts/render-sizes.sh" \
  square \
  "${1:-$SCRIPT_DIR/icon-circle.svg}" \
  "${2:-$SCRIPT_DIR}"

"$SCRIPT_DIR/scripts/render-sizes.sh" \
  appicon \
  "$SCRIPT_DIR/icon-circle.svg" \
  "$SCRIPT_DIR/Assets.xcassets/IconCircle.imageset"

"$SCRIPT_DIR/scripts/render-sizes.sh" \
  appicon \
  "$SCRIPT_DIR/icon-circle-white.svg" \
  "$SCRIPT_DIR/Assets.xcassets/IconCircleWhite.imageset"

"$SCRIPT_DIR/scripts/render-sizes.sh" \
  appicon \
  "$SCRIPT_DIR/background.svg" \
  "$SCRIPT_DIR/Assets.xcassets/Background.imageset"

"$SCRIPT_DIR/scripts/render-sizes.sh" \
  banner \
  "$SCRIPT_DIR/banner.svg" \
  "$SCRIPT_DIR/Assets.xcassets/Banner.imageset"
