#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd "${SCRIPT_DIR}/.." && pwd)

PROFILE="release"
PACKAGE="gnostr"
BIN_MODE="--bins"
LOCKED=true
ALL_FEATURES=false
NO_DEFAULT_FEATURES=false
FEATURES=""
DRY_RUN=false
LIST_ONLY=false
VERBOSE=false

HOST_TRIPLE=$(rustc -vV | awk '/^host: / { print $2 }')
HOST_OS=$(uname -s)
HOST_ARCH=$(uname -m)

declare -a REQUESTED_TARGETS=()
declare -a SKIPPED_TARGETS=()

TARGET_MATRIX=(
  "linux-x64|x86_64-unknown-linux-gnu|cargo"
  "linux-x64-musl|x86_64-unknown-linux-musl|cross"
  "linux-arm64|aarch64-unknown-linux-gnu|cross"
  "linux-arm64-musl|aarch64-unknown-linux-musl|cross"
  "windows-x64|x86_64-pc-windows-msvc|cargo"
  "macos-x64|x86_64-apple-darwin|cargo"
  "macos-arm64|aarch64-apple-darwin|cargo"
)

log() {
  printf '[cross.sh] %s\n' "$*"
}

warn() {
  printf '[cross.sh] WARN: %s\n' "$*" >&2
}

die() {
  printf '[cross.sh] ERROR: %s\n' "$*" >&2
  exit 1
}

usage() {
  cat <<EOF
Usage: $0 [OPTIONS]

Build the repo across the targets this workspace already knows about from
Cross.toml and .github/workflows/build-artifact.yml.

Options:
  --target NAME|TRIPLE   Build only the named CI target or Rust target triple.
                         Repeat to select multiple targets.
  --skip NAME|TRIPLE     Skip the named CI target or Rust target triple.
                         Repeat to skip multiple targets.
  --profile NAME         Cargo profile to use (default: release).
  --package NAME         Cargo package to build (default: gnostr).
  --workspace            Build the full workspace instead of one package.
  --all-features         Enable all cargo features.
  --no-default-features  Disable default cargo features.
  --features LIST        Comma-separated cargo features to enable.
  --unlocked             Do not pass --locked.
  --verbose              Pass --verbose to cargo/cross.
  --list                 List the targets this host can attempt, then exit.
  --dry-run              Print commands without executing them.
  --help                 Show this help.

Examples:
  $0
  $0 --profile dist --target linux-arm64 --target linux-arm64-musl
  $0 --workspace --all-features --dry-run
EOF
}

normalize_target_name() {
  local value="$1"
  local entry name triple tool
  for entry in "${TARGET_MATRIX[@]}"; do
    IFS='|' read -r name triple tool <<<"$entry"
    if [[ "$value" == "$name" || "$value" == "$triple" ]]; then
      printf '%s|%s|%s\n' "$name" "$triple" "$tool"
      return 0
    fi
  done
  return 1
}

cross_available() {
  command -v cross >/dev/null 2>&1
}

host_can_attempt() {
  local triple="$1"
  local tool="$2"

  case "$triple" in
    aarch64-apple-darwin|x86_64-apple-darwin)
      [[ "$HOST_OS" == "Darwin" ]]
      ;;
    x86_64-pc-windows-msvc)
      [[ "$HOST_OS" == MINGW* || "$HOST_OS" == MSYS* || "$HOST_OS" == CYGWIN* || "$HOST_OS" == "Windows_NT" ]]
      ;;
    x86_64-unknown-linux-gnu)
      [[ "$HOST_OS" == "Linux" || "$HOST_TRIPLE" == "$triple" ]]
      ;;
    x86_64-unknown-linux-musl|aarch64-unknown-linux-gnu|aarch64-unknown-linux-musl)
      if [[ "$tool" == "cross" ]]; then
        cross_available
      else
        [[ "$HOST_TRIPLE" == "$triple" ]]
      fi
      ;;
    *)
      [[ "$HOST_TRIPLE" == "$triple" ]]
      ;;
  esac
}

ensure_rust_target() {
  local triple="$1"

  if ! rustup target list --installed | grep -qx "$triple"; then
    if [[ "$DRY_RUN" == true ]]; then
      log "would run: rustup target add $triple"
    else
      log "installing rust target: $triple"
      rustup target add "$triple"
    fi
  fi
}

build_cmd_prefix() {
  local triple="$1"
  local tool="$2"

  if [[ "$tool" == "cross" ]]; then
    printf 'cross build'
  else
    printf 'cargo build'
  fi
}

