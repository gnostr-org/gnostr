#!/bin/bash
set -euo pipefail

cd "$(dirname "$0")/.."

echo "==> Building libp2p-app-template for macOS"
xcodebuild -project libp2p-app-template.xcodeproj \
  -scheme LibP2PAppTemplate \
  -sdk macosx \
  -destination 'platform=macOS' \
  -configuration Debug \
  build

echo "==> macOS build succeeded"
