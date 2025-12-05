#!/usr/bin/env bash

export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/aarch64-apple-darwin/bin:$PATH"

set -e


# This script automates the process of generating code coverage reports for the Rust project.

# 1. Install necessary dependencies
echo "Installing llvm-tools-preview..."
rustup component add llvm-tools-preview

# 2. Clean previous build artifacts
if [ "$1" == "clean" ]; then
    cargo clean
fi
# 3. Build the project with coverage instrumentation
echo "Building with coverage instrumentation..."
RUSTFLAGS="-C instrument-coverage" cargo build

# 4. Run tests to generate coverage data
echo "Running tests..."
RUSTFLAGS="-C instrument-coverage" cargo test

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
find . -name "*.profraw" | xargs llvm-profdata merge -sparse -o default.profdata

# Use llvm-cov to generate the report
llvm-cov show $BINARY_PATH \
    --instr-profile=default.profdata \
    --format=html \
    --output-dir=coverage \
    --show-line-counts-or-regions \
    --show-instantiations \
    --show-regions \
    --Xdemangler=rustfilt

# Also show a summary in the terminal
llvm-cov report $BINARY_PATH --instr-profile=default.profdata

echo "Coverage report generated in the 'coverage' directory."
echo "Open coverage/index.html to view the report."
