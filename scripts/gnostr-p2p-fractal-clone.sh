#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# shellcheck source=scripts/lib/script-ui.sh
source "$ROOT_DIR/scripts/lib/script-ui.sh"
INTERACTIVE=false
REPLACE=false
SOURCE="$ROOT_DIR/web/crlf.git"
OUTPUT="$ROOT_DIR/gnostr_web_crlf"
CLONE="$ROOT_DIR/gnostr_web_crlf_repo"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --interactive|-i) INTERACTIVE=true ;;
    --replace) REPLACE=true ;;
    --source|--output|--clone)
      [[ $# -ge 2 ]] || { echo "$1 requires a value" >&2; exit 1; }
      case "$1" in
        --source) SOURCE="$2" ;;
        --output) OUTPUT="$2" ;;
        --clone) CLONE="$2" ;;
      esac
      shift
      ;;
    --help|-h)
      echo "Usage: gnostr-p2p-fractal-clone.sh [--interactive] [--source PATH] [--output PATH] [--clone PATH] [--replace]"
      echo "Existing output directories require --replace or interactive confirmation."
      exit 0
      ;;
    *) echo "Unknown option: $1" >&2; exit 1 ;;
  esac
  shift
done
if [[ "$INTERACTIVE" == true ]]; then
  ui_prompt "Source repository" "$SOURCE"; SOURCE="$UI_VALUE"
  ui_prompt "Fractal output directory" "$OUTPUT"; OUTPUT="$UI_VALUE"
  ui_prompt "Git clone directory" "$CLONE"; CLONE="$UI_VALUE"
fi
for path in "$OUTPUT" "$CLONE"; do
  if [[ -e "$path" || -L "$path" ]]; then
    echo "Error: output already exists: $path" >&2
    if [[ "$INTERACTIVE" != true && "$REPLACE" != true ]]; then
      echo "Use a new directory, --interactive, or --replace." >&2
      exit 1
    fi
    if [[ -L "$path" || ! -d "$path" ]]; then
      echo "Error: refusing to replace a symlink or non-directory: $path" >&2
      exit 1
    fi
    resolved="$(cd "$path" && pwd -P)"
    # Only replace a direct child of this repository, never a parent or source.
    if [[ "$(dirname "$resolved")" != "$ROOT_DIR" ||
      "$resolved" == "$(cd "$SOURCE" && pwd -P)" ]]; then
      echo "Error: refusing replacement outside repository output directories: $path" >&2
      exit 1
    fi
  fi
done
if [[ "$OUTPUT" == "$CLONE" ]]; then
  echo "Error: output and clone directories must differ." >&2
  exit 1
fi
if [[ "$INTERACTIVE" == true ]]; then
  ui_confirm "Create $OUTPUT and $CLONE from $SOURCE, replacing existing output directories?"
fi
for path in "$OUTPUT" "$CLONE"; do
  if [[ -d "$path" ]]; then
    rm -r -- "$path"
  fi
done
cd "$ROOT_DIR"
cargo run -p gnostr-p2p --example fractal_swarm_perfect_ip -- --recursive "$SOURCE" --logging --out "$OUTPUT"
git clone "$OUTPUT" "$CLONE"
