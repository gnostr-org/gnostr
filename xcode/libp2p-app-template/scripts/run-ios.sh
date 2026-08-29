#!/bin/bash
set -euo pipefail

cd "$(dirname "$0")/.."

DEVICE="${1:-iPhone 17}"

echo "==> Building and running libp2p-app-template on iOS Simulator ($DEVICE)"

xcodebuild -project libp2p-app-template.xcodeproj \
  -scheme LibP2PAppTemplate \
  -sdk iphonesimulator \
  -destination "platform=iOS Simulator,name=$DEVICE" \
  -configuration Debug \
  build

# Find the bundle ID
BUNDLE_ID=$(plutil -extract CFBundleIdentifier raw \
  ~/Library/Developer/Xcode/DerivedData/libp2p-app-template-*/Build/Products/Debug-iphonesimulator/LibP2PAppTemplate.app/Info.plist 2>/dev/null || true)

if [ -z "$BUNDLE_ID" ]; then
    echo "Warning: Could not determine bundle ID. App built but not launched."
    exit 0
fi

# Boot the simulator if needed
SIM_UDID=$(xcrun simctl list devices available | grep -E "$DEVICE \(" | head -1 | sed -E 's/.*\(([A-F0-9-]+)\).*/\1/')
if [ -n "$SIM_UDID" ]; then
    xcrun simctl boot "$SIM_UDID" 2>/dev/null || true
    open -a Simulator
    sleep 2
    xcrun simctl install "$SIM_UDID" ~/Library/Developer/Xcode/DerivedData/libp2p-app-template-*/Build/Products/Debug-iphonesimulator/LibP2PAppTemplate.app
    xcrun simctl launch "$SIM_UDID" "$BUNDLE_ID"
    echo "==> Launched $BUNDLE_ID on $DEVICE"
else
    echo "Warning: Could not find simulator UDID for '$DEVICE'"
fi
