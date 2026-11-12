#!/usr/bin/env bash
set -euo pipefail

BASH_VERSION_CURRENT="${BASH_VERSION:-unknown}"
BASH_MAJOR="${BASH_VERSINFO[0]:-0}"
BASH_MINOR="${BASH_VERSINFO[1]:-0}"
if ! bash -n "${BASH_SOURCE[0]}"; then
  exit 1
fi

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

SINCE=""
UNTIL=""
AUTHOR=""
ALL=false
NO_MERGES=false
JSON=false
FILES=false
MAX_FILES=10

usage() {
  cat <<'EOF'
Usage: git-repo-stats.sh [--today] [--since VALUE] [--until VALUE] [--author VALUE] [--all] [--no-merges] [--files] [--max-files N] [--json]

Options:
  --today            Use the start of today as --since
  --since VALUE      Git date expression for the lower bound
  --until VALUE      Git date expression for the upper bound
  --author VALUE     Filter commits by author
  --all              Ignore date bounds and scan all commits
  --no-merges        Exclude merge commits
  --files            Print touched-file counts
  --max-files N      Limit touched-file output to N entries
  --json             Emit a JSON summary
  --help             Show this help
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --today)
      SINCE="today 00:00"
      ;;
    --since)
      shift
      [[ $# -gt 0 ]] || { echo "--since requires a value" >&2; exit 1; }
      SINCE="$1"
      ;;
    --since=*)
      SINCE="${1#*=}"
      ;;
    --until)
      shift
      [[ $# -gt 0 ]] || { echo "--until requires a value" >&2; exit 1; }
      UNTIL="$1"
      ;;
    --until=*)
      UNTIL="${1#*=}"
      ;;
    --author)
      shift
      [[ $# -gt 0 ]] || { echo "--author requires a value" >&2; exit 1; }
      AUTHOR="$1"
      ;;
    --author=*)
      AUTHOR="${1#*=}"
      ;;
    --all)
      ALL=true
      ;;
    --no-merges)
      NO_MERGES=true
      ;;
    --files)
      FILES=true
      ;;
    --max-files)
      shift
      [[ $# -gt 0 ]] || { echo "--max-files requires a value" >&2; exit 1; }
      MAX_FILES="$1"
      ;;
    --max-files=*)
      MAX_FILES="${1#*=}"
      ;;
    --json)
      JSON=true
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

log_args=(log --pretty=tformat: --numstat)
rev_args=(log --format=%H)

if [[ "$ALL" != true ]]; then
  if [[ -n "$SINCE" ]]; then
    log_args+=(--since="$SINCE")
  else
    log_args+=(--since="today 00:00")
  fi
  if [[ -n "$UNTIL" ]]; then
    log_args+=(--until="$UNTIL")
  fi
fi

if [[ -n "$AUTHOR" ]]; then
  log_args+=(--author="$AUTHOR")
  rev_args+=(--author="$AUTHOR")
fi

if [[ "$NO_MERGES" == true ]]; then
  log_args+=(--no-merges)
  rev_args+=(--no-merges)
fi

if [[ "$ALL" != true ]]; then
  if [[ -n "$SINCE" ]]; then
    rev_args+=(--since="$SINCE")
  else
    rev_args+=(--since="today 00:00")
  fi
  if [[ -n "$UNTIL" ]]; then
    rev_args+=(--until="$UNTIL")
  fi
fi

commits="$(git "${rev_args[@]}" | wc -l | tr -d ' ')"
files_changed=0
insertions=0
deletions=0
unique_files_tmp="${TMPDIR:-/tmp}/repo-stats-files.$$"
trap 'rm -f "$unique_files_tmp"' EXIT
: >"$unique_files_tmp"

while IFS= read -r line; do
  if [[ -z "$line" ]]; then
    continue
  fi
  case "$line" in
    $'\t'*)
      file_path="${line#*$'\t'}"
      printf '%s\n' "$file_path" >>"$unique_files_tmp"
      files_changed=$((files_changed + 1))
      ;;
    *)
      read -r added removed path <<<"$line" || true
      if [[ "$added" != "-" && -n "${added:-}" ]]; then
        insertions=$((insertions + added))
      fi
      if [[ "$removed" != "-" && -n "${removed:-}" ]]; then
        deletions=$((deletions + removed))
      fi
      ;;
  esac
done < <(git "${log_args[@]}")

net=$((insertions - deletions))
touch_count="$(sort "$unique_files_tmp" | uniq | wc -l | tr -d ' ')"

repo_size="$(git count-objects -vH | awk -F': ' '
  /^count:/ { count=$2 }
  /^size:/ { size=$2 }
  /^in-pack:/ { inpack=$2 }
  /^size-pack:/ { sizepack=$2 }
  END {
    printf "count=%s size=%s in-pack=%s size-pack=%s", count, size, inpack, sizepack
  }')"

if [[ "$JSON" == true ]]; then
  printf '{'
  printf '"commits":%s,' "$commits"
  printf '"files_changed":%s,' "$files_changed"
  printf '"unique_files":%s,' "$touch_count"
  printf '"insertions":%s,' "$insertions"
  printf '"deletions":%s,' "$deletions"
  printf '"net":%s,' "$net"
  printf '"repo_size":"%s"' "$repo_size"
  printf '}\n'
  exit 0
fi

printf 'commits: %s\n' "$commits"
printf 'files changed: %s\n' "$files_changed"
printf 'unique files: %s\n' "$touch_count"
printf 'insertions: %s\n' "$insertions"
printf 'deletions: %s\n' "$deletions"
printf 'net: %s\n' "$net"
printf 'repo size: %s\n' "$repo_size"

if [[ "$FILES" == true ]]; then
  printf '\nmost touched files:\n'
  sort "$unique_files_tmp" | uniq -c | sort -nr | head -n "$MAX_FILES"
fi
