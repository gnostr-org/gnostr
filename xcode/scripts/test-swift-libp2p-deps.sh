#!/bin/bash
# Test all local swift-libp2p-* and swift-gnostr-* (FFI) packages.
# Usage: ./scripts/test-swift-libp2p-deps.sh [--verbose] [--rust-ffi]

set -e

VERBOSE=0
RUST_FFI=0
for arg in "$@"; do
    case "$arg" in
        --verbose|-v)
            VERBOSE=1
            ;;
        --rust-ffi)
            RUST_FFI=1
            ;;
        *)
            echo "Unknown argument: $arg"
            echo "Usage: $0 [--verbose] [--rust-ffi]"
            exit 1
            ;;
    esac
done

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
RESULTS=()
FAILURES=()

SWIFT_TEST_ARGS=()
if [ "$VERBOSE" -eq 1 ]; then
    SWIFT_TEST_ARGS+=(--verbose)
fi

run_swift_tests() {
    local pattern="$1"
    for dir in "$ROOT"/$pattern/; do
        if [ -f "$dir/Package.swift" ]; then
            local name
            name=$(basename "$dir")
            echo "=== Testing $name ==="
            if (cd "$dir" && swift test "${SWIFT_TEST_ARGS[@]}" 2>&1 | tail -40); then
                RESULTS+=("$name: PASS")
            else
                RESULTS+=("$name: FAIL")
                FAILURES+=("$name")
            fi
        fi
    done
}

run_swift_tests "swift-libp2p*"
run_swift_tests "swift-gnostr*"

if [ "$RUST_FFI" -eq 1 ]; then
    if [ -f "$ROOT/gnostr-sdk-ffi/Cargo.toml" ]; then
        echo "=== Testing gnostr-sdk-ffi (cargo) ==="
        if (cd "$ROOT/gnostr-sdk-ffi" && cargo test 2>&1 | tail -40); then
            RESULTS+=("gnostr-sdk-ffi: PASS")
        else
            RESULTS+=("gnostr-sdk-ffi: FAIL")
            FAILURES+=("gnostr-sdk-ffi")
        fi
    fi
fi

echo ""
echo "=== SUMMARY ==="
for r in "${RESULTS[@]}"; do
    echo "$r"
done

if [ ${#FAILURES[@]} -gt 0 ]; then
    echo ""
    echo "Failures: ${FAILURES[*]}"
    exit 1
fi
