#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

bash ./scripts/asyncgit-tests.sh

cargo test -p gnostr-asyncgit --lib types::event_kind::test::test_replaceable_ephemeral -- --nocapture
cargo test -p gnostr-asyncgit --lib types::naddr::test::test_short_tlv_errors_instead_of_panicking -- --nocapture
cargo test -p gnostr-asyncgit --lib types::nevent::test::test_short_tlv_errors_instead_of_panicking -- --nocapture
cargo test -p gnostr-asyncgit --lib types::nip19::tests::test_short_tlv_errors_instead_of_panicking -- --nocapture
