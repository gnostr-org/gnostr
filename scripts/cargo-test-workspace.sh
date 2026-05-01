#!/usr/bin/env bash
set -euo pipefail

BASH_VERSION_CURRENT="${BASH_VERSION:-unknown}"
BASH_MAJOR="${BASH_VERSINFO[0]:-0}"
BASH_MINOR="${BASH_VERSINFO[1]:-0}"
if ! bash -n "${BASH_SOURCE[0]}"; then
  exit 1
fi

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

NPROC="$(sysctl -n hw.logicalcpu 2>/dev/null || nproc 2>/dev/null || echo 1)"
TEST_FLAGS=()
FEATURES=()
ALL_FEATURES=false
NO_DEFAULT_FEATURES=false

usage() {
  cat <<'EOF'
Usage: cargo-test-workspace.sh [variant] [--features VALUE] [--ignored] [--nocapture] [--quiet]

Variants:
  default              cargo test --workspace
  workspace            Alias for default
  ignored              Run ignored workspace tests
  long-tests           Enable the long_tests feature
  long-tests-ignored   Enable long_tests and run ignored tests
  all-features         Run workspace tests with --all-features
  no-default-features  Run workspace tests with --no-default-features

Options:
  --features VALUE     Add a Cargo feature (repeatable)
  --features=VALUE     Add a Cargo feature
  --ignored            Pass --ignored to cargo test
  --nocapture          Pass --nocapture to cargo test
  --quiet              Pass --quiet to cargo test
  --help               Show this help
EOF
}

add_feature() {
  local feature="$1"
  FEATURES+=("$feature")
}

join_features() {
  local IFS=,
  echo "${FEATURES[*]}"
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    default|workspace)
      ;;
    ignored)
      TEST_FLAGS+=(--ignored)
      ;;
    long-tests|long_tests)
      add_feature long_tests
      ;;
    long-tests-ignored|long_tests_ignored)
      add_feature long_tests
      TEST_FLAGS+=(--ignored)
      ;;
    all-features)
      ALL_FEATURES=true
      ;;
    no-default-features)
      NO_DEFAULT_FEATURES=true
      ;;
    --features)
      shift
      [[ $# -gt 0 ]] || { echo "--features requires a value" >&2; exit 1; }
      add_feature "$1"
      ;;
    --features=*)
      add_feature "${1#*=}"
      ;;
    --ignored)
      TEST_FLAGS+=(--ignored)
      ;;
    --nocapture)
      TEST_FLAGS+=(--nocapture)
      ;;
    --quiet)
      TEST_FLAGS+=(--quiet)
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      echo "Unsupported argument: $1" >&2
      exit 1
      ;;
  esac
  shift
done

declare -a CARGO_FLAGS=(test --workspace -j"$NPROC")

if [[ "$ALL_FEATURES" == true ]]; then
  CARGO_FLAGS+=(--all-features)
elif [[ "$NO_DEFAULT_FEATURES" == true ]]; then
  CARGO_FLAGS+=(--no-default-features)
elif [[ ${#FEATURES[@]} -gt 0 ]]; then
  CARGO_FLAGS+=(--features "$(join_features)")
fi

if [[ ${#TEST_FLAGS[@]} -gt 0 ]]; then
  cargo "${CARGO_FLAGS[@]}" -- "${TEST_FLAGS[@]}"
else
  cargo "${CARGO_FLAGS[@]}"
fi
