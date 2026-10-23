#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: gh-cancel-queue.sh [DEPTH]

List in-progress, queued, and waiting workflow runs in the current repository.
DEPTH is the maximum number fetched per status (default: 50).

Enter list numbers separated by spaces or commas (for example: 1,3 5),
all to select every listed run, or q to quit. Cancellation requires confirmation.

Checks: bash -n scripts/gh-cancel-queue.sh
        shellcheck scripts/gh-cancel-queue.sh
        python3 -m unittest discover -s scripts/tests -p 'test_gh_cancel_queue.py' -v
EOF
}

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  usage
  exit 0
fi

DEPTH="${1:-50}"
if [[ $# -gt 1 || ! "$DEPTH" =~ ^[1-9][0-9]*$ ]]; then
  echo "Error: DEPTH must be a positive integer." >&2
  usage >&2
  exit 1
fi

run_ids=()
printf '=== ACTIVE WORKFLOW RUNS ===\n'
printf '  #  RUN ID\tSTATUS\tWORKFLOW\tBRANCH\tCREATED\n'
for status in in_progress queued waiting; do
  rows="$(gh run list --status "$status" --limit "$DEPTH" \
    --json createdAt,databaseId,status,workflowName,headBranch \
    --jq 'sort_by(.createdAt, .databaseId) | .[] | [.databaseId, .status, .workflowName, .headBranch, .createdAt] | @tsv')"
  while IFS= read -r row; do
    [[ -z "$row" ]] && continue
    run_ids+=("${row%%$'\t'*}")
    printf '%3d) %s\n' "${#run_ids[@]}" "$row"
  done <<<"$rows"
done

run_count="${#run_ids[@]}"
if [[ "$run_count" -eq 0 ]]; then
  echo "=== NO ACTIVE WORKFLOW RUNS ==="
  exit 0
fi

printf '\nSelect numbers (spaces or commas), all, or q to quit: '
if ! IFS= read -r selection; then
  printf '\nNo runs cancelled.\n'
  exit 0
fi

choices=()
read -r -a choices <<<"${selection//,/ }"
selected=()
for choice in "${choices[@]}"; do
  case "$choice" in
    q|quit)
      echo "No runs cancelled."
      exit 0
      ;;
    all)
      for index in "${!run_ids[@]}"; do
        selected[index]=1
      done
      ;;
    *)
      if [[ ! "$choice" =~ ^[1-9][0-9]*$ || ${#choice} -gt ${#run_count} ]] ||
        (( choice > run_count )); then
        printf 'Error: invalid selection "%s"; choose numbers 1-%s, all, or q.\n' "$choice" "$run_count" >&2
        exit 1
      fi
      selected[choice-1]=1
      ;;
  esac
done

if [[ "${#selected[@]}" -eq 0 ]]; then
  echo "No runs cancelled."
  exit 0
fi

printf '\nSelected run IDs:\n'
for index in "${!selected[@]}"; do
  printf '  %s\n' "${run_ids[index]}"
done
printf 'Cancel these %s workflow runs? [y/N]: ' "${#selected[@]}"
if ! IFS= read -r confirmation || [[ "$confirmation" != y && "$confirmation" != Y ]]; then
  printf '\nNo runs cancelled.\n'
  exit 0
fi

failed=0
cancelled=0
printf '=== REQUESTING CANCELLATION ===\n'
for index in "${!selected[@]}"; do
  if gh run cancel "${run_ids[index]}"; then
    cancelled=$((cancelled + 1))
  else
    printf 'Error: failed to cancel run %s.\n' "${run_ids[index]}" >&2
    failed=$((failed + 1))
  fi
done
printf '=== COMPLETE: %s cancellation requests accepted; %s failed ===\n' "$cancelled" "$failed"
[[ "$failed" -eq 0 ]]
