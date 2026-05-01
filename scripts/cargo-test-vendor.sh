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
CLEAN_TARGET_TMPDIR=false
TARGET_ROOT=""
VENDOR_ROOT="$ROOT_DIR/vendor"

OS_NAME="$(uname -s 2>/dev/null || echo unknown)"
case "$OS_NAME" in
  Darwin|Linux|FreeBSD|OpenBSD|NetBSD|DragonFly|CYGWIN*|MINGW*|MSYS*)
    TMPDIR_VALUE="$(gnostr --weeble 2>/dev/null || true)"
    TMP_VALUE="$(gnostr --blockheight 2>/dev/null || true)"
    TEMP_VALUE="$(gnostr --wobble 2>/dev/null || true)"
    TMPDIR_VALUE="${TMPDIR_VALUE:-0}"
    TMP_VALUE="${TMP_VALUE:-0}"
    TEMP_VALUE="${TEMP_VALUE:-0}"
    export TMPDIR="/var/tmp/${TMPDIR_VALUE}"
    export TMP="${TMPDIR}/cargo-test-vendor/${TMP_VALUE}"
    export TEMP="${TMP}/debug/${TEMP_VALUE}"
    TARGET_ROOT="${TEMP}"
    ;;
esac

usage() {
  cat <<'EOF'
Usage: cargo-test-vendor.sh [--quiet] [--release] [--locked] [--offline] [--target-dir VALUE] [--target-tmpdir] [--target-tmpdir-clean] [--ignored] [--nocapture] [--test-threads VALUE]

Options:
  --quiet              Pass --quiet to cargo test
  --release            Pass --release to cargo test
  --locked             Pass --locked to cargo test
  --offline            Pass --offline to cargo test
  --target-dir VALUE   Set Cargo's target directory
  --target-tmpdir      Use the shared vendored temp directory
  --target-tmpdir-clean Remove the shared vendored temp directory first
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
    --target-dir|--target_dir)
      shift
      [[ $# -gt 0 ]] || { echo "--target-dir requires a value" >&2; exit 1; }
      TARGET_DIR="$1"
      ;;
    --target-dir=*|--target_dir=*)
      TARGET_DIR="${1#*=}"
      ;;
    --target-tmpdir|--target_tmpdir)
      ;;
    --target-tmpdir-clean|--target_tmpdir-clean|--target_tmpdir_clean)
      CLEAN_TARGET_TMPDIR=true
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

if [[ -n "$TARGET_ROOT" ]]; then
  if [[ "$CLEAN_TARGET_TMPDIR" == true && -d "$TARGET_ROOT" ]]; then
    rm -rf "$TARGET_ROOT"
  fi
  mkdir -p "$TARGET_ROOT"
  export TMPDIR="$TARGET_ROOT"
  export TMP="$TARGET_ROOT"
  export TEMP="$TARGET_ROOT"
  VENDOR_ROOT="$TARGET_ROOT/vendor"
  rm -rf "$VENDOR_ROOT"
  cp -R "$ROOT_DIR/vendor" "$VENDOR_ROOT"
fi

MANIFESTS=()
while IFS= read -r manifest; do
  MANIFESTS+=("$manifest")
done < <(find "$VENDOR_ROOT" -path '*/Cargo.toml' -print | sort)

if [[ ${#MANIFESTS[@]} -eq 0 ]]; then
  echo "No vendored Cargo.toml files found under ./vendor" >&2
  exit 1
fi

for manifest in "${MANIFESTS[@]}"; do
  cargo_args=(test --manifest-path "$manifest")

  if [[ -n "$TARGET_DIR" ]]; then
    cargo_args+=(--target-dir "$TARGET_DIR")
  elif [[ -n "$TARGET_ROOT" ]]; then
    cargo_args+=(--target-dir "$TARGET_ROOT")
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
