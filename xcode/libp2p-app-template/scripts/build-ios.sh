#!/bin/bash
set -euo pipefail

cd "$(dirname "$0")/.."

DEVICE="${1:-iPhone 17}"

echo "==> Building libp2p-app-template for iOS Simulator ($DEVICE)"
xcodebuild -project libp2p-app-template.xcodeproj \
  -scheme LibP2PAppTemplate \
  -sdk iphonesimulator \
  -destination "platform=iOS Simulator,name=$DEVICE" \
  -configuration Debug \
  build

echo "==> iOS build succeeded"
