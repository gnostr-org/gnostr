#!/bin/bash
# Test all local swift-libp2p-* packages.
# Run from repository root.

set -e

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
RESULTS=()
FAILURES=()

for dir in "$ROOT"/swift-libp2p*/; do
    if [ -f "$dir/Package.swift" ]; then
        name=$(basename "$dir")
        echo "=== Testing $name ==="
        if (cd "$dir" && swift test 2>&1 | tail -20); then
            RESULTS+=("$name: PASS")
        else
            RESULTS+=("$name: FAIL")
            FAILURES+=("$name")
        fi
    fi
done

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
