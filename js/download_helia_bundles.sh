#!/usr/bin/env bash

set -e

TARGET_DIR="./src/js"
mkdir -p "$TARGET_DIR"

echo "==========================================="
echo " Downloading Helia Production CDN Bundles"
echo "==========================================="

declare -a PACKAGES=(
    "helia:helia@latest/dist/index.min.js"
    "helia-unixfs:@helia/unixfs@latest/dist/index.min.js"
    "helia-json:@helia/json@latest/dist/index.min.js"
    "helia-verified-fetch:@helia/verified-fetch@latest/dist/index.min.js"
    "multiformats:multiformats@latest/dist/index.min.js"
)

for item in "${PACKAGES[@]}"; do
    LOCAL_NAME="${item%%:*}"
    CDN_PATH="${item#*:}"

    # jsDelivr serves npm packages from cdn.jsdelivr.net/npm/<package>
    JS_URL="https://cdn.jsdelivr.net/npm/${CDN_PATH}"
    echo "Downloading: ${LOCAL_NAME}.min.js ..."
    curl -fsSL "$JS_URL" -o "${TARGET_DIR}/${LOCAL_NAME}.min.js"

    MAP_URL="${JS_URL}.map"
    echo "Downloading: ${LOCAL_NAME}.min.js.map ..."
    curl -fsSL "$MAP_URL" -o "${TARGET_DIR}/${LOCAL_NAME}.min.js.map"

    echo "-------------------------------------------"
done

echo "Success! All bundles downloaded to: $TARGET_DIR"
ls -lh "$TARGET_DIR"
