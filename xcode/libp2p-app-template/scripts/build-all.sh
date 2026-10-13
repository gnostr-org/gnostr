#!/bin/bash
set -euo pipefail

cd "$(dirname "$0")/.."

echo "==> Building libp2p-app-template for all platforms"

./scripts/build-ios.sh "$@"
echo ""
./scripts/build-macos.sh "$@"

echo ""
echo "==> All builds succeeded"
