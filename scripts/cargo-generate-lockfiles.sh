#!/bin/bash
set -euo pipefail
BASH_VERSION_CURRENT="${BASH_VERSION:-unknown}"
BASH_MAJOR="${BASH_VERSINFO[0]:-0}"
BASH_MINOR="${BASH_VERSINFO[1]:-0}"
if ! bash -n "${BASH_SOURCE[0]}"; then
  exit 1
fi

# ---
# Description: Recursively find Rust projects and generate/update Cargo.lock files.
# Usage: ./generate_locks.sh [target_directory]
# ---

# Set the target directory to the first argument, or current directory if empty
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/lib/script-ui.sh
source "$SCRIPT_DIR/lib/script-ui.sh"
TARGET_DIR="."
INTERACTIVE=false
while [[ $# -gt 0 ]]; do
  case "$1" in
    --interactive|-i) INTERACTIVE=true ;;
    --help|-h)
      echo "Usage: cargo-generate-lockfiles.sh [--interactive] [target_directory]"
      echo "Interactive mode selects manifests and confirms lockfile updates."
      exit 0
      ;;
    -*) echo "Unknown option: $1" >&2; exit 1 ;;
    *) TARGET_DIR="$1" ;;
  esac
  shift
done

echo "Searching for Rust projects in: $(realpath "$TARGET_DIR")"
echo "----------------------------------------------------"

# Find all Cargo.toml files, excluding target directories (to save time)
manifest_list="$(find "$TARGET_DIR" -name "target" -prune -o -name "Cargo.toml" -print)"
manifests=()
while IFS= read -r manifest; do
    [[ -z "$manifest" ]] || manifests+=("$manifest")
done <<<"$manifest_list"
if [[ "$INTERACTIVE" == true ]]; then
    ui_select "Manifests to sort and generate lockfiles for" ${manifests[@]+"${manifests[@]}"}
    manifests=("${UI_SELECTED[@]}")
    ui_confirm "Update manifests and lockfiles for these ${#manifests[@]} projects?"
fi
for manifest in ${manifests[@]+"${manifests[@]}"}; do
    # Get the directory containing the Cargo.toml
    project_dir=$(dirname "$manifest")
    
    echo "Processing project in: $project_dir"
    
    # Navigate to the project directory and run cargo
    # Using subshell ( ) to ensure we return to the original path automatically
    (
        cd "$project_dir"
        bash "$SCRIPT_DIR/with-system-rocksdb.sh" cargo sort
        if bash "$SCRIPT_DIR/with-system-rocksdb.sh" cargo generate-lockfile; then
            echo "Successfully generated lockfile for $(basename "$project_dir")"
        else
            echo "Error: Failed to generate lockfile in $project_dir" >&2
            exit 1
        fi
    )
    
    echo "----------------------------------------------------"
done

echo "Recursive lockfile generation complete."
git diff || true
