#!/bin/bash
set -euo pipefail
BASH_VERSION_CURRENT="${BASH_VERSION:-unknown}"
BASH_MAJOR="${BASH_VERSINFO[0]:-0}"
BASH_MINOR="${BASH_VERSINFO[1]:-0}"
if ! bash -n "${BASH_SOURCE[0]}"; then
  exit 1
fi

# Configuration with Argument Support
# Usage: ./yank_crate.sh <CRATE_NAME> <VERSION> <CREDENTIALS_FILE>
INTERACTIVE=false
POSITIONAL=()
for arg in "$@"; do
    case "$arg" in
        --interactive|-i) INTERACTIVE=true ;;
        --help|-h)
            echo "Usage: cargo-yank.sh [--interactive] [crate] [version] [credentials_file]"
            echo "Interactive mode prompts for crate/version and confirms the yank."
            exit 0
            ;;
        *) POSITIONAL+=("$arg") ;;
    esac
done
CRATE_NAME="${POSITIONAL[0]:-gnostr-dummy-crate}"
VERSION="${POSITIONAL[1]:-0.0.0}"
CREDENTIALS_FILE="${POSITIONAL[2]:-$HOME/.cargo/credentials.toml}"
if [[ "$INTERACTIVE" == true ]]; then
    # shellcheck source=scripts/lib/script-ui.sh
    source "$(dirname "${BASH_SOURCE[0]}")/lib/script-ui.sh"
    ui_prompt "Crate to yank" "$CRATE_NAME"
    CRATE_NAME="$UI_VALUE"
    ui_prompt "Version to yank" "$VERSION"
    VERSION="$UI_VALUE"
    ui_confirm "Yank $CRATE_NAME@$VERSION from crates.io?"
fi

echo "--- Pre-flight check: Detecting Cargo Authentication ---"

# 1. Check for Environment Variable (Highest Priority)
if [ -n "${CARGO_REGISTRY_TOKEN:-}" ]; then
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
