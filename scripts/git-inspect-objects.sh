#!/usr/bin/env bash
set -euo pipefail

if ! bash -n "${BASH_SOURCE[0]}"; then
  exit 1
fi

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

DELETE=false

usage() {
  cat <<'EOF'
Usage: git-inspect-objects.sh [--delete]

Options:
  --delete  Delete refs that point at tag objects
  --help    Show this help
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --delete)
      DELETE=true
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      echo "Unsupported argument: $1" >&2
      exit 1
      ;;
  esac
  shift
done

mapfile -t bad_refs < <(
  git for-each-ref --format='%(refname) %(objecttype)' refs/tags refs/remotes/origin \
    | awk '$2 == "tag" { print $1 }'
)

if [[ ${#bad_refs[@]} -eq 0 ]]; then
  echo "No refs point at tag objects."
  exit 0
fi

printf '%s\n' "Refs pointing at tag objects:"
printf '%s\n' "${bad_refs[@]}"

if [[ "$DELETE" == true ]]; then
  for ref in "${bad_refs[@]}"; do
    git update-ref -d "$ref"
  done
  echo "Deleted ${#bad_refs[@]} refs."
fi
