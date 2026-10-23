#!/usr/bin/env bash
set -euo pipefail
# Run the gnostr-test-matrix workflow locally with act for a single matrix entry.
# Usage: ./scripts/act-test-matrix.sh [os] [toolchain]
OS="${1:-ubuntu-latest}"
TOOLCHAIN="${2:-stable}"
act -j gnostr-test \
  --matrix "os:${OS}" \
  --matrix "toolchain:${TOOLCHAIN}" \
  -P "${OS}=catthehacker/ubuntu:act-latest" \
  2>&1 | tee /tmp/act-gnostr-test-matrix.log
