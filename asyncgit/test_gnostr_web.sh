#!/bin/bash

set -euo pipefail

PORT=${1:-3333}
LOG_DIR="/Users/git/.gemini/tmp/dd5d8e1cc43eacbaf904327a9ec45d0cf979831f0dcbf352add1fc5cdae5520a"
STDOUT_LOG="$LOG_DIR/gnostr-gnit_test_stdout.log"
STDERR_LOG="$LOG_DIR/gnostr-gnit_test_stderr.log"
DB_DIR=".gnostr/web/$PORT"
SERVER_BIN="./bin/gnostr-gnit"

echo "--- Setting up test environment ---"
# Kill any existing gnostr-gnit processes
pgrep gnostr-gnit && kill $(pgrep gnostr-gnit) || true
rm -rf "$DB_DIR"
rm -f "$STDOUT_LOG" "$STDERR_LOG"

# Build the project (ensures binary is up-to-date)
echo "Building gnostr-gnit..."
cargo b

# Start the server with debug logging
echo "Starting gnostr-gnit server on port $PORT with RUST_LOG=debug..."
RUST_LOG=debug "$SERVER_BIN" -d "$DB_DIR" -b "$PORT" -s . > "$STDOUT_LOG" 2> "$STDERR_LOG" &
SERVER_PID=$!
echo "Server started with PID: $SERVER_PID"

echo "Waiting 10 seconds for server to start and index..."
sleep 10

echo "--- Running tests ---"

# Test 1: Root page accessibility and content (expect 200 OK)
echo "1. Testing root page: http://localhost:$PORT/"
curl -s -f http://localhost:"$PORT"/ > /dev/null
if [ $? -eq 0 ]; then
    echo "   PASS: Root page is accessible."
else
    echo "   FAIL: Root page is not accessible."
    cat "$STDERR_LOG"
    kill "$SERVER_PID" || true
    exit 1
fi

# Test 2: Check if gnostr-gnit.git is listed on the root page
if curl -s http://localhost:"$PORT"/ | grep -q "gnostr-gnit.git"; then
    echo "   PASS: gnostr-gnit.git is listed on the root page."
else
    echo "   FAIL: gnostr-gnit.git is NOT listed on the root page."
    kill "$SERVER_PID" || true
    exit 1
fi

# Test 3: gnostr-gnit.git summary page (expect 200 OK)
echo "2. Testing gnostr-gnit.git summary page: http://localhost:$PORT/gnostr-gnit.git"
curl -s -f http://localhost:"$PORT"/gnostr-gnit.git > /dev/null
if [ $? -eq 0 ]; then
    echo "   PASS: gnostr-gnit.git summary page is accessible."
else
    echo "   FAIL: gnostr-gnit.git summary page is NOT accessible."
    cat "$STDERR_LOG"
    kill "$SERVER_PID" || true
    exit 1
fi

# Test 4: gnostr-gnit.git tree view (expect 200 OK)
echo "3. Testing gnostr-gnit.git tree view: http://localhost:$PORT/gnostr-gnit.git/tree"
curl -s -f http://localhost:"$PORT"/gnostr-gnit.git/tree > /dev/null
if [ $? -eq 0 ]; then
    echo "   PASS: gnostr-gnit.git tree view is accessible."
else
    echo "   FAIL: gnostr-gnit.git tree view is NOT accessible."
    cat "$STDERR_LOG"
    kill "$SERVER_PID" || true
    exit 1
fi

# Test 5: Non-existent repository (expect 404 Not Found)
echo "4. Testing non-existent repository: http://localhost:$PORT/non-existent-repo"
if curl -s -f http://localhost:"$PORT"/non-existent-repo > /dev/null; then
    echo "   FAIL: Non-existent repository returned 200 OK."
    kill "$SERVER_PID" || true
    exit 1
else
    # Check if curl failed with a 404 (or similar, indicating not found)
    if curl -s -o /dev/null -w "%{http_code}" http://localhost:"$PORT"/non-existent-repo | grep -q "404"; then
        echo "   PASS: Non-existent repository returned 404 Not Found."
    else
        echo "   FAIL: Non-existent repository did not return 404 Not Found."
        kill "$SERVER_PID" || true
        exit 1
    fi
fi

# Test 6: Verify no target/ directories are listed as repos on the root page (negative test)
echo "5. Verifying no 'target/' directories are listed as repositories..."
if curl -s http://localhost:"$PORT"/ | grep -q "/target/release/build/"; then
    echo "   FAIL: 'target/' directory artifacts ARE listed as repositories."
    kill "$SERVER_PID" || true
    exit 1
else
    echo "   PASS: No 'target/' directory artifacts found in repository list."
fi

echo "--- All tests completed successfully! ---"

echo "--- Cleaning up ---"
kill "$SERVER_PID" || true
rm -f "$STDOUT_LOG" "$STDERR_LOG"
echo "Cleanup complete."
exit 0
