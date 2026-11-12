#!/usr/bin/env bash
set -euo pipefail

if ! bash -n "${BASH_SOURCE[0]}"; then
  exit 1
fi

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

MODE="tests"
NOCAPTURE=true
TEST_FLAGS=()
TEST_NAME=""

export RUST_LOG="${RUST_LOG:+$RUST_LOG,}ureq=off,serial_test=off,mio=off,tungstenite=off,tokio_tungstenite=off"

usage() {
  cat <<'EOF'
Usage: gnostr-p2p-tests.sh [tests|integration] [--test NAME] [--ignored] [--nocapture] [--capture] [--help]

Modes:
  tests        Run p2p cargo tests (default)
  integration  Run the p2p relay/subscriber/publisher flow

Options:
  --test NAME   Run one exact cargo test when in tests mode
  --ignored     Include ignored cargo tests when in tests mode
  --nocapture   Print test output
  --capture     Silence test output
  --help        Show this help
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    tests|integration)
      MODE="$1"
      ;;
    --integration)
      MODE="integration"
      ;;
    --ignored)
      TEST_FLAGS+=(--ignored)
      ;;
    --nocapture)
      NOCAPTURE=true
      ;;
    --capture)
      NOCAPTURE=false
      ;;
    --test)
      shift
      [[ $# -gt 0 ]] || { echo "--test requires a test name" >&2; exit 1; }
      TEST_NAME="$1"
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      echo "Unsupported flag: $1" >&2
      exit 1
      ;;
  esac
  shift
done

run_tests_suite() {
  local -a test_flags=()
  if [[ ${#TEST_FLAGS[@]} -gt 0 ]]; then
    test_flags+=("${TEST_FLAGS[@]}")
  fi
  if [[ "$NOCAPTURE" == true ]]; then
    test_flags+=(--nocapture)
  fi

  if [[ -n "$TEST_NAME" ]]; then
    bash ./scripts/with-system-rocksdb.sh cargo test -p gnostr-p2p "$TEST_NAME" -- --exact "${test_flags[@]}"
  else
    bash ./scripts/with-system-rocksdb.sh cargo test -p gnostr-p2p -- "${test_flags[@]}"
  fi
}

run_integration_suite() {
  if [[ -n "$TEST_NAME" ]]; then
    echo "--test is only supported in tests mode" >&2
    exit 1
  fi

  if [[ ${#TEST_FLAGS[@]} -gt 0 ]]; then
    echo "--ignored is only supported in tests mode" >&2
    exit 1
  fi

  if ! command -v gnostr >/dev/null 2>&1; then
    echo "gnostr is required on PATH" >&2
    exit 1
  fi

  workdir="${TMPDIR:-/tmp}/gnostr-p2p-test.$(gnostr --blockheight 2>/dev/null || echo 0)"
  logs_dir="$workdir/logs"
  mkdir -p "$logs_dir"

  relay_home="$workdir/relay-home"
  subscriber_one_home="$workdir/subscriber-one-home"
  subscriber_two_home="$workdir/subscriber-two-home"
  publisher_home="$workdir/publisher-home"

  mkdir -p "$publisher_home/gnostr/crawler/42"
  printf '%s\n' '["wss://relay.example"]' >"$publisher_home/gnostr/crawler/42/relays.json"

  export RUST_LOG="${RUST_LOG:+$RUST_LOG,}info,ureq=off,serial_test=off,mio=off,tungstenite=off,tokio_tungstenite=off"

  pids=()

  launch() {
    local log_file="$1"
    shift
    (
      "$@"
    ) >"$log_file" 2>&1 &
    pids+=("$!")
  }

  cleanup() {
    local pid
    for pid in "${pids[@]:-}"; do
      kill "$pid" 2>/dev/null || true
    done
    wait 2>/dev/null || true
    rm -rf "$workdir"
  }

  trap cleanup EXIT INT TERM

  bash ./scripts/with-system-rocksdb.sh cargo build -p gnostr-p2p --bins --quiet

  start_relay() {
    export XDG_CONFIG_HOME="$relay_home"
    ./target/debug/gnostr-p2p-relay-server \
      --secret-key-seed "$(gnostr --hash "gnostr-p2p-relay-server")" \
      --port 0
  }

  start_subscriber_one() {
    export XDG_CONFIG_HOME="$subscriber_one_home"
    {
      printf 'TOPIC crawler/relay-buckets/42\n'
      sleep 25
      printf 'QUIT\n'
    } | ./target/debug/gnostr-p2p \
      --secret-key-seed "$(gnostr --hash "gnostr-p2p-subscriber-one")" \
      --port 0
  }

  start_subscriber_two() {
    export XDG_CONFIG_HOME="$subscriber_two_home"
    {
      printf 'TOPIC crawler/relay-buckets/42\n'
      sleep 25
      printf 'QUIT\n'
    } | ./target/debug/gnostr-p2p \
      --secret-key-seed "$(gnostr --hash "gnostr-p2p-subscriber-two")" \
      --port 0
  }

  start_publisher() {
    export XDG_CONFIG_HOME="$publisher_home"
    {
      sleep 5
      printf 'CRAWLER_BUCKETS\n'
      sleep 5
      printf 'QUIT\n'
    } | ./target/debug/gnostr-p2p \
      --secret-key-seed "$(gnostr --hash "gnostr-p2p-publisher")" \
      --port 0
  }

  relay_log="$logs_dir/relay.log"
  launch "$relay_log" start_relay

  sleep 2

  subscriber_one_log="$logs_dir/subscriber-one.log"
  launch "$subscriber_one_log" start_subscriber_one

  subscriber_two_log="$logs_dir/subscriber-two.log"
  launch "$subscriber_two_log" start_subscriber_two

  sleep 5

  publisher_log="$logs_dir/publisher.log"
  launch "$publisher_log" start_publisher

  for pid in "${pids[@]}"; do
    wait "$pid"
  done

  grep -F "broadcasted 1 crawler relay bucket" "$publisher_log" >/dev/null
  grep -F "Received message:" "$subscriber_one_log" >/dev/null
  grep -F "Received message:" "$subscriber_two_log" >/dev/null

  printf 'relay log: %s\n' "$relay_log"
  printf 'subscriber one log: %s\n' "$subscriber_one_log"
  printf 'subscriber two log: %s\n' "$subscriber_two_log"
  printf 'publisher log: %s\n' "$publisher_log"
}

case "$MODE" in
  tests)
    run_tests_suite
    ;;
  integration)
    run_integration_suite
    ;;
  *)
    echo "Unsupported mode: $MODE" >&2
    exit 1
    ;;
esac
