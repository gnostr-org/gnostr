#!/usr/bin/env bash
set -euo pipefail

BASH_VERSION_CURRENT="${BASH_VERSION:-unknown}"
BASH_MAJOR="${BASH_VERSINFO[0]:-0}"
BASH_MINOR="${BASH_VERSINFO[1]:-0}"
if ! bash -n "${BASH_SOURCE[0]}"; then
  exit 1
fi

REPO=""
WORKFLOW=""
LIMIT=100
THREADS=2
INTERACTIVE=false
# shellcheck source=scripts/lib/script-ui.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib/script-ui.sh"

usage() {
  cat <<'EOF'
Usage: gh-delete-all-workflow-runs.sh [options]

Delete GitHub Actions workflow runs in a repository while keeping the newest N pages.

Options:
  -r, --repo R         Target repo, e.g. owner/name (default: current repo)
  -w, --workflow W     Limit to one workflow file/name/ID
  -l, --limit N        Keep the newest N pages (default: 100)
  -t, --threads N      Split page deletion across N threads (default: 2)
  -i, --interactive    Select eligible runs and confirm deletion
  -h, --help           Show help

Examples:
  gh-delete-all-workflow-runs.sh
  gh-delete-all-workflow-runs.sh --repo gnostr-org/gnostr
  gh-delete-all-workflow-runs.sh --workflow release.yml
  gh-delete-all-workflow-runs.sh --limit 25
  gh-delete-all-workflow-runs.sh --threads 2
EOF
}

resolve_repo() {
  if [[ -n "$REPO" ]]; then
    printf '%s\n' "$REPO"
    return
  fi

  gh repo view --json nameWithOwner --jq '.nameWithOwner'
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    -i|--interactive)
      INTERACTIVE=true
      shift
      ;;
    -r|--repo)
      REPO="${2:?missing value for --repo}"
      shift 2
      ;;
    -w|--workflow)
      WORKFLOW="${2:?missing value for --workflow}"
      shift 2
      ;;
    -l|--limit)
      LIMIT="${2:?missing value for --limit}"
      shift 2
      ;;
    -t|--threads)
      THREADS="${2:?missing value for --threads}"
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

if [[ "$INTERACTIVE" == true ]]; then
  ui_prompt "Repository" "$(resolve_repo)"
  REPO="$UI_VALUE"
  ui_prompt "Workflow file/name/ID (empty includes all)" "$WORKFLOW"
  WORKFLOW="$UI_VALUE"
  ui_prompt "Newest pages to keep (100 runs per page)" "$LIMIT"
  LIMIT="$UI_VALUE"
fi
if [[ ! "$LIMIT" =~ ^[0-9]+$ || ! "$THREADS" =~ ^[1-9][0-9]*$ ]]; then
  echo "Error: --limit must be nonnegative and --threads must be positive." >&2
  exit 1
fi
LIMIT=$((10#$LIMIT))
repo="$(resolve_repo)"

echo "Target repo: ${repo}"
if [[ -n "$WORKFLOW" ]]; then
  runs_json_path="/repos/$repo/actions/workflows/$WORKFLOW/runs?per_page=100"
  echo "Target workflow: ${WORKFLOW}"
else
  runs_json_path="/repos/$repo/actions/runs?per_page=100"
fi

delete_run() {
  local run_id="$1"
  local output

  if ! output="$(gh api -X DELETE "/repos/$repo/actions/runs/$run_id" 2>&1)"; then
    if grep -q '"status": "403"' <<<"$output"; then
      echo "Error: deleting workflow runs requires GitHub Actions delete permission on $repo." >&2
      echo "Use a token with Actions write access (or refresh gh auth with the actions scope) and try again." >&2
      return 1
    fi

    echo "$output" >&2
    return 1
  fi
}

tmpdir=""
cleanup() {
  if [[ -n "$tmpdir" ]]; then
    rm -rf "$tmpdir"
  fi
}

on_interrupt() {
  echo "Cancelled by user." >&2
  local pids
  pids="$(jobs -pr || true)"
  if [[ -n "$pids" ]]; then
    kill $pids 2>/dev/null || true
  fi
  exit 130
}

trap cleanup EXIT
trap on_interrupt INT TERM

deleted=0
echo "Counting workflow runs..."
total="$(gh api "$runs_json_path" --jq '.total_count')"

if [[ "$total" -eq 0 ]]; then
  echo "No workflow runs found."
else
  pages=$(((total + 99) / 100))
  if [[ "$pages" -le "$LIMIT" ]]; then
    echo "Nothing to delete; only ${pages} pages exist and limit is ${LIMIT}."
  else
    run_rows=()
    # Snapshot the older pages before deletion can shift pagination.
    for ((page=LIMIT + 1; page<=pages; page++)); do
      rows="$(gh api "${runs_json_path}&page=${page}" --jq '.workflow_runs[] | [.id,.status,.name,.head_branch,.created_at] | @tsv')"
      while IFS= read -r row; do
        [[ -z "$row" ]] || run_rows+=("$row")
      done <<<"$rows"
    done
    if [[ "${#run_rows[@]}" -eq 0 ]]; then
      echo "No eligible workflow runs found."
      exit 0
    fi
    if [[ "$INTERACTIVE" == true ]]; then
      ui_select "Eligible runs: ID / status / workflow / branch / created" "${run_rows[@]}"
      run_rows=("${UI_SELECTED[@]}")
      ui_confirm "Permanently delete these ${#run_rows[@]} runs from $repo?"
    fi
    if [[ "$THREADS" -gt "${#run_rows[@]}" ]]; then
      THREADS="${#run_rows[@]}"
    fi
    tmpdir="$(mktemp -d "${TMPDIR:-/tmp}/gh-delete-all-workflow-runs.XXXXXX")"
    pids=()
    for ((thread=0; thread<THREADS; thread++)); do
      (
        thread_deleted=0
        for ((i=thread; i<${#run_rows[@]}; i+=THREADS)); do
          row="${run_rows[i]}"
          run_id="${row%%$'\t'*}"
          echo "Deleting workflow run run_id=${run_id}"
          delete_run "$run_id"
          thread_deleted=$((thread_deleted + 1))
        done
        printf '%s\n' "$thread_deleted" > "$tmpdir/thread-${thread}.count"
      ) &
      pids+=("$!")
    done
    failed=0
    for pid in "${pids[@]}"; do
      if ! wait "$pid"; then
        echo "Error: a workflow deletion worker failed." >&2
        failed=1
      fi
    done
    for count_file in "$tmpdir"/thread-*.count; do
      [[ -e "$count_file" ]] || continue
      deleted=$((deleted + $(cat "$count_file")))
    done
    if [[ "$failed" -ne 0 ]]; then
      exit 1
    fi
  fi
fi

echo "Deleted ${deleted} workflow runs from ${repo}."
