#!/bin/bash
set -e

# Change to the script's directory to ensure relative paths work correctly
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
cd "$SCRIPT_DIR"

PROJECT="gnostr.xcodeproj"
SCHEME="gnostr"
BUILD_DIR="./build"
CONFIGURATION="Debug"

# Detect Architecture
ARCH=$(uname -m)
echo "--- Building for $ARCH ---"

# 1. Clean and Build
# Using -derivedDataPath keeps your project isolated from global Xcode clutter
xcodebuild -project "$PROJECT" \
           -scheme "$SCHEME" \
           -configuration "$CONFIGURATION" \
           -derivedDataPath "$BUILD_DIR" \
           -destination "platform=macOS,name=My Mac" \
           SUPPORTS_MACCATALYST=YES \
           SUPPORTS_IOS_DESIGNED_FOR_MAC=YES \
           clean build

# 2. Bundle Verification
# Instead of hardcoding the Kingfisher path, we find it within the local build folder
echo "--- Verifying Resources ---"
APP_PATH=$(find "$BUILD_DIR" -name "${SCHEME}.app" -type d | head -n 1)
RESOURCES_DIR="$APP_PATH/Contents/Resources"

# Check if Kingfisher was bundled automatically
if [ ! -d "$RESOURCES_DIR/Kingfisher_Kingfisher.bundle" ]; then
    echo "Kingfisher bundle missing. Searching in build artifacts..."
    # Logic to find and copy if SPM failed to embed
    # ...
fi

echo "Build complete: $APP_PATH"
