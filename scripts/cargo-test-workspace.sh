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
PACKAGES=()
ALL_FEATURES=false
NO_DEFAULT_FEATURES=false
RELEASE=false
LOCKED=false
OFFLINE=false

usage() {
  cat <<'EOF'
Usage: cargo-test-workspace.sh [variant] [--features VALUE] [--package VALUE] [--ignored] [--nocapture] [--quiet] [--release] [--locked] [--offline] [--test-threads VALUE]

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
  --package VALUE      Select a Cargo package (repeatable)
  --package=VALUE      Select a Cargo package
  --ignored            Pass --ignored to cargo test
  --nocapture          Pass --nocapture to cargo test
  --quiet              Pass --quiet to cargo test
  --release            Pass --release to cargo test
  --locked             Pass --locked to cargo test
  --offline            Pass --offline to cargo test
  --test-threads VALUE Pass --test-threads VALUE to the test harness
  --test-threads=VALUE Pass --test-threads VALUE to the test harness
  --help               Show this help
EOF
}

add_feature() {
  local feature="$1"
  FEATURES+=("$feature")
}

add_package() {
  local package="$1"
  PACKAGES+=("$package")
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
    --package)
      shift
      [[ $# -gt 0 ]] || { echo "--package requires a value" >&2; exit 1; }
      add_package "$1"
      ;;
    --package=*)
      add_package "${1#*=}"
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
    --release)
      RELEASE=true
      ;;
    --locked)
      LOCKED=true
      ;;
    --offline)
      OFFLINE=true
      ;;
    --test-threads)
      shift
      [[ $# -gt 0 ]] || { echo "--test-threads requires a value" >&2; exit 1; }
      TEST_FLAGS+=(--test-threads "$1")
      ;;
    --test-threads=*)
      TEST_FLAGS+=(--test-threads "${1#*=}")
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

if [[ "$RELEASE" == true ]]; then
  CARGO_FLAGS+=(--release)
fi

if [[ "$LOCKED" == true ]]; then
  CARGO_FLAGS+=(--locked)
fi

if [[ "$OFFLINE" == true ]]; then
  CARGO_FLAGS+=(--offline)
fi

for package in "${PACKAGES[@]}"; do
  CARGO_FLAGS+=(--package "$package")
done

if [[ ${#TEST_FLAGS[@]} -gt 0 ]]; then
  cargo "${CARGO_FLAGS[@]}" -- "${TEST_FLAGS[@]}"
else
  cargo "${CARGO_FLAGS[@]}"
fi
