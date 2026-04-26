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

if [[ -n "$WORKFLOW" ]]; then
  runs_json_path="/repos/$repo/actions/workflows/$WORKFLOW/runs?per_page=100"
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
while IFS= read -r run_id; do
  [[ -z "$run_id" ]] && continue
  delete_run "$run_id"
  deleted=$((deleted + 1))
done < <(gh api --paginate "$runs_json_path" --jq '.workflow_runs[].id')

echo "Deleted ${deleted} workflow runs from ${repo}."
