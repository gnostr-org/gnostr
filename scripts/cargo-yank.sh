#!/bin/bash

# Configuration with Argument Support
# Usage: ./yank_crate.sh <CRATE_NAME> <VERSION> <CREDENTIALS_FILE>
CRATE_NAME="${1:-"gnostr-dummy-crate"}"
VERSION="${2:-"0.0.0"}"
CREDENTIALS_FILE="${3:-"$HOME/.cargo/credentials.toml"}"

echo "--- Pre-flight check: Detecting Cargo Authentication ---"

# 1. Check for Environment Variable (Highest Priority)
if [ -n "$CARGO_REGISTRY_TOKEN" ]; then
    echo "Status: Using CARGO_REGISTRY_TOKEN environment variable."
    AUTH_METHOD="env"

# 2. Check for credentials.toml
elif [ -f "$CREDENTIALS_FILE" ] && grep -q "token =" "$CREDENTIALS_FILE"; then
    echo "Status: Found token in $CREDENTIALS_FILE."
    AUTH_METHOD="credentials_file"

# 3. Check for active session
elif cargo login --list >/dev/null 2>&1; then
    echo "Status: Active cargo session detected."
    AUTH_METHOD="keychain"

else
    echo "Error: No Cargo API key detected."
    echo "Please run 'cargo login' or set 'export CARGO_REGISTRY_TOKEN=your_token'."
    exit 1
fi

echo "------------------------------------------------------"
echo "Target Crate: $CRATE_NAME"
echo "Version:      $VERSION"
echo "Auth Method:  $AUTH_METHOD"
echo "------------------------------------------------------"

# Execute yank
if cargo yank --vers "$VERSION" "$CRATE_NAME"; then
    echo "Successfully yanked $CRATE_NAME@$VERSION"
else
    echo "Error: Yank failed. Ensure you are an owner of '$CRATE_NAME' on crates.io."
    exit 1
fi
