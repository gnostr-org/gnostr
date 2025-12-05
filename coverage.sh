#!/usr/bin/env bash

## TODO make less brittle and cross platform
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/aarch64-apple-darwin/bin:$PATH"

set -e


# This script automates the process of generating code coverage reports for the Rust project.

# 1. Install necessary dependencies
echo "Installing llvm-tools-preview..."
rustup component add llvm-tools-preview

echo "Installing rustfilt..."
cargo install rustfilt || true

# 2. Clean previous build artifacts
if [ "$1" == "clean" ]; then
    cargo clean
fi
# 3. Build the project with coverage instrumentation
echo "Building with coverage instrumentation..."
RUSTFLAGS="-C instrument-coverage" cargo build --all-features

# 4. Run tests to generate coverage data
echo "Running tests..."
RUSTFLAGS="-C instrument-coverage" cargo test --all-features -- --nocapture
## RUSTFLAGS="-C instrument-coverage" cargo test --skip "push::tests::integration_tests::" --all-features -- --nocapture


# 5. Process coverage data and generate report
echo "Generating coverage report..."


# Find the project name from Cargo.toml
PROJECT_NAME=$(grep -m 1 '^name = ' Cargo.toml | sed 's/name = "\(.*\)"/\1/')
# Replace hyphens with underscores for the binary name
BINARY_NAME=${PROJECT_NAME//-/_}


# Create a list of all binaries to include in the report
OBJECTS=""
for file in target/debug/deps/${BINARY_NAME}-*; do
    if [[ -x "$file" && ! -d "$file" ]]; then
        OBJECTS="$OBJECTS -object $file"
    fi
done

if [ -z "$OBJECTS" ]; then
  echo "Error: No binaries found for coverage analysis."
  exit 1
fi



# Use llvm-profdata to merge the raw profile data
PROFRAW_DIR="./coverage_tmp"
mkdir -p $PROFRAW_DIR
find . -maxdepth 1 -name "*.profraw" -exec mv {} $PROFRAW_DIR/ \;
llvm-profdata merge -sparse -o $PROFRAW_DIR/default.profdata $PROFRAW_DIR/*.profraw

# Use llvm-cov to generate the report
BINARY_PATH=target/debug/$BINARY_NAME
llvm-cov show $OBJECTS \
    --instr-profile=$PROFRAW_DIR/default.profdata \
    --format=html \
    --output-dir=./coverage \
    --show-line-counts-or-regions \
    --show-instantiations \
    --show-regions \
    --Xdemangler=rustfilt

# Also show a summary in the terminal
llvm-cov report $OBJECTS --instr-profile=$PROFRAW_DIR/default.profdata

echo "Coverage report generated in the 'coverage' directory."
echo "Open coverage/index.html to view the report."

# Clean up temporary profile data directory
rm -rf $PROFRAW_DIR
