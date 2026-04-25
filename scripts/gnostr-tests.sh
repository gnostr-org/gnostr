#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

LIST_ONLY=false
TEST_NAME=""
RUN_IGNORED=false

usage() {
  cat <<'EOF'
Usage: gnostr-tests.sh [--workspace] [--list] [--test <name>] [--ignored] [--help]

Options:
  --workspace   Run workspace tests (default)
  --list        List all workspace tests
  --test NAME   Run one exact test by name
  --ignored     Run ignored tests
  --help        Show this help

Examples:
  ./scripts/gnostr-tests.sh --list
  ./scripts/gnostr-tests.sh --test sub_commands::dm::dm_tests::test_dm_command_success
  ./scripts/gnostr-tests.sh --workspace --ignored
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
    --ignored)
      RUN_IGNORED=true
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
  if [[ "$RUN_IGNORED" == true ]]; then
    cargo test --workspace --all-targets "$TEST_NAME" -- --exact --nocapture --ignored
  else
    cargo test --workspace --all-targets "$TEST_NAME" -- --exact --nocapture
  fi
else
  if [[ "$RUN_IGNORED" == true ]]; then
    cargo test --workspace --all-targets -- --nocapture --ignored
  else
    cargo test --workspace --all-targets -- --nocapture
  fi
fi
