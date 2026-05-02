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
while [[ $# -gt 0 ]]; do
  case "$1" in
    --nocapture)
      TEST_FLAGS+=(--nocapture)
      ;;
    --ignored)
      TEST_FLAGS+=(--ignored)
      ;;
    *)
      echo "Unsupported flag: $1" >&2
      exit 1
      ;;
  esac
  shift
done

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
  cargo run --bin gnostr -- chat --topic gnostr-dev --name copilot --oneshot "$test_name" >/dev/null 2>&1 || true
}

run_cargo_test_step() {
  local test_name="$1"
  shift
  if "$@"; then
    send_chat_update "$test_name successful"
  else
    send_chat_update "$test_name fail"
    return 1
  fi
}

run_cargo_capture_step() {
  local test_name="$1"
  shift
  local output
  if output="$("$@" 2>&1)"; then
    printf '%s\n' "$output"
    send_chat_update "$test_name successful"
  else
    local status=$?
    printf '%s\n' "$output" >&2
    send_chat_update "$test_name fail"
    return "$status"
  fi
}

if bash ./scripts/asyncgit-tests.sh "$@"; then
  send_chat_update "asyncgit bootstrap successful"
else
  send_chat_update "asyncgit bootstrap fail"
  exit 1
fi

run_cargo_test_step "asyncgit event_kind test" cargo test -p gnostr-asyncgit --lib types::event_kind::test::test_replaceable_ephemeral -- "${TEST_FLAGS[@]}"
run_cargo_test_step "asyncgit naddr tlv test" cargo test -p gnostr-asyncgit --lib types::naddr::test::test_short_tlv_errors_instead_of_panicking -- "${TEST_FLAGS[@]}"
run_cargo_test_step "asyncgit nevent tlv test" cargo test -p gnostr-asyncgit --lib types::nevent::test::test_short_tlv_errors_instead_of_panicking -- "${TEST_FLAGS[@]}"
run_cargo_test_step "asyncgit nip19 tlv test" cargo test -p gnostr-asyncgit --lib types::nip19::tests::test_short_tlv_errors_instead_of_panicking -- "${TEST_FLAGS[@]}"
run_cargo_test_step "asyncgit nip44 long message test" cargo test -p gnostr-asyncgit --lib types::nip44::tests::test_valid_encrypt_decrypt_long_msg -- "${TEST_FLAGS[@]}"
run_cargo_test_step "asyncgit nip44 invalid lengths test" cargo test -p gnostr-asyncgit --lib types::nip44::tests::test_invalid_encrypt_msg_lengths -- "${TEST_FLAGS[@]}"
run_cargo_test_step "asyncgit nip34 root commit test" cargo test -p gnostr-asyncgit --lib types::nip34::tests::repo_ref_defaults_identifier_from_root_commit -- "${TEST_FLAGS[@]}"
run_cargo_test_step "asyncgit nip34 maintainer fanout test" cargo test -p gnostr-asyncgit --lib types::nip34::tests::repo_ref_coordinates_include_relay_hint_and_all_maintainers -- "${TEST_FLAGS[@]}"
run_cargo_test_step "asyncgit nip34 repo url vector test" cargo test -p gnostr-asyncgit --lib types::nip34::tests::repo_url_vector_matches_ngit_coordinate -- "${TEST_FLAGS[@]}"
run_cargo_test_step "asyncgit nip34 npub tag test" cargo test -p gnostr-asyncgit --lib types::nip34::tests::event_tag_from_nip19_or_hex_accepts_npub_when_allowed -- "${TEST_FLAGS[@]}"
NIP4_TEST_OUTPUT="$(run_cargo_capture_step "asyncgit nip4 dm roundtrip test" cargo test -p gnostr-asyncgit --lib types::nip4::tests::encrypt_and_decrypt_real_dm_events_in_both_directions -- "${TEST_FLAGS[@]}")"
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

if cargo run --bin gnostr -- query -i "$OUTBOUND_DM_EVENT_ID" | grep -F '["EOSE","gnostr-query"]'; then
  send_chat_update "asyncgit nip4 outbound query successful"
else
  send_chat_update "asyncgit nip4 outbound query fail"
  exit 1
fi

if cargo run --bin gnostr -- query -i "$RETURN_DM_EVENT_ID" | grep -F '["EOSE","gnostr-query"]'; then
  send_chat_update "asyncgit nip4 return query successful"
else
  send_chat_update "asyncgit nip4 return query fail"
  exit 1
fi

CLIENT_TEST_OUTPUT="$(run_cargo_capture_step "asyncgit client direct message test" cargo test -p gnostr-asyncgit --lib types::client::tests::build_nip44_direct_message_event_uses_real_keys_and_recipient_tag -- "${TEST_FLAGS[@]}")"
printf '%s\n' "$CLIENT_TEST_OUTPUT"
CLIENT_DM_EVENT_ID="$(
  printf '%s\n' "$CLIENT_TEST_OUTPUT" | awk -F': ' '/direct message event id:/ { print $2; exit }'
)"
if [[ -z "$CLIENT_DM_EVENT_ID" ]]; then
  echo "Failed to extract direct message event id from client test output" >&2
  send_chat_update "asyncgit client direct message test fail"
  exit 1
fi

if cargo run --bin gnostr -- query -i "$CLIENT_DM_EVENT_ID" | grep -F '["EOSE","gnostr-query"]'; then
  send_chat_update "asyncgit client direct message query successful"
else
  send_chat_update "asyncgit client direct message query fail"
  exit 1
fi
