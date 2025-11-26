#!/bin/bash

# Get the version from the root Cargo.toml
ROOT_CARGO_TOML="./Cargo.toml"
if [ ! -f "$ROOT_CARGO_TOML" ]; then
    echo "Error: $ROOT_CARGO_TOML not found."
    exit 1
fi

CURRENT_VERSION=$(grep '^version =' "$ROOT_CARGO_TOML" | head -1)

if [ -z "$CURRENT_VERSION" ]; then
    echo "Error: Could not find version in $ROOT_CARGO_TOML."
    exit 1
fi

echo "Root version found: $CURRENT_VERSION"

# Find all other Cargo.toml files and update them
find . -type f -name "Cargo.toml" ! -path "./Cargo.toml" | while read -r file; do
    echo "Updating $file..."
    # Use sed to replace the version line
    # -i ensures in-place editing
    # The regex matches a line starting with 'version =' and replaces it with the new version
    sed -i '' "s/^version = \".*\"/$CURRENT_VERSION/" "$file"
    echo "Updated $file"
done

echo "All Cargo.toml files updated."
