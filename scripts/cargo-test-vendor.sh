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

if [[ "${RUST_LOG:-}" == *trace* ]]; then
  export RUST_LOG="${RUST_LOG:+$RUST_LOG,}ureq=trace"
else
  export RUST_LOG="${RUST_LOG:+$RUST_LOG,}ureq=off"
fi

TEST_FLAGS=()
QUIET=false
RELEASE=false
LOCKED=false
OFFLINE=false
TARGET_DIR=""
CLEAN_TARGET_TMPDIR=false
TARGET_ROOT=""
TARGET_TREE_ROOT=""
PRUNE_LIMIT_SPEC="20G"
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
    export TMPDIR="/var/tmp/cargo/test/vendor/${TMPDIR_VALUE}"
    export TMP="${TMPDIR}/${TMP_VALUE}"
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
  --prune-limit VALUE  Prune oldest dirs once the tree reaches VALUE (default 20G)
  --ignored            Pass --ignored to cargo test
  --nocapture          Pass --nocapture to cargo test
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
      "")
        echo "$value"
        ;;
      K)
        echo "$value"
        ;;
      M)
        echo $((value * 1024))
        ;;
      G)
        echo $((value * 1024 * 1024))
        ;;
      *)
        return 1
        ;;
    esac
  else
    return 1
  fi
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

  if [[ -n "$TARGET_DIR" ]]; then
    TARGET_TREE_ROOT="$(dirname "$TARGET_DIR")"
  elif [[ -n "$TARGET_ROOT" ]]; then
    TARGET_TREE_ROOT="$(dirname "$TARGET_ROOT")"
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

if [[ -n "$TARGET_TREE_ROOT" ]]; then
  prune_target_tree "$TARGET_TREE_ROOT"
  report_target_dir_size "$TARGET_TREE_ROOT"
fi
