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
TARGET_DIR=""
TARGET_TMPDIR=false
TARGET_TMPDIR_CLEAN=false
QUIET=false
RELEASE=false
LOCKED=false
OFFLINE=false

usage() {
  cat <<'EOF'
Usage: gnostr-asyncgit-tests.sh [--quiet] [--release] [--locked] [--offline] [--target-dir VALUE] [--target-tmpdir] [--target-tmpdir-clean] [--ignored] [--nocapture]

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
  --help               Show this help
EOF
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
  export TMPDIR="/var/tmp/${TMPDIR_VALUE}/cargo/test/asyncgit/${TMPDIR_VALUE}"
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
fi

run_cargo() {
  local cmd="$1"
  shift

  if [[ -n "$TARGET_DIR" ]]; then
    cargo "${CARGO_COMMON_FLAGS[@]}" "$cmd" "${CARGO_SUBCOMMAND_FLAGS[@]}" --target-dir "$TARGET_DIR" "$@"
  else
    cargo "${CARGO_COMMON_FLAGS[@]}" "$cmd" "${CARGO_SUBCOMMAND_FLAGS[@]}" "$@"
  fi
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
  if [[ ${#TEST_FLAGS[@]} -gt 0 ]]; then
    if run_cargo "$@" -- "${TEST_FLAGS[@]}"; then
      send_chat_update "$test_name successful"
    else
      send_chat_update "$test_name fail"
      return 1
    fi
  else
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
  if [[ ${#TEST_FLAGS[@]} -gt 0 ]]; then
    if output="$(run_cargo "$@" -- "${TEST_FLAGS[@]}" 2>&1)"; then
      printf '%s\n' "$output"
      send_chat_update "$test_name successful"
    else
      local status=$?
      printf '%s\n' "$output" >&2
      send_chat_update "$test_name fail"
      return "$status"
    fi
  else
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

if bash ./scripts/asyncgit-tests.sh; then
  send_chat_update "asyncgit bootstrap successful"
else
  send_chat_update "asyncgit bootstrap fail"
  exit 1
fi

run_cargo_test_step "asyncgit event_kind test" test -p gnostr-asyncgit --lib types::event_kind::test::test_replaceable_ephemeral
run_cargo_test_step "asyncgit naddr tlv test" test -p gnostr-asyncgit --lib types::naddr::test::test_short_tlv_errors_instead_of_panicking
run_cargo_test_step "asyncgit nevent tlv test" test -p gnostr-asyncgit --lib types::nevent::test::test_short_tlv_errors_instead_of_panicking
run_cargo_test_step "asyncgit nip19 tlv test" test -p gnostr-asyncgit --lib types::nip19::tests::test_short_tlv_errors_instead_of_panicking
run_cargo_test_step "asyncgit nip44 long message test" test -p gnostr-asyncgit --lib types::nip44::tests::test_valid_encrypt_decrypt_long_msg
run_cargo_test_step "asyncgit nip44 invalid lengths test" test -p gnostr-asyncgit --lib types::nip44::tests::test_invalid_encrypt_msg_lengths
run_cargo_test_step "asyncgit nip34 root commit test" test -p gnostr-asyncgit --lib types::nip34::tests::repo_ref_defaults_identifier_from_root_commit
run_cargo_test_step "asyncgit nip34 maintainer fanout test" test -p gnostr-asyncgit --lib types::nip34::tests::repo_ref_coordinates_include_relay_hint_and_all_maintainers
run_cargo_test_step "asyncgit nip34 repo url vector test" test -p gnostr-asyncgit --lib types::nip34::tests::repo_url_vector_matches_ngit_coordinate
run_cargo_test_step "asyncgit nip34 npub tag test" test -p gnostr-asyncgit --lib types::nip34::tests::event_tag_from_nip19_or_hex_accepts_npub_when_allowed
NIP4_TEST_OUTPUT="$(run_cargo_capture_step "asyncgit nip4 dm roundtrip test" test -p gnostr-asyncgit --lib types::nip4::tests::encrypt_and_decrypt_real_dm_events_in_both_directions)"
printf '%s\n' "$NIP4_TEST_OUTPUT"
OUTBOUND_DM_EVENT_ID="$(
  printf '%s\n' "$NIP4_TEST_OUTPUT" | awk -F': ' '/outbound dm event id:/ { print $2; exit }'
)"
RETURN_DM_EVENT_ID="$(
  printf '%s\n' "$NIP4_TEST_OUTPUT" | awk -F': ' '/return dm event id:/ { print $2; exit }'
)"
if [[ -z "$OUTBOUND_DM_EVENT_ID" || -z "$RETURN_DM_EVENT_ID" ]]; then
  echo "Failed to extract DM event ids from nip4 test output" >&2
  send_chat_update "asyncgit nip4 dm roundtrip test fail"
  exit 1
fi

if run_cargo run --bin gnostr -- query -i "$OUTBOUND_DM_EVENT_ID" | grep -F '["EOSE","gnostr-query"]'; then
  send_chat_update "asyncgit nip4 outbound query successful"
else
  send_chat_update "asyncgit nip4 outbound query fail"
  exit 1
fi

if run_cargo run --bin gnostr -- query -i "$RETURN_DM_EVENT_ID" | grep -F '["EOSE","gnostr-query"]'; then
  send_chat_update "asyncgit nip4 return query successful"
else
  send_chat_update "asyncgit nip4 return query fail"
  exit 1
fi

CLIENT_TEST_OUTPUT="$(run_cargo_capture_step "asyncgit client direct message test" test -p gnostr-asyncgit --lib types::client::tests::build_nip44_direct_message_event_uses_real_keys_and_recipient_tag)"
printf '%s\n' "$CLIENT_TEST_OUTPUT"
CLIENT_DM_EVENT_ID="$(
  printf '%s\n' "$CLIENT_TEST_OUTPUT" | awk -F': ' '/direct message event id:/ { print $2; exit }'
)"
if [[ -z "$CLIENT_DM_EVENT_ID" ]]; then
  echo "Failed to extract direct message event id from client test output" >&2
  send_chat_update "asyncgit client direct message test fail"
  exit 1
fi

report_target_dir_size "$TARGET_DIR"

if run_cargo run --bin gnostr -- query -i "$CLIENT_DM_EVENT_ID" | grep -F '["EOSE","gnostr-query"]'; then
  send_chat_update "asyncgit client direct message query successful"
else
  send_chat_update "asyncgit client direct message query fail"
  exit 1
fi
