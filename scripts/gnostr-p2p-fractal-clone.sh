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
SOURCE="$(cd "$SOURCE" && pwd -P)"
resolve_output() {
  local path="$1"
  if [[ -L "$path" ]]; then
    echo "Error: refusing a symlinked output: $path" >&2
    return 1
  fi
  if [[ -d "$path" ]]; then
    (cd "$path" && pwd -P)
  else
    (cd "$(dirname "$path")" && printf '%s/%s\n' "$PWD" "$(basename "$path")")
  fi
}
OUTPUT="$(resolve_output "$OUTPUT")"
CLONE="$(resolve_output "$CLONE")"
if [[ "$OUTPUT" == "$CLONE" || "$OUTPUT/" == "$CLONE/"* || "$CLONE/" == "$OUTPUT/"* ]]; then
  echo "Error: output and clone directories must not overlap." >&2
  exit 1
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
    if [[ "$SOURCE/" == "$path/"* || "$ROOT_DIR/" == "$path/"* ||
      "$HOME/" == "$path/"* ]]; then
      echo "Error: refusing to replace a source or parent directory: $path" >&2
      exit 1
    fi
    case "$path" in
      "$ROOT_DIR/gnostr_web_crlf"|"$ROOT_DIR/gnostr_web_crlf_repo") ;;
      *)
        if [[ ! -f "$path/.gnostr-fractal-output" ||
          -L "$path/.gnostr-fractal-output" ||
          "$(cat "$path/.gnostr-fractal-output")" != gnostr-p2p-fractal-clone ]]; then
          echo "Error: refusing to replace an unrecognized output directory: $path" >&2
          exit 1
        fi
        ;;
    esac
  fi
done
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
printf '%s\n' gnostr-p2p-fractal-clone > "$OUTPUT/.gnostr-fractal-output"
git clone "$OUTPUT" "$CLONE"
printf '%s\n' gnostr-p2p-fractal-clone > "$CLONE/.gnostr-fractal-output"
