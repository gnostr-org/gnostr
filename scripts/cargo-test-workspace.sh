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

export RUST_LOG="${RUST_LOG:+$RUST_LOG,}ureq=warn"

NPROC="$(sysctl -n hw.logicalcpu 2>/dev/null || nproc 2>/dev/null || echo 1)"
TEST_FLAGS=()
FEATURES=()
PACKAGES=()
TARGET_DIR=""
TARGET_TMPDIR=false
TARGET_TMPDIR_CLEAN=false
PRUNE_LIMIT_SPEC="20G"
ALL_FEATURES=false
NO_DEFAULT_FEATURES=false
VENDORED=false
RELEASE=false
LOCKED=false
OFFLINE=false

    TMPDIR_VALUE="$(gnostr --weeble 2>/dev/null || true)"
    TMP_VALUE="$(gnostr --blockheight 2>/dev/null || true)"
    TEMP_VALUE="$(gnostr --wobble 2>/dev/null || true)"
    TMPDIR_VALUE="${TMPDIR_VALUE:-0}"
    TMP_VALUE="${TMP_VALUE:-0}"
    TEMP_VALUE="${TEMP_VALUE:-0}"
    export TMPDIR="/var/tmp/cargo/test/workspace/${TMPDIR_VALUE}"
    export TMP="${TMPDIR}/${TMP_VALUE}"
    export TEMP="${TMP}/debug/${TEMP_VALUE}"
TARGET_ROOT="${TEMP}"

usage() {
  cat <<'EOF'
Usage: cargo-test-workspace.sh [variant] [--features VALUE] [--package VALUE] [--ignored] [--nocapture] [--quiet] [--release] [--locked] [--offline] [--test-threads VALUE]

Variants:
  default              cargo test --workspace
  workspace            Alias for default
  vendored             Run cargo tests for every vendored manifest
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
  --target-dir VALUE   Set Cargo's target directory
  --target-dir=VALUE   Set Cargo's target directory
  --target-tmpdir      Use the shared temp directory
  --target-tmpdir-clean Remove the shared temp directory first
  --prune-limit VALUE  Prune oldest dirs once the tree reaches VALUE (default 20G)
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

parse_prune_limit_kib() {
  local raw
  local value unit

  raw="$(printf '%s' "$1" | tr '[:lower:]' '[:upper:]')"

  if [[ "$raw" =~ ^([0-9]+)([KMG])?$ ]]; then
    value="${BASH_REMATCH[1]}"
    unit="${BASH_REMATCH[2]}"
    case "$unit" in
      ""|K) echo "$value" ;;
      M) echo $((value * 1024)) ;;
      G) echo $((value * 1024 * 1024)) ;;
      *) return 1 ;;
    esac
  else
    return 1
  fi
}

target_dir_size_kib() {
  local target_path="$1"

  if [[ -z "$target_path" || ! -d "$target_path" ]]; then
    echo 0
    return
  fi

  du -sk "$target_path" 2>/dev/null | awk '{print $1+0}'
}

oldest_child_dir() {
  local prune_root="$1"
  local os_name

  os_name="$(uname -s 2>/dev/null || echo unknown)"
  find "$prune_root" -mindepth 1 -maxdepth 1 -type d 2>/dev/null | while IFS= read -r child; do
    case "$os_name" in
      Darwin|FreeBSD|OpenBSD|NetBSD|DragonFly)
        stat -f '%m\t%N' "$child" 2>/dev/null || true
        ;;
      *)
        stat -c '%Y\t%n' "$child" 2>/dev/null || true
        ;;
    esac
  done | sort -n | head -1 | awk -F'\t' '{print $2}'
}

prune_target_tree() {
  local prune_root="$1"
  local current_size
  local oldest

  if [[ -z "$prune_root" || ! -d "$prune_root" ]]; then
    return 0
  fi

  while :; do
    current_size="$(target_dir_size_kib "$prune_root")"
    if (( current_size <= TARGET_SIZE_LIMIT_KIB )); then
      break
    fi

    oldest="$(oldest_child_dir "$prune_root")"
    if [[ -z "$oldest" ]]; then
      break
    fi

    printf 'pruning oldest target dir: %s\n' "$oldest"
    rm -rf "$oldest"
  done
}

