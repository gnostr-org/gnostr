#!/usr/bin/env bash

exec "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/scripts/render-sizes.sh" \
  banner \
  "${1:-$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/icon3072x1024.png}" \
  "${2:-$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)}"
