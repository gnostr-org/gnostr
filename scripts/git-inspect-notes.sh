#!/usr/bin/env bash
set -euo pipefail

if ! bash -n "${BASH_SOURCE[0]}"; then
  exit 1
fi

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

usage() {
  cat <<'EOF'
Usage: git-inspect-notes.sh

Lists note refs and prints the note contents for each annotated object.
EOF
}

case "${1:-}" in
  -h|--help)
    usage
    exit 0
    ;;
  "")
    ;;
  *)
    echo "Unsupported argument: $1" >&2
    exit 1
    ;;
esac

notes_found=false

while IFS= read -r ref; do
  notes_found=true
  echo "== $ref =="
  while IFS=' ' read -r note_oid object_oid; do
    echo "-- object: $object_oid"
    git notes --ref="$ref" show "$object_oid"
    echo
  done < <(git notes --ref="$ref" list)
done < <(git for-each-ref --format='%(refname)' refs/notes)

if [[ "$notes_found" == false ]]; then
  echo "No git notes refs found."
fi
