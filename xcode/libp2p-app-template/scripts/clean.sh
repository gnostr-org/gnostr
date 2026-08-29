#!/bin/bash
set -euo pipefail

cd "$(dirname "$0")/.."

echo "==> Cleaning build products"
xcodebuild -project libp2p-app-template.xcodeproj \
  -scheme LibP2PAppTemplate \
  clean

echo "==> Cleaning DerivedData"
rm -rf ~/Library/Developer/Xcode/DerivedData/libp2p-app-template-*

echo "==> Clean complete"
