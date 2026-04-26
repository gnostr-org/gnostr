#!/usr/bin/env bash
set -euo pipefail

REPO=""
WORKFLOW=""

usage() {
  cat <<'EOF'
Usage: gh-delete-all-workflow-runs.sh [options]

Delete all GitHub Actions workflow runs in a repository.

Options:
  -r, --repo R         Target repo, e.g. owner/name (default: current repo)
  -w, --workflow W     Limit to one workflow file/name/ID
  -h, --help           Show help

Examples:
  gh-delete-all-workflow-runs.sh
  gh-delete-all-workflow-runs.sh --repo gnostr-org/gnostr
  gh-delete-all-workflow-runs.sh --workflow release.yml
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
  page=1
  current=0
  while :; do
    echo "Listing workflow runs on page ${page}..."
    run_ids=()
    while IFS= read -r run_id; do
      [[ -z "$run_id" ]] && continue
      run_ids+=("$run_id")
    done < <(gh api "${runs_json_path}&page=${page}" --jq '.workflow_runs[].id')

    if [[ "${#run_ids[@]}" -eq 0 ]]; then
      break
    fi

    for run_id in "${run_ids[@]}"; do
      current=$((current + 1))
      echo "Deleting workflow run ${current}/${total} run_id=${run_id}..."
      delete_run "$run_id"
      deleted=$((deleted + 1))
    done

    page=$((page + 1))
  done
fi

echo "Deleted ${deleted} workflow runs from ${repo}."
