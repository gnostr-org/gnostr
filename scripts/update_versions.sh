#!/bin/bash

if [ -z "$CARGO_REGISTRY_TOKEN" ]; then
    echo "Error: CARGO_REGISTRY_TOKEN is not set."
    echo "Please set the CARGO_REGISTRY_TOKEN environment variable before running this script."
    echo "You can get one from https://crates.io/settings/tokens"
    exit 1
fi

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
find . -type f -name "Cargo.toml" ! -path "./Cargo.toml" ! -path "*/target/*" ! -path "*/vendor/*" | while read -r file; do
    echo "Updating $file..."
    # Use sed to replace the version line
    # -i ensures in-place editing
    # The regex matches a line starting with 'version =' and replaces it with the new version
    sed -i '' "s/^version = \".*\"/$CURRENT_VERSION/" "$file"
    echo "Updated $file"
done


echo "All Cargo.toml files updated."

# Update versions for local path dependencies
find . -type f -name "Cargo.toml" ! -path "./Cargo.toml" ! -path "*/target/*" ! -path "*/vendor/*" | while read -r crate_file; do
    CRATE_DIR=$(dirname "$crate_file")
    echo "Checking local dependencies in $crate_file..."

    # Find local path dependencies in the current Cargo.toml
    grep -E '\w+ = { version = ".*", path = ".*" }' "$crate_file" | while read -r dep_line; do
        DEP_NAME=$(echo "$dep_line" | awk -F' = ' '{print $1}')
        DEP_PATH_RELATIVE=$(echo "$dep_line" | grep -oP 'path = "\K[^"]+')

        if [ -n "$DEP_PATH_RELATIVE" ] && [ -n "$DEP_NAME" ]; then
            # Resolve absolute path for the dependency's Cargo.toml
            DEP_CARGO_TOML="$(realpath --relative-to="$(pwd)" "$CRATE_DIR/$DEP_PATH_RELATIVE/Cargo.toml")"
            
            if [ -f "$DEP_CARGO_TOML" ]; then
                DEP_CURRENT_VERSION=$(grep '^version =' "$DEP_CARGO_TOML" | head -1 | awk -F'"' '{print $2}')
                
                if [ -n "$DEP_CURRENT_VERSION" ]; then
                    echo "  - Found local dependency $DEP_NAME (path: $DEP_PATH_RELATIVE). Its version is $DEP_CURRENT_VERSION."
                    # Update the version in the current crate_file
                    sed -i '' "s/^$DEP_NAME = { version = \".*\", path = \"$DEP_PATH_RELATIVE\"}/$DEP_NAME = { version = \"$DEP_CURRENT_VERSION\", path = \"$DEP_PATH_RELATIVE\"}/" "$crate_file"
                    echo "    Updated $DEP_NAME version in $crate_file"
                else
                    echo "    Warning: Could not find version in $DEP_CARGO_TOML for dependency $DEP_NAME."
                fi
            else
                echo "    Warning: Dependency Cargo.toml not found at $DEP_CARGO_TOML for $DEP_NAME."
            fi
        fi
    done
done

echo "All local path dependencies updated."

#publishing order

export CARGO_REGISTRY_TOKEN=$CARGO_REGISTRY_TOKEN

COMMAND='cargo sort'

sleep 1 && pushd grammar && $COMMAND || true && popd
sleep 1 && pushd filetreelist && $COMMAND || true && popd
sleep 1 && pushd scopetime && $COMMAND || true && popd
sleep 1 && pushd crawler && $COMMAND || true && popd
sleep 1 && pushd query && $COMMAND || true && popd
sleep 1 && pushd git2-hooks && $COMMAND || true && popd
sleep 1 && pushd invalidstring && $COMMAND || true && popd
sleep 1 && pushd asyncgit && $COMMAND || true && popd
sleep 1 && pushd legit && $COMMAND || true && popd
sleep 1 && pushd qr && $COMMAND || true && popd
sleep 1 && pushd relay && $COMMAND || true && popd

COMMAND='cargo publish -j8'

sleep 1 && pushd grammar && $COMMAND || true && popd
sleep 1 && pushd filetreelist && $COMMAND || true && popd
sleep 1 && pushd scopetime && $COMMAND || true && popd
sleep 1 && pushd crawler && $COMMAND || true && popd
sleep 1 && pushd query && $COMMAND || true && popd
sleep 1 && pushd git2-hooks && $COMMAND || true && popd
sleep 1 && pushd invalidstring && $COMMAND || true && popd
sleep 1 && pushd asyncgit && $COMMAND || true && popd
sleep 1 && pushd legit && $COMMAND || true && popd
sleep 1 && pushd qr && $COMMAND || true && popd
sleep 1 && pushd relay && $COMMAND || true && popd
$COMMAND --no-verify


