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
ACT_ARGS=()
INTERACTIVE=false
for arg in "$@"; do
  case "$arg" in
    --interactive|-i) INTERACTIVE=true ;;
    --help|-h)
      echo "Usage: cargo-cross.sh [--interactive] [act options...]"
      echo "Environment: CARGO_CROSS_WORKFLOW, CARGO_CROSS_TAG, CARGO_CROSS_TOOLCHAIN"
      exit 0
      ;;
    *) ACT_ARGS+=("$arg") ;;
  esac
done
if [[ "$INTERACTIVE" == true ]]; then
  # shellcheck source=scripts/lib/script-ui.sh
  source "$SCRIPT_DIR/lib/script-ui.sh"
  ui_prompt "Workflow file" "$WORKFLOW"
  WORKFLOW="$UI_VALUE"
  ui_prompt "Tag or ref to build" "$TAG"
  TAG="$UI_VALUE"
  ui_choose "Rust toolchain" stable nightly
  TOOLCHAIN="$UI_VALUE"
  ui_prompt "Workflow job" build-binaries
  JOB="$UI_VALUE"
  ui_confirm "Run $WORKFLOW / $JOB locally for $TAG / $TOOLCHAIN?"
else
  JOB=build-binaries
fi

if ! command -v act >/dev/null 2>&1; then
  echo "cargo-cross.sh: act is required" >&2
  exit 1
fi

cd "$REPO_ROOT"
exec act workflow_dispatch \
  --bind \
  -W "$WORKFLOW" \
  --job "$JOB" \
  --input "tag=$TAG" \
  --input "toolchain=$TOOLCHAIN" \
  ${ACT_ARGS[@]+"${ACT_ARGS[@]}"}
