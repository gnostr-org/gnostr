#!/usr/bin/env bash
set -euo pipefail
# Run the gnostr-test-matrix workflow locally with act for a single matrix entry.
# Usage: ./scripts/act-test-matrix.sh [os] [toolchain]
INTERACTIVE=false
if [[ "${1:-}" == --interactive || "${1:-}" == -i ]]; then
  INTERACTIVE=true
  shift
fi
if [[ "${1:-}" == --help || "${1:-}" == -h ]]; then
  echo "Usage: act-test-matrix.sh [--interactive] [os] [toolchain]"
  exit 0
fi
OS="${1:-ubuntu-latest}"
TOOLCHAIN="${2:-stable}"
RUNNER_IMAGE="catthehacker/ubuntu:act-latest"
if [[ "$INTERACTIVE" == true ]]; then
  # shellcheck source=scripts/lib/script-ui.sh
  source "$(dirname "${BASH_SOURCE[0]}")/lib/script-ui.sh"
  ui_choose "Matrix OS" ubuntu-latest macos-latest
  OS="$UI_VALUE"
  ui_choose "Rust toolchain" stable nightly
  TOOLCHAIN="$UI_VALUE"
  [[ "$OS" != macos-latest ]] || RUNNER_IMAGE="-self-hosted"
  ui_prompt "act runner image (-self-hosted uses this host)" "$RUNNER_IMAGE"
  RUNNER_IMAGE="$UI_VALUE"
  ui_confirm "Run gnostr-test locally for $OS / $TOOLCHAIN using $RUNNER_IMAGE?"
fi
act -j gnostr-test \
  --matrix "os:${OS}" \
  --matrix "toolchain:${TOOLCHAIN}" \
  -P "${OS}=${RUNNER_IMAGE}" \
  2>&1 | tee /tmp/act-gnostr-test-matrix.log
