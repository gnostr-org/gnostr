#!/usr/bin/env bash
set -euo pipefail

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

bash ./scripts/asyncgit-tests.sh

cargo test -p gnostr-asyncgit --lib types::event_kind::test::test_replaceable_ephemeral --  "${TEST_FLAGS[@]}"
cargo test -p gnostr-asyncgit --lib types::naddr::test::test_short_tlv_errors_instead_of_panicking --  "${TEST_FLAGS[@]}"
cargo test -p gnostr-asyncgit --lib types::nevent::test::test_short_tlv_errors_instead_of_panicking --  "${TEST_FLAGS[@]}"
cargo test -p gnostr-asyncgit --lib types::nip19::tests::test_short_tlv_errors_instead_of_panicking --  "${TEST_FLAGS[@]}"
cargo test -p gnostr-asyncgit --lib types::nip44::tests::test_valid_encrypt_decrypt_long_msg --   "${TEST_FLAGS[@]}"
cargo test -p gnostr-asyncgit --lib types::nip44::tests::test_invalid_encrypt_msg_lengths --   "${TEST_FLAGS[@]}"
cargo test -p gnostr-asyncgit --lib types::nip4::tests::encrypt_and_decrypt_real_dm_events_in_both_directions --  "${TEST_FLAGS[@]}"
