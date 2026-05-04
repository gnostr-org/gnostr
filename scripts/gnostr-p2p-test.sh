#!/usr/bin/env bash
set -euo pipefail

if ! bash -n "${BASH_SOURCE[0]}"; then
  exit 1
fi

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

if ! command -v gnostr >/dev/null 2>&1; then
  echo "gnostr is required on PATH" >&2
  exit 1
fi

workdir="$(mktemp -d "${TMPDIR:-/tmp}/gnostr-p2p-test.XXXXXX")"
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

cleanup() {
  local pid
  for pid in "${pids[@]:-}"; do
    kill "$pid" 2>/dev/null || true
  done
  wait 2>/dev/null || true
  rm -rf "$workdir"
}

trap cleanup EXIT INT TERM

cargo build -p gnostr-p2p --bins --quiet

relay_log="$logs_dir/relay.log"
(
  export XDG_CONFIG_HOME="$relay_home"
  ./target/debug/gnostr-p2p-relay-server \
    --secret-key-seed "$(gnostr --hash "gnostr-p2p-relay-server")" \
    --port 0
) >"$relay_log" 2>&1 &
pids+=("$!")

sleep 2

subscriber_one_log="$logs_dir/subscriber-one.log"
(
  export XDG_CONFIG_HOME="$subscriber_one_home"
  {
    printf 'TOPIC crawler/relay-buckets/42\n'
    sleep 25
    printf 'QUIT\n'
  } | ./target/debug/gnostr-p2p \
    --secret-key-seed "$(gnostr --hash "gnostr-p2p-subscriber-one")" \
    --port 0
) >"$subscriber_one_log" 2>&1 &
pids+=("$!")

subscriber_two_log="$logs_dir/subscriber-two.log"
(
  export XDG_CONFIG_HOME="$subscriber_two_home"
  {
    printf 'TOPIC crawler/relay-buckets/42\n'
    sleep 25
    printf 'QUIT\n'
  } | ./target/debug/gnostr-p2p \
    --secret-key-seed "$(gnostr --hash "gnostr-p2p-subscriber-two")" \
    --port 0
) >"$subscriber_two_log" 2>&1 &
pids+=("$!")

sleep 5

publisher_log="$logs_dir/publisher.log"
(
  export XDG_CONFIG_HOME="$publisher_home"
  {
    sleep 5
    printf 'CRAWLER_BUCKETS\n'
    sleep 5
    printf 'QUIT\n'
  } | ./target/debug/gnostr-p2p \
    --secret-key-seed "$(gnostr --hash "gnostr-p2p-publisher")" \
    --port 0
) >"$publisher_log" 2>&1 &
pids+=("$!")

wait "${pids[1]}"
wait "${pids[2]}"
wait "${pids[3]}"

grep -F "broadcasted 1 crawler relay bucket" "$publisher_log" >/dev/null
grep -F "Received message:" "$subscriber_one_log" >/dev/null
grep -F "Received message:" "$subscriber_two_log" >/dev/null

printf 'relay log: %s\n' "$relay_log"
printf 'subscriber one log: %s\n' "$subscriber_one_log"
printf 'subscriber two log: %s\n' "$subscriber_two_log"
printf 'publisher log: %s\n' "$publisher_log"