run_build() {
  local name="$1"
  local triple="$2"
  local tool="$3"
  local cmd_prefix
  local -a cmd

  cmd_prefix=$(build_cmd_prefix "$triple" "$tool")
  ensure_rust_target "$triple"

  read -r -a cmd <<<"$cmd_prefix"
  cmd+=(--target "$triple")

  if [[ "$PROFILE" == "release" ]]; then
    cmd+=(--release)
  else
    cmd+=(--profile "$PROFILE")
  fi

  if [[ "$LOCKED" == true ]]; then
    cmd+=(--locked)
  fi

  if [[ "$VERBOSE" == true ]]; then
    cmd+=(--verbose)
  fi

  if [[ "$PACKAGE" == "__workspace__" ]]; then
    cmd+=(--workspace)
  else
    cmd+=(-p "$PACKAGE")
  fi

  if [[ "$BIN_MODE" != "" ]]; then
    cmd+=("$BIN_MODE")
  fi

  if [[ "$ALL_FEATURES" == true ]]; then
    cmd+=(--all-features)
  fi

  if [[ "$NO_DEFAULT_FEATURES" == true ]]; then
    cmd+=(--no-default-features)
  fi

  if [[ -n "$FEATURES" ]]; then
    cmd+=(--features "$FEATURES")
  fi

  log "building $name ($triple) via ${cmd[0]}"
  if [[ "$DRY_RUN" == true ]]; then
    printf '  %q' "${cmd[@]}"
    printf '\n'
  else
    (
      cd "$REPO_ROOT"
      "${cmd[@]}"
    )
  fi
}

parse_args() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --target)
        [[ $# -ge 2 ]] || die "--target requires a value"
        REQUESTED_TARGETS+=("$2")
        shift 2
        ;;
      --skip)
        [[ $# -ge 2 ]] || die "--skip requires a value"
        SKIPPED_TARGETS+=("$2")
        shift 2
        ;;
      --profile)
        [[ $# -ge 2 ]] || die "--profile requires a value"
        PROFILE="$2"
        shift 2
        ;;
      --package)
        [[ $# -ge 2 ]] || die "--package requires a value"
        PACKAGE="$2"
        shift 2
        ;;
      --workspace)
        PACKAGE="__workspace__"
        shift
        ;;
      --all-features)
        ALL_FEATURES=true
        shift
        ;;
      --no-default-features)
        NO_DEFAULT_FEATURES=true
        shift
        ;;
      --features)
        [[ $# -ge 2 ]] || die "--features requires a value"
        FEATURES="$2"
        shift 2
        ;;
      --unlocked)
        LOCKED=false
        shift
        ;;
      --verbose)
        VERBOSE=true
        shift
        ;;
      --dry-run)
        DRY_RUN=true
        shift
        ;;
      --list)
        LIST_ONLY=true
        shift
        ;;
      --help|-h)
        usage
        exit 0
        ;;
      *)
        die "unknown option: $1"
        ;;
    esac
  done
}

should_skip() {
  local name="$1"
  local triple="$2"
  local skip
  for skip in "${SKIPPED_TARGETS[@]}"; do
    if [[ "$skip" == "$name" || "$skip" == "$triple" ]]; then
      return 0
    fi
  done
  return 1
}

collect_targets() {
  local entry normalized
  local -a selected=()

  if [[ ${#REQUESTED_TARGETS[@]} -eq 0 ]]; then
    for entry in "${TARGET_MATRIX[@]}"; do
      selected+=("$entry")
    done
  else
    local requested
    for requested in "${REQUESTED_TARGETS[@]}"; do
      normalized=$(normalize_target_name "$requested") || die "unknown target: $requested"
      selected+=("$normalized")
    done
  fi

  printf '%s\n' "${selected[@]}"
}

main() {
  parse_args "$@"

  log "host: $HOST_TRIPLE ($HOST_OS/$HOST_ARCH)"

  local entry name triple tool
  local attempted=0

  while IFS= read -r entry; do
    [[ -n "$entry" ]] || continue
    IFS='|' read -r name triple tool <<<"$entry"

    if should_skip "$name" "$triple"; then
      log "skipping $name ($triple): requested"
      continue
    fi

    if ! host_can_attempt "$triple" "$tool"; then
      warn "skipping $name ($triple): unsupported from this host or missing 'cross'"
      continue
    fi

    if [[ "$LIST_ONLY" == true ]]; then
      printf '%s\t%s\t%s\n' "$name" "$triple" "$tool"
      continue
    fi

    run_build "$name" "$triple" "$tool"
    attempted=$((attempted + 1))
  done < <(collect_targets)

  if [[ "$LIST_ONLY" == true ]]; then
    exit 0
  fi

  if [[ $attempted -eq 0 ]]; then
    die "no buildable targets selected for this host"
  fi

  log "completed $attempted target(s)"
}

main "$@"
