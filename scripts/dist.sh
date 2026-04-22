#!/bin/bash

# 1. Install cargo-dist if not present
if ! command -v cargo-dist &> /dev/null; then
    echo "Installing cargo-dist..."
    curl --proto '=https' --tlsv1.2 -LsSf https://github.com/axodotdev/cargo-dist/releases/latest/download/cargo-dist-installer.sh | sh
fi

# 2. Navigate to the repo root that contains Cargo.toml
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

# 3. Initialize cargo-dist 
# The --yes flag uses defaults to generate files without interactive prompts.
cargo dist init --yes

# 4. Explicitly generate/update the CI scripts
# This ensures the .github/workflows/release.yml is created/refreshed.
cargo dist generate-ci

# 5. Plan the release to verify the generated manifest
# This creates a local preview of what the release artifacts will look like.
cargo dist plan

echo "---"
echo "Files generated/updated by cargo-dist:"
ls -R .github/workflows/release.yml Cargo.toml dist-workspace.toml
