#!/usr/bin/env bash
# Smoke test for `gnostr chat --oneshot`.
#
# Starts a headless chat peer, sends one or more oneshot messages to it, and
# asserts that every message is received.  Passes unknown `gnostr chat` flags
# through to both the receiver and the senders.
#
# Usage:
#   ./scripts/gnostr-chat-one-shot.sh [OPTIONS] [message ...]
#
# Options:
#   --topic TOPIC          Chat topic (default: gnostr-oneshot-smoke-<timestamp>)
#   --receiver-name NAME   Name for the headless receiver (default: receiver)
#   --sender-name NAME     Base name for senders (default: sender)
#   --messages M1,M2,...   Comma-separated messages (alternative to positional args)
#   --help                 Show this help
#
# Examples:
#   ./scripts/gnostr-chat-one-shot.sh
#   ./scripts/gnostr-chat-one-shot.sh --topic my-topic "hello" "world"
#   ./scripts/gnostr-chat-one-shot.sh --topic dev --name copilot --messages "one,two,three"

set -euo pipefail

GNOSTR="${GNOSTR:-./target/release/gnostr}"
TOPIC=""
RECEIVER_NAME="receiver"
SENDER_NAME="sender"
MESSAGES=""
PASSTHROUGH_FLAGS=()

usage() {
    sed -n '/^# /,/^# /p' "$0" | sed 's/^# //'
    exit "${1:-0}"
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --help|-h)
            usage 0
            ;;
        --topic)
            TOPIC="$2"
            shift 2
            ;;
        --receiver-name)
            RECEIVER_NAME="$2"
            shift 2
            ;;
        --sender-name)
            SENDER_NAME="$2"
            shift 2
            ;;
        --messages)
            MESSAGES="$2"
            shift 2
            ;;
        --)
            shift
            break
            ;;
        --*)
            if [[ $# -lt 2 ]]; then
                echo "error: missing value for flag $1" >&2
                usage 1
            fi
            PASSTHROUGH_FLAGS+=("$1" "$2")
            shift 2
            ;;
        *)
            break
            ;;
    esac
done

# Positional arguments are messages.
if [[ $# -gt 0 ]]; then
    MESSAGES="$(printf '%s,' "$@")"
    MESSAGES="${MESSAGES%,}"
fi

# Default messages.
if [[ -z "${MESSAGES}" ]]; then
    MESSAGES="hello first,hello second"
fi

# Default topic.
if [[ -z "${TOPIC}" ]]; then
    TOPIC="gnostr-oneshot-smoke-$(date +%s)"
fi

IFS=',' read -ra MSG_ARRAY <<< "${MESSAGES}"

RECEIVER_LOG="$(mktemp)"
SENDER_LOGS=()

cleanup() {
    if [[ -n "${RECEIVER_PID:-}" ]]; then
        kill "${RECEIVER_PID}" 2>/dev/null || true
        wait "${RECEIVER_PID}" 2>/dev/null || true
    fi
    rm -f "${RECEIVER_LOG}" "${SENDER_LOGS[@]}" \
        "${TMPDIR:-/tmp}/gnostr-chat-oneshot-${TOPIC}.json"
}
trap cleanup EXIT

if [[ ! -x "${GNOSTR}" ]]; then
    echo "error: ${GNOSTR} not found or not executable" >&2
    echo "run: cargo build --release --bin gnostr" >&2
    exit 1
fi

echo "==> starting headless receiver on topic ${TOPIC}"
"${GNOSTR}" chat --headless --topic "${TOPIC}" --name "${RECEIVER_NAME}" "${PASSTHROUGH_FLAGS[@]}" >"${RECEIVER_LOG}" 2>&1 &
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

for i in "${!MSG_ARRAY[@]}"; do
    msg="${MSG_ARRAY[$i]}"
    sender_name="${SENDER_NAME}-$((i + 1))"
    log="$(mktemp)"
    SENDER_LOGS+=("${log}")
    echo "==> sending oneshot $((i + 1))/${#MSG_ARRAY[@]}: ${msg}"
    "${GNOSTR}" chat --topic "${TOPIC}" --name "${sender_name}" --oneshot "${msg}" "${PASSTHROUGH_FLAGS[@]}" >"${log}" 2>&1
done

# Give the receiver a moment to print the inbound messages.
sleep 1

echo "==> receiver log tail:"
tail -n 40 "${RECEIVER_LOG}"

for log in "${SENDER_LOGS[@]}"; do
    echo "==> sender log:"
    cat "${log}"
done

MISSING=0
for msg in "${MSG_ARRAY[@]}"; do
    if ! grep -qF "${msg}" "${RECEIVER_LOG}"; then
        echo "FAIL: receiver did not get '${msg}'" >&2
        MISSING=1
    fi
done

if [[ "${MISSING}" -eq 0 ]]; then
    echo "PASS: all ${#MSG_ARRAY[@]} oneshot message(s) were received"
    exit 0
else
    exit 1
fi
