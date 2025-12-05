#!/usr/bin/env bash

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
PROJECT_NAME=$(grep -m 1 '^name = ' Cargo.toml | sed -E 's/name = "([^"]+)"/\\1/')
# Replace hyphens with underscores for the binary name
BINARY_NAME=${PROJECT_NAME//-/_}
# Replace hyphens with underscores for the binary name
BINARY_NAME=${PROJECT_NAME//-/_}


# The main binary for coverage analysis
BINARY_PATH="./target/debug/${PROJECT_NAME}"

# Ensure the binary exists
if [ ! -f "$BINARY_PATH" ]; then
    echo "Error: Could not find the main binary at ${BINARY_PATH}"
    echo "Searching for test binaries instead..."
    # If the main binary doesn't exist (e.g., for a library), use test binaries.
    # This is a fallback and might not cover all code if not all code is exercised by tests.
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
    BINARY_PATH=$OBJECTS
else
    # Create a list of all binaries to include in the report
    OBJECTS="-object ${BINARY_PATH}"
    for file in target/debug/deps/${BINARY_NAME}-*; do
        if [[ -x "$file" && ! -d "$file" ]]; then
            OBJECTS="$OBJECTS -object $file"
        fi
    done
    BINARY_PATH=$OBJECTS
fi


# Use llvm-profdata to merge the raw profile data
llvm-profdata merge -sparse default.profraw -o default.profdata

# Use llvm-cov to generate the report
llvm-cov show $BINARY_PATH \
    --instr-profile=default.profdata \
    --format=html \
    --output-dir=coverage \
    --show-line-counts-or-regions \
    --show-instantiations \
    --show-missing-regions \
    --Xdemangler=rustfilt

# Also show a summary in the terminal
llvm-cov report $BINARY_PATH --instr-profile=default.profdata

echo "Coverage report generated in the 'coverage' directory."
echo "Open coverage/index.html to view the report."
