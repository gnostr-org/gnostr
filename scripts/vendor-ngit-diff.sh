#!/usr/bin/env bash
set -euo pipefail

if ! bash -n "${BASH_SOURCE[0]}"; then
  exit 1
fi

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

LEFT_DIR="./ngit"
RIGHT_DIR="./vendor/ngit-cli"

if [[ ! -d "$LEFT_DIR" ]]; then
  echo "Missing directory: $LEFT_DIR" >&2
  exit 1
fi

if [[ ! -d "$RIGHT_DIR" ]]; then
  echo "Missing directory: $RIGHT_DIR" >&2
  exit 1
fi

set +e
git diff "$@" --no-index -- "$LEFT_DIR" "$RIGHT_DIR"
status=$?
set -e

if [[ $status -ne 0 && $status -ne 1 ]]; then
  exit "$status"
fi

exit 0