report_target_dir_size() {
  local target_path="$1"
  local size

  if [[ -z "$target_path" || ! -d "$target_path" ]]; then
    return 0
  fi

  size="$(du -sh "$target_path" 2>/dev/null | awk '{print $1}')"
  if [[ -n "$size" ]]; then
    printf 'target dir size: %s (%s)\n' "$size" "$target_path"
  fi
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
    vendored)
      VENDORED=true
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
    --target-dir|--target_dir)
      shift
      [[ $# -gt 0 ]] || { echo "--target-dir requires a value" >&2; exit 1; }
      TARGET_DIR="$1"
      ;;
    --target-dir=*|--target_dir=*)
      TARGET_DIR="${1#*=}"
      ;;
    --target-tmpdir|--target_tmpdir)
      TARGET_TMPDIR=true
      ;;
    --target-tmpdir-clean|--target_tmpdir-clean)
      TARGET_TMPDIR_CLEAN=true
      ;;
    --prune-limit)
      shift
      [[ $# -gt 0 ]] || { echo "--prune-limit requires a value" >&2; exit 1; }
      PRUNE_LIMIT_SPEC="$1"
      ;;
    --prune-limit=*)
      PRUNE_LIMIT_SPEC="${1#*=}"
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

TARGET_SIZE_LIMIT_KIB="$(parse_prune_limit_kib "$PRUNE_LIMIT_SPEC")" || {
  echo "Invalid --prune-limit value: $PRUNE_LIMIT_SPEC" >&2
  exit 1
}

if [[ "$VENDORED" == true ]]; then
  if [[ "$ALL_FEATURES" == true || "$NO_DEFAULT_FEATURES" == true || ${#FEATURES[@]} -gt 0 || ${#PACKAGES[@]} -gt 0 ]]; then
    echo "vendored mode does not support --features, --package, all-features, or no-default-features" >&2
    exit 1
  fi

  VENDORED_FLAGS=()

  if [[ "$RELEASE" == true ]]; then
    VENDORED_FLAGS+=(--release)
  fi

  if [[ "$LOCKED" == true ]]; then
    VENDORED_FLAGS+=(--locked)
  fi

  if [[ "$OFFLINE" == true ]]; then
    VENDORED_FLAGS+=(--offline)
  fi

  if [[ -n "$TARGET_DIR" ]]; then
    VENDORED_FLAGS+=(--target-dir "$TARGET_DIR")
  fi

  if [[ "$TARGET_TMPDIR" == true ]]; then
    VENDORED_FLAGS+=(--target-tmpdir)
  fi

  if [[ "$TARGET_TMPDIR_CLEAN" == true ]]; then
    VENDORED_FLAGS+=(--target-tmpdir-clean)
  fi

  if [[ ${#TEST_FLAGS[@]} -gt 0 ]]; then
    ./scripts/cargo-test-vendor.sh "${VENDORED_FLAGS[@]}" "${TEST_FLAGS[@]}"
  else
    ./scripts/cargo-test-vendor.sh "${VENDORED_FLAGS[@]}"
  fi
  exit $?
fi

if [[ "$TARGET_TMPDIR_CLEAN" == true && -d "$TARGET_ROOT" ]]; then
  rm -rf "$TARGET_ROOT"
fi
mkdir -p "$TARGET_ROOT"

if [[ "$TARGET_TMPDIR" == true || -z "$TARGET_DIR" ]]; then
  TARGET_TREE_ROOT="$(dirname "$TARGET_ROOT")"
  prune_target_tree "$TARGET_TREE_ROOT"
fi

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

if [[ -n "$TARGET_DIR" ]]; then
  CARGO_FLAGS+=(--target-dir "$TARGET_DIR")
elif [[ -n "$TARGET_ROOT" ]]; then
  CARGO_FLAGS+=(--target-dir "$TARGET_ROOT")
fi

if [[ ${#PACKAGES[@]} -gt 0 ]]; then
  for package in "${PACKAGES[@]}"; do
    CARGO_FLAGS+=(--package "$package")
  done
fi

if [[ ${#TEST_FLAGS[@]} -gt 0 ]]; then
  cargo "${CARGO_FLAGS[@]}" -- "${TEST_FLAGS[@]}"
else
  cargo "${CARGO_FLAGS[@]}"
fi

if [[ -n "${TARGET_TREE_ROOT:-}" ]]; then
  prune_target_tree "$TARGET_TREE_ROOT"
fi

if [[ -n "$TARGET_DIR" ]]; then
  report_target_dir_size "$TARGET_DIR"
elif [[ -n "$TARGET_ROOT" ]]; then
  report_target_dir_size "$TARGET_ROOT"
fi
