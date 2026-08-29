#!/bin/bash
set -euo pipefail

cd "$(dirname "$0")/.."

REQUIRED_REPOS=(
    "../swift-libp2p"
    "../swift-libp2p-noise"
    "../swift-libp2p-yamux"
    "../swift-libp2p-dcutr"
    "../swift-libp2p-mdns"
    "../swift-libp2p-kad-dht"
    "../swift-libp2p-pubsub"
    "../swift-libp2p-core"
    "../swift-peer-id"
    "../swift-multiaddr"
    "../swift-multihash"
    "../swift-libp2p-crypto"
    "../swift-multibase"
    "../swift-varint"
    "../swift-noise"
)

MISSING=0

echo "==> Checking sibling dependencies"
for repo in "${REQUIRED_REPOS[@]}"; do
    if [ -d "$repo" ]; then
        echo "  [OK] $repo"
    else
        echo "  [MISSING] $repo"
        MISSING=1
    fi
done

if [ $MISSING -ne 0 ]; then
    echo ""
    echo "Some sibling repositories are missing. They are expected in the parent directory:"
    echo "  $(cd .. && pwd)"
    echo ""
    echo "Clone them from https://github.com/gnostr (or the appropriate org) and re-run."
    exit 1
fi

echo ""
echo "==> All dependencies present. Resolving packages..."

xcodebuild -project libp2p-app-template.xcodeproj \
  -scheme LibP2PAppTemplate \
  -resolvePackageDependencies

echo ""
echo "==> Bootstrap complete"
