#!/bin/bash
set -euo pipefail

cd "$(dirname "$0")/.."

echo "==> Testing libp2p-app-template"

# The Xcode project scheme does not include a Test action, but the Swift
# Package Manager test target (defined in Package.swift) works from the
# command line and is the canonical test runner for this repo.
swift test

echo "==> Tests passed"
