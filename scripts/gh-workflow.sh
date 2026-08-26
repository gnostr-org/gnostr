#!/usr/bin/env bash
set -euo pipefail

BASH_VERSION_CURRENT="${BASH_VERSION:-unknown}"
BASH_MAJOR="${BASH_VERSINFO[0]:-0}"
BASH_MINOR="${BASH_VERSINFO[1]:-0}"
if ! bash -n "${BASH_SOURCE[0]}"; then
  exit 1
fi

DEPTH=50
WORKFLOW=""
REPO=""
STATUSES=("in_progress" "queued" "waiting" "requested" "pending" "action_required")
STATUS_OVERRIDDEN=false

usage() {
  cat <<'EOF'
Usage: gh-workflow.sh [options]

Cancel GitHub Actions runs for the current repo.

Options:
  -d, --depth N       Max number of runs to inspect per status (default: 50)
  -w, --workflow W    Limit to a workflow file/name/ID
  -r, --repo R        Target repo, e.g. owner/name (default: current repo)
  -s, --status S      Status to cancel; may be repeated
  -h, --help          Show help

Examples:
  gh-workflow.sh
  gh-workflow.sh --workflow gnostr-test-matrix.yml
  gh-workflow.sh --status queued --status in_progress
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    -d|--depth)
      DEPTH="${2:?missing value for --depth}"
      shift 2
      ;;
    -w|--workflow)
      WORKFLOW="${2:?missing value for --workflow}"
      shift 2
      ;;
    -r|--repo)
      REPO="${2:?missing value for --repo}"
      shift 2
      ;;
    -s|--status)
      if [[ "$STATUS_OVERRIDDEN" == false ]]; then
        STATUSES=()
        STATUS_OVERRIDDEN=true
      fi
      STATUSES+=("${2:?missing value for --status}")
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

cancel_status() {
  local status="$1"
  local ids
  local list_args=(-s "$status" -L "$DEPTH" --json databaseId --jq '.[] | .databaseId')

  if [[ -n "$REPO" ]]; then
    list_args+=(-R "$REPO")
  fi

  if [[ -n "$WORKFLOW" ]]; then
    list_args+=(-w "$WORKFLOW")
  fi

  ids="$(gh run list "${list_args[@]}" | tr '\n' ' ')"
  if [[ -z "${ids// }" ]]; then
    return 0
  fi

  for id in $ids; do
    gh run cancel "$id"
  done
}

for status in "${STATUSES[@]}"; do
  cancel_status "$status"
done
