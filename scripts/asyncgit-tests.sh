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
TARGET_TMPDIR=false
TARGET_TMPDIR_CLEAN=false

usage() {
  cat <<'EOF'
Usage: asyncgit-tests.sh [--quiet] [--release] [--locked] [--offline] [--target-dir VALUE] [--target-tmpdir] [--target-tmpdir-clean] [--ignored] [--nocapture] [--test-threads VALUE]

Options:
  --quiet              Pass --quiet to cargo test
  --release            Pass --release to cargo test
  --locked             Pass --locked to cargo test
  --offline            Pass --offline to cargo test
  --target-dir VALUE   Set Cargo's target directory
  --target-tmpdir      Use the shared asyncgit temp directory
  --target-tmpdir-clean Remove the shared asyncgit temp directory first
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
      TARGET_TMPDIR=true
      ;;
    --target-tmpdir-clean|--target_tmpdir-clean|--target_tmpdir_clean)
      TARGET_TMPDIR=true
      TARGET_TMPDIR_CLEAN=true
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

TARGET_ROOT=""
if [[ "$TARGET_TMPDIR" == true ]]; then
  TMPDIR_VALUE="$(gnostr --weeble 2>/dev/null || true)"
  TMP_VALUE="$(gnostr --blockheight 2>/dev/null || true)"
  TEMP_VALUE="$(gnostr --wobble 2>/dev/null || true)"
  TMPDIR_VALUE="${TMPDIR_VALUE:-0}"
  TMP_VALUE="${TMP_VALUE:-0}"
  TEMP_VALUE="${TEMP_VALUE:-0}"
  TARGET_ROOT="/var/tmp/1876/cargo/test/asyncgit/${TMPDIR_VALUE}"
  export TMPDIR="$TARGET_ROOT"
  export TMP="$TARGET_ROOT/${TMP_VALUE}"
  export TEMP="$TMP/debug/${TEMP_VALUE}"
  if [[ "$TARGET_TMPDIR_CLEAN" == true && -d "$TARGET_ROOT" ]]; then
    rm -rf "$TARGET_ROOT"
  fi
  mkdir -p "$TARGET_ROOT"
  if [[ -z "$TARGET_DIR" ]]; then
    TARGET_DIR="$TARGET_ROOT"
  fi
fi

declare -a CARGO_FLAGS=(test -p gnostr-asyncgit --lib)

if [[ "$QUIET" == true ]]; then
  CARGO_FLAGS+=(--quiet)
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
fi

run_cargo_test_step() {
  local test_name="$1"
  local cargo_cmd=(cargo "${CARGO_FLAGS[@]}" "$test_name")
  if [[ ${#TEST_FLAGS[@]} -gt 0 ]]; then
    "${cargo_cmd[@]}" -- "${TEST_FLAGS[@]}"
  else
    "${cargo_cmd[@]}"
  fi
}

run_cargo_test_step "types::event_kind::test::test_replaceable_ephemeral"
run_cargo_test_step "types::naddr::test::test_short_tlv_errors_instead_of_panicking"
run_cargo_test_step "types::nevent::test::test_short_tlv_errors_instead_of_panicking"
run_cargo_test_step "types::nip19::tests::test_short_tlv_errors_instead_of_panicking"
run_cargo_test_step "types::nip44::tests::test_valid_encrypt_decrypt_long_msg"
run_cargo_test_step "types::nip44::tests::test_invalid_encrypt_msg_lengths"
run_cargo_test_step "types::nip34::tests::repo_ref_defaults_identifier_from_root_commit"
run_cargo_test_step "types::nip34::tests::repo_ref_coordinates_include_relay_hint_and_all_maintainers"
run_cargo_test_step "types::nip34::tests::repo_url_vector_matches_ngit_coordinate"
run_cargo_test_step "types::nip34::tests::event_tag_from_nip19_or_hex_accepts_npub_when_allowed"
run_cargo_test_step "types::nip4::tests::encrypt_and_decrypt_real_dm_events_in_both_directions"
