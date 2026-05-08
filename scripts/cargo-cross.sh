#!/usr/bin/env bash
BASH_VERSION_CURRENT="${BASH_VERSION:-unknown}"
BASH_MAJOR="${BASH_VERSINFO[0]:-0}"
BASH_MINOR="${BASH_VERSINFO[1]:-0}"
if ! bash -n "${BASH_SOURCE[0]}"; then
  exit 1
fi

set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd "${SCRIPT_DIR}/.." && pwd)
WORKFLOW=${CARGO_CROSS_WORKFLOW:-.github/workflows/cargo-cross.yml}
TAG=${CARGO_CROSS_TAG:-$(git -C "$REPO_ROOT" describe --tags --always --dirty --abbrev=7)}
TOOLCHAIN=${CARGO_CROSS_TOOLCHAIN:-stable}

if ! command -v act >/dev/null 2>&1; then
  echo "cargo-cross.sh: act is required" >&2
  exit 1
fi

cd "$REPO_ROOT"
exec act workflow_dispatch \
  --bind \
  -W "$WORKFLOW" \
  --job build-binaries \
  --input "tag=$TAG" \
  --input "toolchain=$TOOLCHAIN" \
  "$@"
