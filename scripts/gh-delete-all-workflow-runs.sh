#!/usr/bin/env bash
set -euo pipefail

REPO=""
WORKFLOW=""
LIMIT=100
THREADS=2

usage() {
  cat <<'EOF'
Usage: gh-delete-all-workflow-runs.sh [options]

Delete GitHub Actions workflow runs in a repository while keeping the newest N pages.

Options:
  -r, --repo R         Target repo, e.g. owner/name (default: current repo)
  -w, --workflow W     Limit to one workflow file/name/ID
  -l, --limit N        Keep the newest N pages (default: 100)
  -t, --threads N      Split page deletion across N threads (default: 2)
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

deleted=0
echo "Counting workflow runs..."
total="$(gh api "$runs_json_path" --jq '.total_count')"

if [[ "$total" -eq 0 ]]; then
  echo "No workflow runs found."
else
  pages=$(((total + 99) / 100))
  delete_through_page=$((pages - LIMIT))
  if [[ "$delete_through_page" -le 0 ]]; then
    echo "Nothing to delete; only ${pages} pages exist and limit is ${LIMIT}."
  else
    if [[ "$THREADS" -lt 1 ]]; then
      echo "Error: --threads must be at least 1." >&2
      exit 1
    fi

    if [[ "$THREADS" -gt "$delete_through_page" ]]; then
      THREADS="$delete_through_page"
    fi

    tmpdir="$(mktemp -d)"
    trap 'rm -rf "$tmpdir"' EXIT

    for thread in $(seq 1 "$THREADS"); do
      (
        thread_deleted=0
        page="$thread"
        while [[ "$page" -le "$delete_through_page" ]]; do
          echo "Listing workflow runs on page ${page} (thread ${thread}/${THREADS})"
          run_ids=()
          while IFS= read -r run_id; do
            [[ -z "$run_id" ]] && continue
            run_ids+=("$run_id")
          done < <(gh api "${runs_json_path}&page=${page}" --jq '.workflow_runs[].id')

          for ((i=${#run_ids[@]} - 1; i >= 0; i--)); do
            run_id="${run_ids[$i]}"
            echo "Deleting workflow run run_id=${run_id} (page ${page}, thread ${thread}/${THREADS})"
            delete_run "$run_id"
            thread_deleted=$((thread_deleted + 1))
          done

          page=$((page + THREADS))
        done

        printf '%s\n' "$thread_deleted" > "$tmpdir/thread-${thread}.count"
      ) &
    done

    wait

    for count_file in "$tmpdir"/thread-*.count; do
      [[ -e "$count_file" ]] || continue
      deleted=$((deleted + $(cat "$count_file")))
    done
  fi
fi

echo "Deleted ${deleted} workflow runs from ${repo}."
