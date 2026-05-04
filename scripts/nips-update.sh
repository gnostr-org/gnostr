#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
NIPS_DIR="${NIPS_DIR:-$REPO_ROOT/nips}"
UPSTREAM_URL="${UPSTREAM_URL:-https://github.com/nostr-protocol/nips.git}"

if ! command -v git >/dev/null 2>&1; then
  echo "Error: git is required." >&2
  exit 1
fi

if [ ! -d "$NIPS_DIR" ]; then
  mkdir -p "$NIPS_DIR"
fi

TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/nips-update.XXXXXX")"
cleanup() {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

shopt -s nullglob

mkdir -p "$TMP_DIR/git-template"
GIT_TEMPLATE_DIR="$TMP_DIR/git-template" git clone --quiet --depth 1 "$UPSTREAM_URL" "$TMP_DIR/nips"

for upstream_file in "$TMP_DIR"/nips/*.md; do
  cp "$upstream_file" "$NIPS_DIR/"
done

for local_file in "$NIPS_DIR"/*.md; do
  base_name="${local_file##*/}"
  if [ ! -f "$TMP_DIR/nips/$base_name" ]; then
    rm -f "$local_file"
  fi
done

echo "Updated $NIPS_DIR from $UPSTREAM_URL"
