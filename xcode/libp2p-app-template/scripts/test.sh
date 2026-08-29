#!/bin/bash
set -euo pipefail

cd "$(dirname "$0")/.."

echo "==> Testing libp2p-app-template"

# Note: The scheme must have a Test action configured in Xcode for this to work.
# If tests are not configured, open the project in Xcode and add the test targets
# to the scheme's Test action.

xcodebuild -project libp2p-app-template.xcodeproj \
  -scheme LibP2PAppTemplate \
  -destination 'platform=macOS' \
  test || {
    echo ""
    echo "Tests failed or no test target is configured."
    echo "To enable tests, open the project in Xcode, edit the scheme, and add"
    echo "libp2p-app-templateTests to the Test action."
    exit 1
  }

echo "==> Tests passed"
