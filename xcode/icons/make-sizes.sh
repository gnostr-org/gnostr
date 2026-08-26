#!/usr/bin/env bash

exec "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/scripts/render-sizes.sh" \
  square \
  "${1:-$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/1024x1024.png}" \
  "${2:-$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)}"
