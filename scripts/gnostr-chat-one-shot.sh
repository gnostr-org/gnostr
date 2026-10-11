#!/usr/bin/env bash
# Smoke test for `gnostr chat --oneshot`.
#
# Starts a headless chat peer, sends two oneshot messages to it, and asserts
# that both messages are received.  Exits non-zero if either message is missing.

set -euo pipefail

TOPIC="gnostr-oneshot-smoke-$(date +%s)"
RECEIVER_LOG="$(mktemp)"
SENDER1_LOG="$(mktemp)"
SENDER2_LOG="$(mktemp)"
GNOSTR="${GNOSTR:-./target/release/gnostr}"

cleanup() {
    if [[ -n "${RECEIVER_PID:-}" ]]; then
        kill "${RECEIVER_PID}" 2>/dev/null || true
        wait "${RECEIVER_PID}" 2>/dev/null || true
    fi
    rm -f "${RECEIVER_LOG}" "${SENDER1_LOG}" "${SENDER2_LOG}" \
        "${TMPDIR:-/tmp}/gnostr-chat-oneshot-${TOPIC}.json"
}
trap cleanup EXIT

if [[ ! -x "${GNOSTR}" ]]; then
    echo "error: ${GNOSTR} not found or not executable" >&2
    echo "run: cargo build --release --bin gnostr" >&2
    exit 1
fi

echo "==> starting headless receiver on topic ${TOPIC}"
"${GNOSTR}" chat --headless --topic "${TOPIC}" --name receiver >"${RECEIVER_LOG}" 2>&1 &
RECEIVER_PID=$!

# Wait for the receiver to begin listening.
for _ in $(seq 1 30); do
    if grep -q "Local node is listening" "${RECEIVER_LOG}" 2>/dev/null; then
        break
    fi
    sleep 0.2
done

if ! grep -q "Local node is listening" "${RECEIVER_LOG}" 2>/dev/null; then
    echo "error: receiver did not start listening" >&2
    cat "${RECEIVER_LOG}" >&2
    exit 1
fi

echo "==> receiver listening, sending first oneshot"
"${GNOSTR}" chat --topic "${TOPIC}" --name sender1 --oneshot "hello first" >"${SENDER1_LOG}" 2>&1

echo "==> sending second oneshot"
"${GNOSTR}" chat --topic "${TOPIC}" --name sender2 --oneshot "hello second" >"${SENDER2_LOG}" 2>&1

# Give the receiver a moment to print the inbound messages.
sleep 1

echo "==> receiver log tail:"
tail -n 40 "${RECEIVER_LOG}"

echo "==> sender 1 log:"
cat "${SENDER1_LOG}"

echo "==> sender 2 log:"
cat "${SENDER2_LOG}"

MISSING=0
if ! grep -q "hello first" "${RECEIVER_LOG}"; then
    echo "FAIL: receiver did not get 'hello first'" >&2
    MISSING=1
fi
if ! grep -q "hello second" "${RECEIVER_LOG}"; then
    echo "FAIL: receiver did not get 'hello second'" >&2
    MISSING=1
fi

if [[ "${MISSING}" -eq 0 ]]; then
    echo "PASS: both oneshot messages were received"
    exit 0
else
    exit 1
fi
