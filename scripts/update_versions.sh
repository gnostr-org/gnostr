#!/bin/bash

# Function to escape a string for use in a sed regex pattern
escape_sed_regex() {
    printf "%s" "$1" | sed -e 's/[\[\]\\.^$*(){}+?|&]/\\&/g'
}

# Function to escape a string for use in a sed replacement string
escape_sed_replacement() {
    printf "%s" "$1" | sed -e 's/[\&/]/\\&/g'
}

# Function to ensure taplo-cli is installed
ensure_taplo_installed() {
    if ! command -v taplo &> /dev/null; then
        echo "taplo-cli not found. Installing it..."
        if cargo install taplo-cli; then
            echo "taplo-cli installed successfully."
        else
            echo "Error: Failed to install taplo-cli. Please install it manually using 'cargo install taplo-cli'."
            exit 1
        fi
    else
        echo "taplo-cli is already installed."
    fi
}

ensure_taplo_installed

SED_CMD="sed"

# Check if on macOS and if brew is installed
if [[ "$(uname -s)" == "Darwin" ]]; then
    if command -v brew &> /dev/null; then
        echo "Homebrew detected. Checking for gsed..."
        if ! command -v gsed &> /dev/null; then
            echo "gsed not found. Installing gnu-sed via Homebrew..."
            if brew install gnu-sed; then
                echo "gnu-sed installed successfully."
                SED_CMD="gsed"
            else
                echo "Warning: Failed to install gnu-sed. Falling back to default sed."
            fi
        else
            echo "gsed is already installed."
            SED_CMD="gsed"
        fi
    else
        echo "Homebrew not found on macOS. Falling back to default sed."
    fi
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
    taplo format "$file"
    echo "Updating $file..."
    # Use sed to replace the version line
    # -i ensures in-place editing
    # The regex matches a line starting with 'version =' and replaces it with the new version
    ESCAPED_CURRENT_VERSION_REPLACEMENT=$(escape_sed_replacement "$CURRENT_VERSION")
    ${SED_CMD} -i '' 's|^version = \".*\"|'"${ESCAPED_CURRENT_VERSION_REPLACEMENT}"''|' "$file"
    echo "Updated $file"
done


echo "All Cargo.toml files updated."

# Update versions for local path dependencies
find . -type f -name "Cargo.toml" ! -path "./Cargo.toml" ! -path "*/target/*" ! -path "*/vendor/*" | while read -r crate_file; do
    taplo format "$crate_file"
    CRATE_DIR=$(dirname "$crate_file")
    echo "Checking local dependencies in $crate_file..."

    # Find local path dependencies in the current Cargo.toml
    grep -E '\w+ = { version = ".*", path = ".*" }' "$crate_file" | while read -r dep_line; do
        DEP_NAME=$(echo "$dep_line" | awk -F' = ' '{print $1}')
        DEP_PATH_RELATIVE=$(echo "$dep_line" | awk -F'path = "' '{print $2}' | awk -F'"' '{print $1}')

        if [ -n "$DEP_PATH_RELATIVE" ] && [ -n "$DEP_NAME" ]; then
            # Resolve absolute path for the dependency's Cargo.toml
            # Construct absolute path for the dependency's Cargo.toml
            # This avoids using realpath --relative-to which is not portable
            DEP_CARGO_TOML="$CRATE_DIR/$DEP_PATH_RELATIVE/Cargo.toml"
            # Normalize the path to handle '..' etc.
            DEP_CARGO_TOML=$(cd $(dirname "$DEP_CARGO_TOML") && pwd)/$(basename "$DEP_CARGO_TOML")
            
            if [ -f "$DEP_CARGO_TOML" ]; then
                DEP_CURRENT_VERSION=$(grep '^version =' "$DEP_CARGO_TOML" | head -1 | awk -F'"' '{print $2}')
                
                if [ -n "$DEP_CURRENT_VERSION" ]; then
                    echo "  - Found local dependency $DEP_NAME (path: $DEP_PATH_RELATIVE). Its version is $DEP_CURRENT_VERSION."
                    ESCAPED_DEP_NAME=$(escape_sed_regex "$DEP_NAME")
                    ESCAPED_DEP_PATH_RELATIVE=$(escape_sed_regex "$DEP_PATH_RELATIVE")
                    # Update the version in the current crate_file
                    ${SED_CMD} -i '' 's|^${ESCAPED_DEP_NAME} = { version = \"[^\"]*\", path = \"${ESCAPED_DEP_PATH_RELATIVE}\"}|'"${ESCAPED_DEP_NAME} = { version = \"$(escape_sed_replacement "$DEP_CURRENT_VERSION")\", path = \"${ESCAPED_DEP_PATH_RELATIVE}\"}|'' "$crate_file"
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

echo "Synchronizing versions for gnostr-* dependencies across all Cargo.toml files..."

