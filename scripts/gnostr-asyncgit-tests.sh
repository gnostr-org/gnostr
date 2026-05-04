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

export RUST_LOG="${RUST_LOG:+$RUST_LOG,}ureq=off,serial_test=off,mio=off,tungstenite=off,tokio_tungstenite=off"

TEST_FLAGS=()
TARGET_DIR=""
TARGET_TMPDIR=false
TARGET_TMPDIR_CLEAN=false
TARGET_TREE_ROOT=""
PRUNE_LIMIT_SPEC="20G"
QUIET=false
RELEASE=false
LOCKED=false
OFFLINE=false
MODE="full"

usage() {
  cat <<'EOF'
Usage: gnostr-asyncgit-tests.sh <nip34|matrix|full|list> [flags]

Commands:
  nip34   Run the NIP-34 note tests
  matrix  Run only the PoW matrix test
  full    Run nip34 plus the full asyncgit suite
  list    Print the exact runnable commands and exit

Flags:
  --quiet               Pass --quiet to cargo test
  --release             Pass --release to cargo test
  --locked              Pass --locked to cargo test
  --offline             Pass --offline to cargo test
  --target-dir VALUE    Set Cargo's target directory
  --target-tmpdir       Use the shared asyncgit temp directory
  --target-tmpdir-clean Remove the shared asyncgit temp directory first
  --prune-limit VALUE   Prune oldest dirs once the tree reaches VALUE (default 20G)
  --ignored             Pass --ignored to cargo test
  --nocapture           Pass --nocapture to cargo test
  --help                Show this help

Notes:
  `nip34` runs:
    cargo test -p gnostr-asyncgit --lib repo_announcement_event_matches_ngit -- --nocapture
    cargo test -p gnostr-asyncgit --lib repo_state_parsing_matches_ngit -- --nocapture
    cargo test -p gnostr-asyncgit --lib repo_state_round_trip_adds_head -- --nocapture
    cargo test -p gnostr-asyncgit --lib generate_git_note_event_uses_the_note_message -- --nocapture
    cargo test -p gnostr-asyncgit --lib generate_git_note_event_with_pow_adds_nonce -- --nocapture
    cargo test -p gnostr-asyncgit --lib git_note_event_matrix_covers_commit_and_pow_variants -- --nocapture
    cargo test -p gnostr-ngit --test nip34_kinds -- --nocapture
    cargo test -p gnostr-ngit --test git_notes -- --nocapture
  `matrix` runs:
    cargo test -p gnostr-asyncgit --lib git_note_event_matrix_covers_commit_and_pow_variants -- --nocapture
  `full` runs notes plus:
    cargo test -p gnostr-asyncgit --all-targets --features nostr -- --nocapture
  `nostr_sdk` is only used from asyncgit test code.
  ureq logging is always silenced here.

Examples:
  ./scripts/gnostr-asyncgit-tests.sh nip34 --nocapture
  ./scripts/gnostr-asyncgit-tests.sh matrix --nocapture
  ./scripts/gnostr-asyncgit-tests.sh full --nocapture
  ./scripts/gnostr-asyncgit-tests.sh list
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
    nip34|matrix|full|list)
      MODE="$1"
      ;;
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
    --nip34|--notes)
      MODE="nip34"
      ;;
    --matrix)
      MODE="matrix"
      ;;
    --full)
      MODE="full"
      ;;
    --list)
      MODE="list"
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
    --target-tmpdir-clean|--target_tmpdir-clean|--target_tmpdir_clean)
      TARGET_TMPDIR=true
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

TARGET_SIZE_LIMIT_KIB="$(parse_prune_limit_kib "$PRUNE_LIMIT_SPEC")" || {
  echo "Invalid --prune-limit value: $PRUNE_LIMIT_SPEC" >&2
  exit 1
}

CARGO_COMMON_FLAGS=()
CARGO_SUBCOMMAND_FLAGS=()

if [[ "$QUIET" == true ]]; then
  CARGO_COMMON_FLAGS+=(--quiet)
fi

if [[ "$RELEASE" == true ]]; then
  CARGO_SUBCOMMAND_FLAGS+=(--release)
fi

if [[ "$LOCKED" == true ]]; then
  CARGO_COMMON_FLAGS+=(--locked)
fi

if [[ "$OFFLINE" == true ]]; then
  CARGO_COMMON_FLAGS+=(--offline)
fi

