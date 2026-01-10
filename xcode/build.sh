#!/bin/bash

# Exit on error
set -e

# Set project and scheme
PROJECT="gnostr.xcodeproj"
SCHEME="gnostr"

# Set build directory for Mac Catalyst
BUILD_DIR="./build/Build/Products"

# Determine architecture for Mac Catalyst
# For Intel Macs, it's x86_64. For Apple Silicon Macs, it's arm64.
# We can dynamically determine this, or build for both (fat binary).
# For "Designed for iPad" usually means Universal build for Mac Catalyst.
ARCH="$(arch)"
if [ "$ARCH" == "arm64" ]; then
    DEST_ARCH="arm64"
else
    DEST_ARCH="x86_64"
fi

# Clean build folder
echo "Cleaning build folder for Mac Catalyst ($DEST_ARCH)..."
xcodebuild -project "$PROJECT" \
           -scheme "$SCHEME" \
           -sdk macosx \
           -configuration "Debug" \
           -destination "platform=macOS,arch=$DEST_ARCH,variant=Mac Catalyst" \
           clean

# Build the project for Mac Catalyst "Designed for iPad" 
echo "Building project for Mac Catalyst \"Designed for iPad\" ($DEST_ARCH)..."
xcodebuild -project "$PROJECT" \
           -scheme "$SCHEME" \
           -sdk macosx \
           -configuration "Debug" \
           -destination "platform=macOS,arch=$DEST_ARCH,variant=Mac Catalyst" \
           build \
           V=YES \
           SUPPORTS_MACCATALYST=YES \
           SUPPORTS_IOS_DESIGNED_FOR_MAC=YES \
           SYMROOT="${BUILD_DIR}" \
           -allowProvisioningUpdates

### Locate Kingfisher bundle in DerivedData and copy it
echo "Locating and copying Kingfisher_Kingfisher.bundle..."
# Find the actual DerivedData path
DERIVED_DATA_PATH=$(xcodebuild -showBuildSettings -project "$PROJECT" -scheme "$SCHEME" -sdk macosx -configuration "Debug" -destination "platform=macOS,arch=$DEST_ARCH,variant=Mac Catalyst" | grep -E "^\s*BUILD_DIR\s*=" | sed 's/.*= *//' | sed 's/\(.*\)\/Build\/Products.*/\1/')

KINGFISHER_BUNDLE_SOURCE_PATH="$DERIVED_DATA_PATH/SourcePackages/checkouts/Kingfisher/build/Build/Products/Debug-maccatalyst/Kingfisher_Kingfisher.bundle"
APP_RESOURCES_DEST_PATH="${BUILD_DIR}/Debug-maccatalyst/${SCHEME}.app/Contents/Resources/"

if [ -d "$KINGFISHER_BUNDLE_SOURCE_PATH" ]; then
    echo "Copying Kingfisher_Kingfisher.bundle from $KINGFISHER_BUNDLE_SOURCE_PATH to $APP_RESOURCES_DEST_PATH"
    cp -R "$KINGFISHER_BUNDLE_SOURCE_PATH" "$APP_RESOURCES_DEST_PATH"
else
    echo "Warning: Kingfisher_Kingfisher.bundle not found at expected DerivedData location: $KINGFISHER_BUNDLE_SOURCE_PATH"
fi

echo "Build complete."
echo "Output will be in ${BUILD_DIR}/Debug-maccatalyst"
