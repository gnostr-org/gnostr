#!/usr/bin/env bash

set -e

TARGET_DIR="./src/js"
mkdir -p "$TARGET_DIR"

echo "==========================================="
echo " Downloading Vendored JS/CDN Bundles"
echo "==========================================="

# Helia ecosystem packages served by jsDelivr.
declare -a HELIA_PACKAGES=(
    "helia:helia@latest/dist/index.min.js"
    "helia-unixfs:@helia/unixfs@latest/dist/index.min.js"
    "helia-json:@helia/json@latest/dist/index.min.js"
    "helia-verified-fetch:@helia/verified-fetch@latest/dist/index.min.js"
    "multiformats:multiformats@latest/dist/index.min.js"
)

for item in "${HELIA_PACKAGES[@]}"; do
    LOCAL_NAME="${item%%:*}"
    CDN_PATH="${item#*:}"

    JS_URL="https://cdn.jsdelivr.net/npm/${CDN_PATH}"
    echo "Downloading: ${LOCAL_NAME}.min.js ..."
    curl -fsSL "$JS_URL" -o "${TARGET_DIR}/${LOCAL_NAME}.min.js"

    MAP_URL="${JS_URL}.map"
    echo "Downloading: ${LOCAL_NAME}.min.js.map ..."
    curl -fsSL "$MAP_URL" -o "${TARGET_DIR}/${LOCAL_NAME}.min.js.map" || true

    echo "-------------------------------------------"
done

# Other vendored third-party bundles and their remote CDN sources.
declare -a OTHER_BUNDLES=(
    "bip32@2.0.6.js:https://bundle.run/bip32@2.0.6"
    "buffer@6.0.3.js:https://bundle.run/buffer@6.0.3"
    "browserify-cipher@1.0.1.js:https://bundle.run/browserify-cipher@1.0.1"
    "noble-secp256k1@1.2.14.js:https://bundle.run/noble-secp256k1@1.2.14"
    "bitcoinjs-lib.js:https://bitcoincore.tech/apps/bitcoinjs-ui/lib/bitcoinjs-lib.js"
    "isomorphic-git.js:https://unpkg.com/isomorphic-git@1.37.6"
)

for item in "${OTHER_BUNDLES[@]}"; do
    LOCAL_NAME="${item%%:*}"
    BUNDLE_URL="${item#*:}"

    echo "Downloading: ${LOCAL_NAME} ..."
    if ! curl -fsSL "$BUNDLE_URL" -o "${TARGET_DIR}/${LOCAL_NAME}"; then
        echo "Warning: could not download ${LOCAL_NAME} from ${BUNDLE_URL}; keeping existing file."
    fi

    echo "-------------------------------------------"
done

echo "Success! Bundles synced to: $TARGET_DIR"
ls -lh "$TARGET_DIR"/helia*.min.js "$TARGET_DIR"/multiformats*.min.js "$TARGET_DIR"/bip32@*.js "$TARGET_DIR"/buffer@*.js "$TARGET_DIR"/browserify-cipher@*.js "$TARGET_DIR"/noble-secp256k1@*.js "$TARGET_DIR"/bitcoinjs-lib.js "$TARGET_DIR"/isomorphic-git.js