TARGET_ROOT=""
if [[ "$TARGET_TMPDIR" == true ]]; then
  TMPDIR_VALUE="$(gnostr --weeble 2>/dev/null || true)"
  TMP_VALUE="$(gnostr --blockheight 2>/dev/null || true)"
  TEMP_VALUE="$(gnostr --wobble 2>/dev/null || true)"
  TMPDIR_VALUE="${TMPDIR_VALUE:-0}"
  TMP_VALUE="${TMP_VALUE:-0}"
  TEMP_VALUE="${TEMP_VALUE:-0}"
  export TMPDIR="/var/tmp/cargo/test/asyncgit/${TMPDIR_VALUE}"
  export TMP="${TMPDIR}/${TMP_VALUE}"
  export TEMP="${TMP}/debug/${TEMP_VALUE}"
  TARGET_ROOT="$TEMP"
  if [[ "$TARGET_TMPDIR_CLEAN" == true && -d "$TARGET_ROOT" ]]; then
    rm -rf "$TARGET_ROOT"
  fi
  mkdir -p "$TARGET_ROOT"
  if [[ -z "$TARGET_DIR" ]]; then
    TARGET_DIR="$TARGET_ROOT"
  fi
  TARGET_TREE_ROOT="$(dirname "$TARGET_DIR")"
  prune_target_tree "$TARGET_TREE_ROOT"
fi

