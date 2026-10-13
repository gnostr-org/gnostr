#!/bin/bash
set -euo pipefail

cd "$(dirname "$0")/.."

echo "==> Opening libp2p-app-template.xcodeproj"
open libp2p-app-template.xcodeproj
