#!/usr/bin/env bash

set -euo pipefail

render_variants() {
  local source_file="$1"
  shift

  for variant in "$@"; do
    local size="${variant%%:*}"
    local output_name="${variant#*:}"

    sips -z "$size" "$size" "$source_file" --out "icon.iconset/$output_name" >/dev/null
  done
}

mkdir -p icon.iconset

render_variants app/gnostr.png \
  16:icon_16x16.png \
  32:icon_16x16@2x.png \
  32:icon_32x32.png \
  64:icon_32x32@2x.png \
  128:icon_128x128.png \
  256:icon_128x128@2x.png \
  256:icon_256x256.png \
  512:icon_256x256@2x.png \
  512:icon_512x512.png \
  1024:icon_1024x1024.png \
  2048:icon_1024x1024@2x.png

render_variants app/background.png \
  16:background-icon_16x16.png \
  32:background-icon_16x16@2x.png \
  32:background-icon_32x32.png \
  64:background-icon_32x32@2x.png \
  128:background-icon_128x128.png \
  256:background-icon_128x128@2x.png \
  256:background-icon_256x256.png \
  512:background-icon_256x256@2x.png \
  512:background-icon_512x512.png \
  1024:background-icon_1024x1024.png \
  2048:background-icon_1024x1024@2x.png

iconutil -c icns --output icon.icns icon.iconset
rm -R icon.iconset
