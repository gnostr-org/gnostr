#!/usr/bin/env bash
set -euo pipefail

if ! bash -n "${BASH_SOURCE[0]}"; then
  exit 1
fi

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

REMOTE="origin"
DELETE=false
INCLUDE_LOCAL_BRANCHES=false
KNOWN_COMMITS_ONLY=true
COMMITS=()

KNOWN_BIG_COMMITS=(
  "57222ce14d61801b078af7243965bcec79ddd03c"
  "a056c6ab8b6def25fabd5b27c42638a7e8ee5558"
  "ddbca317c74d926af6ab07eb3fa1afae712e9f1a"
)

usage() {
  cat <<'EOF'
Usage: remove-big-commits.sh [options]

List refs that still contain known large-object commits, and optionally delete them.

By default this script scans for these known commits:
  57222ce14d61801b078af7243965bcec79ddd03c
  a056c6ab8b6def25fabd5b27c42638a7e8ee5558
  ddbca317c74d926af6ab07eb3fa1afae712e9f1a

Options:
  --commit SHA          Add one commit to scan for (repeatable)
  --local-branches      Include local branches in the scan
  --remote NAME         Remote to inspect/delete on (default: origin)
  --delete              Delete matching refs instead of only listing them
  --help                Show this help

Examples:
  ./scripts/remove-big-commits.sh
  ./scripts/remove-big-commits.sh --commit 57222ce14d61801b078af7243965bcec79ddd03c
  ./scripts/remove-big-commits.sh --delete
  ./scripts/remove-big-commits.sh --delete --local-branches
EOF
}

add_commit() {
  local commit="$1"
  local existing

  for existing in "${COMMITS[@]:-}"; do
    if [[ "$existing" == "$commit" ]]; then
      return 0
    fi
  done

  COMMITS+=("$commit")
}

validate_commit() {
  local commit="$1"

  if ! git rev-parse -q --verify "${commit}^{commit}" >/dev/null; then
    echo "Unknown commit: $commit" >&2
    exit 1
  fi
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --commit)
      shift
      [[ $# -gt 0 ]] || { echo "--commit requires a value" >&2; exit 1; }
      validate_commit "$1"
      add_commit "$1"
      KNOWN_COMMITS_ONLY=false
      ;;
    --local-branches)
      INCLUDE_LOCAL_BRANCHES=true
      ;;
    --remote)
      shift
      [[ $# -gt 0 ]] || { echo "--remote requires a value" >&2; exit 1; }
      REMOTE="$1"
      ;;
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

if [[ "$KNOWN_COMMITS_ONLY" == true ]]; then
  for commit in "${KNOWN_BIG_COMMITS[@]}"; do
    if git rev-parse -q --verify "${commit}^{commit}" >/dev/null; then
      add_commit "$commit"
    fi
  done
fi

if [[ ${#COMMITS[@]} -eq 0 ]]; then
  echo "No commits to inspect." >&2
  exit 1
fi

tmpdir="$(mktemp -d "${TMPDIR:-/tmp}/remove-big-commits.XXXXXX")"
matches_file="$tmpdir/matches.tsv"

cleanup() {
  rm -rf "$tmpdir"
}
trap cleanup EXIT

commit_subject() {
  git --no-pager show --no-patch --format='%ad %s' --date=iso-strict "$1"
}

for commit in "${COMMITS[@]}"; do
  while IFS= read -r ref; do
    [[ -n "$ref" ]] || continue
    case "$ref" in
      */HEAD)
        continue
        ;;
    esac
    printf '%s\t%s\n' "$commit" "$ref" >> "$matches_file"
  done < <(
    if [[ "$INCLUDE_LOCAL_BRANCHES" == true ]]; then
      git for-each-ref --format='%(refname)' --contains "$commit" "refs/tags" "refs/remotes/${REMOTE}" "refs/heads"
    else
      git for-each-ref --format='%(refname)' --contains "$commit" "refs/tags" "refs/remotes/${REMOTE}"
    fi
  )
done

if [[ ! -f "$matches_file" ]]; then
  echo "No matching refs found."
  exit 0
fi

echo "Matching refs:"
for commit in "${COMMITS[@]}"; do
  if ! grep -q "^${commit}"$'\t' "$matches_file"; then
    continue
  fi

  printf '\n%s %s\n' "$commit" "$(commit_subject "$commit")"
  grep "^${commit}"$'\t' "$matches_file" | cut -f2 | sort -u | while IFS= read -r ref; do
    printf '  %s\n' "$ref"
  done
done

if [[ "$DELETE" != true ]]; then
  printf '\nRun again with --delete to remove these refs.\n'
  exit 0
fi

current_branch="$(git symbolic-ref --quiet --short HEAD 2>/dev/null || true)"
remote_exists=false
if git remote get-url "$REMOTE" >/dev/null 2>&1; then
  remote_exists=true
fi

delete_ref() {
  local ref="$1"
  local name

  case "$ref" in
    refs/tags/*)
      name="${ref#refs/tags/}"
      if git rev-parse -q --verify "$ref" >/dev/null 2>&1; then
        git tag -d "$name"
      fi
      if [[ "$remote_exists" == true ]] && git ls-remote --tags "$REMOTE" "refs/tags/$name" | grep -q .; then
        git push "$REMOTE" ":refs/tags/$name"
      fi
      ;;
    refs/remotes/"$REMOTE"/*)
      name="${ref#refs/remotes/$REMOTE/}"
      if [[ "$remote_exists" == true ]] && git ls-remote --heads "$REMOTE" "refs/heads/$name" | grep -q .; then
        git push "$REMOTE" --delete "$name"
      fi
      if git rev-parse -q --verify "$ref" >/dev/null 2>&1; then
        git update-ref -d "$ref"
      fi
      ;;
    refs/heads/*)
      name="${ref#refs/heads/}"
      if [[ "$name" == "$current_branch" ]]; then
        echo "Refusing to delete checked out branch: $name" >&2
        exit 1
      fi
      git branch -D "$name"
      ;;
    *)
      echo "Skipping unsupported ref: $ref" >&2
      ;;
  esac
}

printf '\nDeleting matching refs...\n'
cut -f2 "$matches_file" | sort -u | while IFS= read -r ref; do
  [[ -n "$ref" ]] || continue
  printf 'Deleting %s\n' "$ref"
  delete_ref "$ref"
done

printf '\nDone.\n'
