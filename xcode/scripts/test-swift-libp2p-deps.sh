#!/bin/bash
# Test all local swift-libp2p-* and swift-gnostr-* (FFI) packages.
# Usage: ./scripts/test-swift-libp2p-deps.sh [--verbose] [--rust-ffi] [--cargo-verbose|-vv] [--cargo-nocapture]

set -euo pipefail

VERBOSE=0
RUST_FFI=0
CARGO_VERBOSE=0
CARGO_NOCAPTURE=0
for arg in "$@"; do
    case "$arg" in
        --verbose|-v)
            VERBOSE=1
            ;;
        --rust-ffi)
            RUST_FFI=1
            ;;
        --cargo-verbose|-vv)
            CARGO_VERBOSE=1
            ;;
        --cargo-nocapture)
            CARGO_NOCAPTURE=1
            ;;
        *)
            echo "Unknown argument: $arg"
            echo "Usage: $0 [--verbose] [--rust-ffi] [--cargo-verbose|-vv] [--cargo-nocapture]"
            exit 1
            ;;
    esac
done

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
RESULTS=()
FAILURES=()
LOG_DIR="$ROOT/.build/swift-libp2p-test-logs"
mkdir -p "$LOG_DIR"

SWIFT_TEST_ARGS=()
if [ "$VERBOSE" -eq 1 ]; then
    SWIFT_TEST_ARGS+=(--verbose)
fi

CARGO_TEST_ARGS=()
CARGO_TEST_BINARY_ARGS=()
if [ "$CARGO_VERBOSE" -eq 1 ]; then
    CARGO_TEST_ARGS+=(-vv)
fi
if [ "$CARGO_NOCAPTURE" -eq 1 ]; then
    CARGO_TEST_BINARY_ARGS+=(--nocapture)
fi

run_one_swift_test() {
    local dir="$1"
    local log="$2"
    if [ ${#SWIFT_TEST_ARGS[@]} -gt 0 ]; then
        cd "$dir" && swift test "${SWIFT_TEST_ARGS[@]}" 2>&1 | tee "$log" | tail -40
    else
        cd "$dir" && swift test 2>&1 | tee "$log" | tail -40
    fi
    return "${PIPESTATUS[0]}"
}

run_swift_tests() {
    local pattern="$1"
    for dir in "$ROOT"/$pattern/; do
        if [ -f "$dir/Package.swift" ]; then
            local name
            name=$(basename "$dir")

            # Redis-backed packages require a running redis-server.
            if [[ "$name" == *redis* ]] && [ "$REDIS_AVAILABLE" -eq 0 ]; then
                echo "=== Skipping $name (redis-server not available) ==="
                RESULTS+=("$name: SKIP (redis-server not available)")
                continue
            fi

            local log="$LOG_DIR/$name.log"
            echo "=== Testing $name (log: $log) ==="
            if run_one_swift_test "$dir" "$log"; then
                RESULTS+=("$name: PASS")
            else
                RESULTS+=("$name: FAIL")
                FAILURES+=("$name")
            fi
        fi
    done
}

run_one_cargo_test() {
    local crate_path="$1"
    local log="$2"
    if [ ${#CARGO_TEST_ARGS[@]} -gt 0 ] && [ ${#CARGO_TEST_BINARY_ARGS[@]} -gt 0 ]; then
        cd "$crate_path" && cargo test "${CARGO_TEST_ARGS[@]}" -- "${CARGO_TEST_BINARY_ARGS[@]}" 2>&1 | tee "$log" | tail -40
    elif [ ${#CARGO_TEST_ARGS[@]} -gt 0 ]; then
        cd "$crate_path" && cargo test "${CARGO_TEST_ARGS[@]}" 2>&1 | tee "$log" | tail -40
    elif [ ${#CARGO_TEST_BINARY_ARGS[@]} -gt 0 ]; then
        cd "$crate_path" && cargo test -- "${CARGO_TEST_BINARY_ARGS[@]}" 2>&1 | tee "$log" | tail -40
    else
        cd "$crate_path" && cargo test 2>&1 | tee "$log" | tail -40
    fi
    return "${PIPESTATUS[0]}"
}

run_cargo_tests() {
    local crate_path="$1"
    local name="$2"
    local log="$LOG_DIR/$name.log"
    echo "=== Testing $name (cargo) (log: $log) ==="
    if run_one_cargo_test "$crate_path" "$log"; then
        RESULTS+=("$name: PASS")
    else
        RESULTS+=("$name: FAIL")
        FAILURES+=("$name")
    fi
}

REDIS_AVAILABLE=0
if command -v redis-server >/dev/null 2>&1; then
    REDIS_AVAILABLE=1
fi

run_swift_tests "swift-libp2p*"
run_swift_tests "swift-gnostr*"

if [ "$RUST_FFI" -eq 1 ]; then
    if [ -f "$ROOT/gnostr-sdk-ffi/Cargo.toml" ]; then
        run_cargo_tests "$ROOT/gnostr-sdk-ffi" "gnostr-sdk-ffi"
    fi

    for crate in "$ROOT"/swift-gnostr-*/Rust/*-ffi; do
        if [ -f "$crate/Cargo.toml" ]; then
            name=$(basename "$crate")
            run_cargo_tests "$crate" "$name"
        fi
    done
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