# Iterate through all Cargo.toml files, including the root one
find . -type f -name "Cargo.toml" ! -path "*/target/*" ! -path "*/vendor/*" | while read -r current_cargo_toml; do
    taplo format "$current_cargo_toml"
    echo "Processing dependencies in $current_cargo_toml..."

    # Find all gnostr-* dependencies in the current Cargo.toml
    grep -E '^\s*(gnostr-\w+|filetreelist)\s*=' "$current_cargo_toml" | while read -r dep_entry_line; do
        DEP_VAR_NAME=$(echo "$dep_entry_line" | awk -F'=' '{print $1}' | tr -d ' ' | tr -d '\t')

        # Map `filetreelist` to `gnostr-filetreelist` for consistency in lookup
        CRATE_ID_NAME="$DEP_VAR_NAME"
        if [ "$DEP_VAR_NAME" = "filetreelist" ]; then
            CRATE_ID_NAME="gnostr-filetreelist"
        fi

        # Determine the local crate's directory name (e.g., filetreelist from gnostr-filetreelist)
        CRATE_FOLDER_NAME=$(echo "$CRATE_ID_NAME" | sed 's/^gnostr-//')
        DEP_CARGO_TOML_PATH="./$CRATE_FOLDER_NAME/Cargo.toml"

        if [ -f "$DEP_CARGO_TOML_PATH" ]; then
            ACTUAL_DEP_VERSION=$(grep '^version =' "$DEP_CARGO_TOML_PATH" | head -1 | awk -F'"' '{print $2}')

            if [ -n "$ACTUAL_DEP_VERSION" ]; then
                ESCAPED_ACTUAL_DEP_VERSION_REPL=$(escape_sed_replacement "$ACTUAL_DEP_VERSION")
                ESCAPED_DEP_VAR_NAME=$(escape_sed_regex "$DEP_VAR_NAME")
                
                # Replace version for dependencies with path = "..."
                if [ "$SED_CMD" = "gsed" ]; then
                    ${SED_CMD} -i '' -E 's#^('${ESCAPED_DEP_VAR_NAME}' = { [^}]*version = "')[^""]*("", path = [^}]*})#\1'"${ESCAPED_ACTUAL_DEP_VERSION_REPL}"'\2#g' "$current_cargo_toml"
                else
                    ${SED_CMD} -i '' "s#^\\(${ESCAPED_DEP_VAR_NAME} = { [^}]*version = \\\"\)[^\\\"\"]*\\(\\\"\\\", path = [^}]*\\}\)#\\\\1${ESCAPED_ACTUAL_DEP_VERSION_REPL}\\\\2#" "$current_cargo_toml"
                fi
                # Replace version for direct dependencies (e.g., name = "^1.2.3")
                if [ "$SED_CMD" = "gsed" ]; then
                    ${SED_CMD} -i '' -E 's#^('${ESCAPED_DEP_VAR_NAME}' = ")[~^=]*("")#\1'"${ESCAPED_ACTUAL_DEP_VERSION_REPL}"'\2#g' "$current_cargo_toml"
                else
                    ${SED_CMD} -i '' "s#^\\(${ESCAPED_DEP_VAR_NAME} = \\\"[~^=]*\\\)[^\\\"\"]*\\(\\\"\\\"\\)#\\\\1${ESCAPED_ACTUAL_DEP_VERSION_REPL}\\\\2#" "$current_cargo_toml"
                fi
                
                echo "    Synchronized $CRATE_ID_NAME version in $current_cargo_toml to $ACTUAL_DEP_VERSION"
            else
                echo "    Warning: Could not extract actual version from $DEP_CARGO_TOML_PATH for $CRATE_ID_NAME. Skipping synchronization."
        fi
done

echo "All gnostr-* dependencies versions synchronized."

if [ -z "$CARGO_REGISTRY_TOKEN" ]; then

    echo "Error: CARGO_REGISTRY_TOKEN is not set."
    echo "Please set the CARGO_REGISTRY_TOKEN environment variable before running this script."
    echo "You can get one from https://crates.io/settings/tokens"
    exit 1
fi

#publishing order

export CARGO_REGISTRY_TOKEN=$CARGO_REGISTRY_TOKEN

COMMAND='cargo sort'

sleep 1 && pushd git2-hooks && $COMMAND || true && popd
sleep 1 && pushd grammar && $COMMAND || true && popd
sleep 1 && pushd filetreelist && $COMMAND || true && popd
sleep 1 && pushd scopetime && $COMMAND || true && popd
sleep 1 && pushd crawler && $COMMAND || true && popd
sleep 1 && pushd query && $COMMAND || true && popd
sleep 1 && pushd invalidstring && $COMMAND || true && popd
sleep 1 && pushd asyncgit && $COMMAND || true && popd
sleep 1 && pushd legit && $COMMAND || true && popd
sleep 1 && pushd qr && $COMMAND || true && popd
sleep 1 && pushd relay && $COMMAND || true && popd

COMMAND='cargo publish -j8'

sleep 1 && pushd invalidstring && $COMMAND || true && popd
sleep 1 && pushd git2-hooks && $COMMAND || true && popd
sleep 1 && pushd grammar && $COMMAND || true && popd
sleep 1 && pushd filetreelist && $COMMAND || true && popd
sleep 1 && pushd scopetime && $COMMAND || true && popd
sleep 1 && pushd asyncgit && $COMMAND || true && popd

sleep 1 && pushd crawler && $COMMAND || true && popd
sleep 1 && pushd query && $COMMAND || true && popd
sleep 1 && pushd legit && $COMMAND || true && popd
sleep 1 && pushd qr && $COMMAND || true && popd
sleep 1 && pushd relay && $COMMAND || true && popd
$COMMAND --no-verify


