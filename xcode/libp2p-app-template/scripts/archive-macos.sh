#!/bin/bash
set -euo pipefail

cd "$(dirname "$0")/.."

OUTPUT_PATH="${1:-build/archive}"

echo "==> Archiving libp2p-app-template for macOS (Mac Catalyst)"

xcodebuild -project libp2p-app-template.xcodeproj \
  -scheme LibP2PAppTemplate \
  -destination 'platform=macOS,variant=Mac Catalyst' \
  -configuration Release \
  -archivePath "$OUTPUT_PATH/LibP2PAppTemplate.xcarchive" \
  archive

echo "==> Archive created at $OUTPUT_PATH/LibP2PAppTemplate.xcarchive"
