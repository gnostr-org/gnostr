#!/bin/bash

# Define the project and scheme
PROJECT="gnostore.xcodeproj"
SCHEME="gnostore (iOS)" # This scheme builds the iOS app, which should include the Safari Extension as an embedded target.
CONFIGURATION="Debug"   # Use Debug for development, or Release for a final build.
DESTINATION="generic/platform=iOS"

echo "Building Xcode project: $PROJECT, Scheme: $SCHEME, Configuration: $CONFIGURATION"

npm run build;
npm run watch-tailwind &
sleep 5

xcodebuild -project "$PROJECT" \
           -scheme "$SCHEME" \
           -configuration "$CONFIGURATION" \
           -destination "$DESTINATION" \
           clean

xcodebuild -project "$PROJECT" \
           -scheme "$SCHEME" \
           -configuration "$CONFIGURATION" \
           -destination "$DESTINATION" \
           ARCHS="arm64 x86_64" \
           ONLY_ACTIVE_ARCH=NO \
           build

if [ $? -eq 0 ]; then
    echo "Build successful."
    echo "The Safari Extension for macOS has been built. Now, you need to enable it in Safari."
else
    echo "Build failed."
    exit 1
fi
