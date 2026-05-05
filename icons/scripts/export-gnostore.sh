#!/bin/bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GNOSTORE_ROOT="${1:-$SCRIPT_DIR/../../gnostore}"

if [ ! -d "$GNOSTORE_ROOT" ]; then
    echo "gnostore root not found: $GNOSTORE_ROOT" >&2
    exit 1
fi

GNOSTORE_ROOT="$(cd "$GNOSTORE_ROOT" && pwd)"

APPICON_SOURCE="$SCRIPT_DIR/../Assets.xcassets/AppIcon.appiconset"
BIGICON_SOURCE="$SCRIPT_DIR/../Assets.xcassets/Icon.imageset"
EXT_ICON_SOURCE="$SCRIPT_DIR/../app/gnostr.png"

APPICON_DEST="$GNOSTORE_ROOT/Shared (App)/Assets.xcassets/AppIcon.appiconset"
BIGICON_DEST="$GNOSTORE_ROOT/Shared (App)/Assets.xcassets/bigicon.imageset"
EXT_IMAGE_DEST="$GNOSTORE_ROOT/Shared (Extension)/Resources/images"

sync_asset_dir() {
    local source_dir="$1"
    local dest_dir="$2"

    rm -rf "$dest_dir"
    mkdir -p "$dest_dir"
    cp -R "$source_dir"/. "$dest_dir"/
}

render_png() {
    local source_file="$1"
    local size="$2"
    local dest_file="$3"

    sips -z "$size" "$size" "$source_file" --out "$dest_file" >/dev/null
}

render_sizes() {
    local source_file="$1"
    local dest_dir="$2"
    local prefix="$3"
    shift 3

    mkdir -p "$dest_dir"
    for size in "$@"; do
        render_png "$source_file" "$size" "$dest_dir/${prefix}-${size}.png"
    done
}

echo "Exporting AppIcon to gnostore..."
sync_asset_dir "$APPICON_SOURCE" "$APPICON_DEST"

echo "Exporting bigicon to gnostore..."
sync_asset_dir "$BIGICON_SOURCE" "$BIGICON_DEST"

echo "Exporting extension icons to gnostore..."
render_sizes "$EXT_ICON_SOURCE" "$EXT_IMAGE_DEST" toolbar 16 19 32 38 48 72
render_sizes "$EXT_ICON_SOURCE" "$EXT_IMAGE_DEST" icon 48 96 128 256 512

echo "Export complete."
