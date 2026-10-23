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
INTERACTIVE=false
# shellcheck source=scripts/lib/script-ui.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib/script-ui.sh"

usage() {
  cat <<'EOF'
Usage: gh-workflow.sh [options]

Cancel GitHub Actions runs for the current repo.

Options:
  -d, --depth N       Max number of runs to inspect per status (default: 50)
  -w, --workflow W    Limit to a workflow file/name/ID
  -r, --repo R        Target repo, e.g. owner/name (default: current repo)
  -s, --status S      Status to cancel; may be repeated
  -i, --interactive   Select runs from a numbered list and confirm cancellation
  -h, --help          Show help

Examples:
  gh-workflow.sh
  gh-workflow.sh --workflow gnostr-test-matrix.yml
  gh-workflow.sh --status queued --status in_progress
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    -i|--interactive)
      INTERACTIVE=true
      shift
      ;;
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

run_rows=()
cancel_status() {
  local status="$1"
  local ids
  local list_args=(-s "$status" -L "$DEPTH")
  if [[ "$INTERACTIVE" == true ]]; then
    list_args+=(--json "databaseId,status,workflowName,headBranch" --jq '.[] | [.databaseId,.status,.workflowName,.headBranch] | @tsv')
  else
    list_args+=(--json databaseId --jq '.[] | .databaseId')
  fi

  if [[ -n "$REPO" ]]; then
    list_args+=(-R "$REPO")
  fi

  if [[ -n "$WORKFLOW" ]]; then
    list_args+=(-w "$WORKFLOW")
  fi

  ids="$(gh run list "${list_args[@]}")"
  if [[ "$INTERACTIVE" == true ]]; then
    while IFS= read -r id; do
      [[ -z "$id" ]] || run_rows+=("$id")
    done <<<"$ids"
    return
  fi
  if [[ -z "${ids// }" ]]; then
    return 0
  fi

  for id in $ids; do
    gh run cancel "$id"
  done
}

if [[ "$INTERACTIVE" == true ]]; then
  ui_prompt "Repository (empty uses current repository)" "$REPO"
  REPO="$UI_VALUE"
  ui_prompt "Workflow file/name/ID (empty includes all)" "$WORKFLOW"
  WORKFLOW="$UI_VALUE"
  ui_select "Run statuses" "${STATUSES[@]}"
  STATUSES=("${UI_SELECTED[@]}")
fi

for status in "${STATUSES[@]}"; do
  cancel_status "$status"
done

if [[ "$INTERACTIVE" == true ]]; then
  ui_select "Workflow runs: ID / status / workflow / branch" ${run_rows[@]+"${run_rows[@]}"}
  ui_confirm "Cancel these ${#UI_SELECTED[@]} selected runs?"
  failed=0
  cancel_args=()
  [[ -z "$REPO" ]] || cancel_args+=(--repo "$REPO")
  for row in "${UI_SELECTED[@]}"; do
    if ! gh run cancel "${row%%$'\t'*}" ${cancel_args[@]+"${cancel_args[@]}"}; then
      printf 'Error: could not cancel run %s.\n' "${row%%$'\t'*}" >&2
      failed=1
    fi
  done
  exit "$failed"
fi
