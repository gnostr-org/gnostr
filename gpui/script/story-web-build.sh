#!/usr/bin/env bash

# 1. Safely find the git repository root directory
REPO_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
TARGET_DIR="${CARGO_TARGET_DIR:-$REPO_ROOT/target/wasm-build}"

echo CARGO_TARGET_DIR=$CARGO_TARGET_DIR
echo TARGET_DIR=$TARGET_DIR
# 2. Run the nightly Wasm target build
# Ensure cargo writes into TARGET_DIR so the script can find the .wasm output
CARGO_TARGET_DIR="$TARGET_DIR" cargo +nightly build \
  --manifest-path "$REPO_ROOT/gpui/crates/story-web/Cargo.toml" \
  --target wasm32-unknown-unknown \
  --verbose

# 3. Dynamic check for the generated .wasm file
echo "Checking for compiled artifacts in: $TARGET_DIR/wasm32-unknown-unknown/debug/"
if [ -d "$TARGET_DIR/wasm32-unknown-unknown/debug" ]; then
  ls -l "$TARGET_DIR"/wasm32-unknown-unknown/debug/*.wasm 2>/dev/null || \
  echo "Error: No .wasm files found in target directory. Verify crate matches 'lib' with 'cdylib' type."
else
  echo "Error: Target output directory does not exist."
  exit 1
fi

# List the expected wasm file to confirm presence
ls -l "$TARGET_DIR/wasm32-unknown-unknown/debug/gpui_component_story_web.wasm" || true
