#!/bin/bash
set -euo pipefail

cd "$(dirname "$0")/.."

echo "==> Building libp2p-app-template (default platform)"
xcodebuild -project libp2p-app-template.xcodeproj \
  -scheme LibP2PAppTemplate \
  -configuration Debug \
  build

echo "==> Build succeeded"