run_cargo() {
  local cmd="$1"
  shift
  local -a argv=(cargo)
  # Build argv incrementally so empty arrays never trip `set -u`.
  if [[ ${#CARGO_COMMON_FLAGS[@]} -gt 0 ]]; then
    argv+=("${CARGO_COMMON_FLAGS[@]}")
  fi
  argv+=("$cmd")
  if [[ ${#CARGO_SUBCOMMAND_FLAGS[@]} -gt 0 ]]; then
    argv+=("${CARGO_SUBCOMMAND_FLAGS[@]}")
  fi

  if [[ -n "$TARGET_DIR" ]]; then
    argv+=(--target-dir "$TARGET_DIR")
  fi
  argv+=("$@")
  "${argv[@]}"
}

print_cargo_command() {
  local cmd="$1"
  shift
  local -a argv=(cargo)
  # Mirror the exact cargo invocation before we run it.
  if [[ ${#CARGO_COMMON_FLAGS[@]} -gt 0 ]]; then
    argv+=("${CARGO_COMMON_FLAGS[@]}")
  fi
  argv+=("$cmd")
  if [[ ${#CARGO_SUBCOMMAND_FLAGS[@]} -gt 0 ]]; then
    argv+=("${CARGO_SUBCOMMAND_FLAGS[@]}")
  fi
  if [[ -n "$TARGET_DIR" ]]; then
    argv+=(--target-dir "$TARGET_DIR")
  fi
  argv+=("$@")
  printf '+'
  printf ' %q' "${argv[@]}"
  printf '\n'
}

VECTOR_FILE="./asyncgit/src/lib/types/nip44/nip44.vectors.json"
EXPECTED_VECTOR_SHA256="269ed0f69e4c192512cc779e78c555090cebc7c785b609e338a62afc3ce25040"
if command -v shasum >/dev/null 2>&1; then
  ACTUAL_VECTOR_SHA256="$(shasum -a 256 "$VECTOR_FILE" | awk '{print $1}')"
elif command -v sha256sum >/dev/null 2>&1; then
  ACTUAL_VECTOR_SHA256="$(sha256sum "$VECTOR_FILE" | awk '{print $1}')"
else
  echo "No SHA-256 tool found (shasum or sha256sum)" >&2
  exit 1
fi

if [[ "$ACTUAL_VECTOR_SHA256" != "$EXPECTED_VECTOR_SHA256" ]]; then
  echo "nip44 vector hash mismatch: expected $EXPECTED_VECTOR_SHA256, got $ACTUAL_VECTOR_SHA256" >&2
  exit 1
fi

send_chat_update() {
  local test_name="$1"
  run_cargo run --bin gnostr -- chat --topic gnostr-dev --name copilot --oneshot "$test_name" >/dev/null 2>&1 || true
}

run_cargo_test_step() {
  local test_name="$1"
  shift
  local -a test_flags=("${TEST_FLAGS[@]}")
  if [[ ! " ${test_flags[*]} " =~ " --nocapture " ]]; then
    test_flags+=(--nocapture)
  fi
  if [[ ${#test_flags[@]} -gt 0 ]]; then
    print_cargo_command "$@" -- "${test_flags[@]}"
    if run_cargo "$@" -- "${test_flags[@]}"; then
      send_chat_update "$test_name successful"
    else
      send_chat_update "$test_name fail"
      return 1
    fi
  else
    print_cargo_command "$@"
    if run_cargo "$@"; then
      send_chat_update "$test_name successful"
    else
      send_chat_update "$test_name fail"
      return 1
    fi
  fi
}

run_cargo_capture_step() {
  local test_name="$1"
  shift
  local output
  local -a test_flags=("${TEST_FLAGS[@]}")
  if [[ ! " ${test_flags[*]} " =~ " --nocapture " ]]; then
    test_flags+=(--nocapture)
  fi
  if [[ ${#test_flags[@]} -gt 0 ]]; then
    print_cargo_command "$@" -- "${test_flags[@]}"
    if output="$(run_cargo "$@" -- "${test_flags[@]}" 2>&1)"; then
      printf '%s\n' "$output"
      send_chat_update "$test_name successful"
    else
      local status=$?
      printf '%s\n' "$output" >&2
      send_chat_update "$test_name fail"
      return "$status"
    fi
  else
    print_cargo_command "$@"
    if output="$(run_cargo "$@" 2>&1)"; then
      printf '%s\n' "$output"
      send_chat_update "$test_name successful"
    else
      local status=$?
      printf '%s\n' "$output" >&2
      send_chat_update "$test_name fail"
      return "$status"
    fi
  fi
}

run_nip34_suite() {
  # Keep the note-related cases together so `nip34` stays predictable.
  run_cargo_test_step "asyncgit nip34 repo announcement" test -p gnostr-asyncgit --lib repo_announcement_event_matches_ngit
  run_cargo_test_step "asyncgit nip34 repo state" test -p gnostr-asyncgit --lib repo_state_parsing_matches_ngit
  run_cargo_test_step "asyncgit nip34 repo state round trip" test -p gnostr-asyncgit --lib repo_state_round_trip_adds_head
  run_cargo_test_step "asyncgit nip34 note message" test -p gnostr-asyncgit --lib generate_git_note_event_uses_the_note_message
  run_cargo_test_step "asyncgit nip34 pow nonce" test -p gnostr-asyncgit --lib generate_git_note_event_with_pow_adds_nonce
  run_cargo_test_step "asyncgit nip34 matrix" test -p gnostr-asyncgit --lib git_note_event_matrix_covers_commit_and_pow_variants
  run_cargo_test_step "ngit nip34 kinds" test -p gnostr-ngit --test nip34_kinds
  run_cargo_test_step "ngit git notes" test -p gnostr-ngit --test git_notes
}

list_tests() {
  # Print runnable commands, not just labels, so the matrix is copy-pasteable.
  printf '%s\n' \
    "./scripts/gnostr-asyncgit-tests.sh matrix --nocapture" \
    "  cargo test -p gnostr-asyncgit --lib git_note_event_matrix_covers_commit_and_pow_variants -- --nocapture" \
    "./scripts/gnostr-asyncgit-tests.sh nip34 --nocapture" \
    "  cargo test -p gnostr-asyncgit --lib repo_announcement_event_matches_ngit -- --nocapture" \
    "  cargo test -p gnostr-asyncgit --lib repo_state_parsing_matches_ngit -- --nocapture" \
    "  cargo test -p gnostr-asyncgit --lib repo_state_round_trip_adds_head -- --nocapture" \
    "  cargo test -p gnostr-asyncgit --lib generate_git_note_event_uses_the_note_message -- --nocapture" \
    "  cargo test -p gnostr-asyncgit --lib generate_git_note_event_with_pow_adds_nonce -- --nocapture" \
    "  cargo test -p gnostr-asyncgit --lib git_note_event_matrix_covers_commit_and_pow_variants -- --nocapture" \
    "  cargo test -p gnostr-ngit --test nip34_kinds -- --nocapture" \
    "  cargo test -p gnostr-ngit --test git_notes -- --nocapture"
  if [[ "$MODE" != "nip34" ]]; then
    printf '%s\n' \
      "./scripts/gnostr-asyncgit-tests.sh full --nocapture" \
      "  cargo test -p gnostr-asyncgit --all-targets --features nostr -- --nocapture"
  fi
}

if [[ "$MODE" == "list" ]]; then
  list_tests
  exit 0
fi

case "$MODE" in
  nip34)
    run_nip34_suite
    ;;
  matrix)
    run_cargo_test_step "asyncgit nip34 matrix" test -p gnostr-asyncgit --lib git_note_event_matrix_covers_commit_and_pow_variants
    ;;
  full)
    if bash ./scripts/asyncgit-tests.sh; then
      send_chat_update "asyncgit bootstrap successful"
    else
      send_chat_update "asyncgit bootstrap fail"
      exit 1
    fi
    run_nip34_suite
    run_cargo_test_step "asyncgit full test suite" test -p gnostr-asyncgit --all-targets --features nostr
    ;;
  *)
    echo "Unsupported mode: $MODE" >&2
    exit 1
    ;;
esac

report_target_dir_size "$TARGET_DIR"
if [[ -n "$TARGET_TREE_ROOT" ]]; then
  prune_target_tree "$TARGET_TREE_ROOT"
fi
