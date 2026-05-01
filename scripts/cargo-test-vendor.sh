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

TEST_FLAGS=()
QUIET=false
RELEASE=false
LOCKED=false
OFFLINE=false
TARGET_DIR=""

OS_NAME="$(uname -s 2>/dev/null || echo unknown)"
case "$OS_NAME" in
  Darwin|Linux|FreeBSD|OpenBSD|NetBSD|DragonFly|CYGWIN*|MINGW*|MSYS*)
    TMP_BASE="${TMPDIR:-${TMP:-${TEMP:-/tmp}}}"
    if command -v mktemp >/dev/null 2>&1; then
      TARGET_DIR="$(mktemp -d "${TMP_BASE%/}/cargo-test-vendor.XXXXXX")"
    else
      TARGET_DIR="${TMP_BASE%/}/cargo-test-vendor.$$"
      mkdir -p "$TARGET_DIR"
    fi
    trap '[[ -n "${TARGET_DIR:-}" && -d "$TARGET_DIR" ]] && rm -rf "$TARGET_DIR"' EXIT
    ;;
esac

usage() {
  cat <<'EOF'
Usage: cargo-test-vendor.sh [--quiet] [--release] [--locked] [--offline] [--ignored] [--nocapture] [--test-threads VALUE]

Options:
  --quiet              Pass --quiet to cargo test
  --release            Pass --release to cargo test
  --locked             Pass --locked to cargo test
  --offline            Pass --offline to cargo test
  --ignored            Pass --ignored to cargo test
  --nocapture          Pass --nocapture to cargo test
  --test-threads VALUE Pass --test-threads VALUE to the test harness
  --test-threads=VALUE Pass --test-threads VALUE to the test harness
  --help               Show this help
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --quiet)
      QUIET=true
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
    --ignored)
      TEST_FLAGS+=(--ignored)
      ;;
    --nocapture)
      TEST_FLAGS+=(--nocapture)
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

MANIFESTS=()
while IFS= read -r manifest; do
  MANIFESTS+=("$manifest")
done < <(find vendor -path '*/Cargo.toml' -print | sort)

if [[ ${#MANIFESTS[@]} -eq 0 ]]; then
  echo "No vendored Cargo.toml files found under ./vendor" >&2
  exit 1
fi

for manifest in "${MANIFESTS[@]}"; do
  cargo_args=(test --manifest-path "$manifest")

  if [[ -n "$TARGET_DIR" ]]; then
    cargo_args+=(--target-dir "$TARGET_DIR")
  fi

  if [[ "$QUIET" == true ]]; then
    cargo_args+=(--quiet)
  fi

  if [[ "$RELEASE" == true ]]; then
    cargo_args+=(--release)
  fi

  if [[ "$LOCKED" == true ]]; then
    cargo_args+=(--locked)
  fi

  if [[ "$OFFLINE" == true ]]; then
    cargo_args+=(--offline)
  fi

  printf '==> %s\n' "$manifest"
  if [[ ${#TEST_FLAGS[@]} -gt 0 ]]; then
    cargo "${cargo_args[@]}" -- "${TEST_FLAGS[@]}"
  else
    cargo "${cargo_args[@]}"
  fi
done
