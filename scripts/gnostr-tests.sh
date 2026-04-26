#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

LIST_ONLY=false
TEST_NAME=""
TEST_FLAGS=()

usage() {
  cat <<'EOF'
Usage: gnostr-tests.sh [--workspace] [--list] [--test <name>] [--ignored] [--help]

Options:
  --workspace   Run workspace tests (default)
  --list        List all workspace tests
  --test NAME   Run one exact test by name
  --ignored     Run ignored tests
  --nocapture   Print test output
  --help        Show this help

Examples:
  ./scripts/gnostr-tests.sh --list
  ./scripts/gnostr-tests.sh --test sub_commands::dm::dm_tests::test_dm_command_success
  ./scripts/gnostr-tests.sh --ignored
  ./scripts/gnostr-tests.sh --workspace --ignored
  ./scripts/gnostr-tests.sh --test blossom_remote_push_list_and_fetch_round_trip -- --nocapture
EOF
}

check_nip44_vectors() {
  local vector_file="./asyncgit/src/lib/types/nip44/nip44.vectors.json"
  local expected_vector_sha256="269ed0f69e4c192512cc779e78c555090cebc7c785b609e338a62afc3ce25040"
  local actual_vector_sha256

  if command -v shasum >/dev/null 2>&1; then
    actual_vector_sha256="$(shasum -a 256 "$vector_file" | awk '{print $1}')"
  elif command -v sha256sum >/dev/null 2>&1; then
    actual_vector_sha256="$(sha256sum "$vector_file" | awk '{print $1}')"
  else
    echo "No SHA-256 tool found (shasum or sha256sum)" >&2
    exit 1
  fi

  if [[ "$actual_vector_sha256" != "$expected_vector_sha256" ]]; then
    echo "nip44 vector hash mismatch: expected $expected_vector_sha256, got $actual_vector_sha256" >&2
    exit 1
  fi
}

send_chat_update() {
  local message="$1"
  cargo run --bin gnostr -- chat --topic gnostr-dev --name copilot --oneshot "$message" >/dev/null 2>&1 || true
}

run_test_step() {
  local test_name="$1"
  shift
  if "$@"; then
    send_chat_update "$test_name successful"
  else
    send_chat_update "$test_name fail"
    return 1
  fi
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --workspace)
      :
      ;;
    --list)
      LIST_ONLY=true
      ;;
    --test)
      shift
      if [[ $# -eq 0 || -z "${1:-}" ]]; then
        echo "--test requires a test name" >&2
        exit 1
      fi
      TEST_NAME="$1"
      ;;
    --nocapture)
      TEST_FLAGS+=(--nocapture)
      ;;
    --ignored)
      TEST_FLAGS+=(--ignored)
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

if [[ -n "$TEST_NAME" && "$LIST_ONLY" == true ]]; then
  echo "--list cannot be combined with --test" >&2
  exit 1
fi

check_nip44_vectors

if [[ "$LIST_ONLY" == true ]]; then
  cargo test --workspace --all-targets -- --list
  exit 0
fi

if [[ -n "$TEST_NAME" ]]; then
  if [[ ${#TEST_FLAGS[@]} -gt 0 ]]; then
    cargo_cmd=(cargo test --workspace --all-targets "$TEST_NAME" -- --exact "${TEST_FLAGS[@]}")
  else
    cargo_cmd=(cargo test --workspace --all-targets "$TEST_NAME" -- --exact)
  fi
  if "${cargo_cmd[@]}"; then
    cargo run --bin gnostr -- chat --topic gnostr-dev --name copilot --oneshot "gnostr workspace test ${TEST_NAME} successful" >/dev/null 2>&1 || true
  else
    cargo run --bin gnostr -- chat --topic gnostr-dev --name copilot --oneshot "gnostr workspace test ${TEST_NAME} fail" >/dev/null 2>&1 || true
    exit 1
  fi
else
  if [[ ${#TEST_FLAGS[@]} -gt 0 ]]; then
    cargo_cmd=(cargo test --workspace --all-targets -- "${TEST_FLAGS[@]}")
  else
    cargo_cmd=(cargo test --workspace --all-targets --)
  fi
  if "${cargo_cmd[@]}"; then
    cargo run --bin gnostr -- chat --topic gnostr-dev --name copilot --oneshot "gnostr workspace test suite successful" >/dev/null 2>&1 || true
  else
    cargo run --bin gnostr -- chat --topic gnostr-dev --name copilot --oneshot "gnostr workspace test suite fail" >/dev/null 2>&1 || true
    exit 1
  fi
fi
