#!/bin/bash
set -euo pipefail

cd "$(dirname "$0")/.."

ICONS_DIR="${1:-../icons}"
SOURCE_APPICON="$ICONS_DIR/Assets.xcassets/AppIcon.appiconset"
TARGET_APPICON="Assets.xcassets/AppIcon.appiconset"

if [ ! -d "$SOURCE_APPICON" ]; then
    echo "Error: Source icon set not found at $SOURCE_APPICON"
    echo "Usage: $0 [path/to/icons/repo]"
    exit 1
fi

echo "==> Syncing app icons from $ICONS_DIR"

# Clear existing icons
rm -f "$TARGET_APPICON"/*.png
rm -f "$TARGET_APPICON"/Contents.json

# Copy Contents.json
cp "$SOURCE_APPICON/Contents.json" "$TARGET_APPICON/"

# Copy all PNGs referenced in Contents.json
for filename in $(grep '"filename"' "$TARGET_APPICON/Contents.json" | sed 's/.*"filename" : "\([^"]*\)".*/\1/' | sort -u); do
    src="$SOURCE_APPICON/$filename"
    if [ -f "$src" ]; then
        cp "$src" "$TARGET_APPICON/"
        echo "    Copied $filename"
    else
        echo "    Warning: $filename not found in source"
    fi
done

echo "==> App icons synced"
