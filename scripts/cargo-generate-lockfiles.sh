#!/bin/bash

# ---
# Description: Recursively find Rust projects and generate/update Cargo.lock files.
# Usage: ./generate_locks.sh [target_directory]
# ---

# Set the target directory to the first argument, or current directory if empty
TARGET_DIR="${1:-.}"

echo "Searching for Rust projects in: $(realpath "$TARGET_DIR")"
echo "----------------------------------------------------"

# Find all Cargo.toml files, excluding target directories (to save time)
find "$TARGET_DIR" -name "target" -prune -o -name "Cargo.toml" -print | while read -r manifest; do
    # Get the directory containing the Cargo.toml
    project_dir=$(dirname "$manifest")
    
    echo "Processing project in: $project_dir"
    
    # Navigate to the project directory and run cargo
    # Using subshell ( ) to ensure we return to the original path automatically
    (
        cd "$project_dir" || exit
        cargo sort;
        if cargo generate-lockfile; then
            echo "Successfully generated lockfile for $(basename "$project_dir")"
        else
            echo "Error: Failed to generate lockfile in $project_dir" >&2
        fi
    )
    
    echo "----------------------------------------------------"
done

echo "Recursive lockfile generation complete."
git diff || true
