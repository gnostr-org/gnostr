#!/bin/bash
set -euo pipefail

cd "$(dirname "$0")/.."

echo "==> Building libp2p-app-template for macOS (Mac Catalyst)"
xcodebuild -project libp2p-app-template.xcodeproj \
  -scheme LibP2PAppTemplate \
  -destination 'platform=macOS,variant=Mac Catalyst' \
  -configuration Debug \
  build

echo "==> macOS build succeeded"
