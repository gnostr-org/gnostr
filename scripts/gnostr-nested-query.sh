#!/usr/bin/env bash
set -euo pipefail

BASH_VERSION_CURRENT="${BASH_VERSION:-unknown}"
BASH_MAJOR="${BASH_VERSINFO[0]:-0}"
BASH_MINOR="${BASH_VERSINFO[1]:-0}"
if ! bash -n "${BASH_SOURCE[0]}"; then
  exit 1
fi

GNOSTR_BIN="${GNOSTR_BIN:-${GNOSTR:-gnostr}}"
NOSTR_PUBKEY="${NOSTR_PUBKEY:-npub15d9enu3v0yxyud4jk0pvxk3kmvrzymjpc6f0eq4ck44vr32qck7smrxq6k}"
QUERY_LIMIT="${QUERY_LIMIT:-1}"
QUERY_RELAYS="${QUERY_RELAYS:-}"

if ! command -v "$GNOSTR_BIN" >/dev/null 2>&1; then
  echo "gnostr binary not found: $GNOSTR_BIN" >&2
  exit 1
fi

ids="$("$GNOSTR_BIN" bech32-to-any "$NOSTR_PUBKEY" --raw)"
if [[ -z "$ids" ]]; then
  echo "failed to derive ids from $NOSTR_PUBKEY" >&2
  exit 1
fi

query_args=(query --ids "$ids" --limit "$QUERY_LIMIT")
if [[ -n "$QUERY_RELAYS" ]]; then
  IFS=',' read -r -a relay_list <<< "$QUERY_RELAYS"
  for relay in "${relay_list[@]}"; do
    [[ -z "$relay" ]] && continue
    query_args+=(-r "$relay")
  done
fi

"$GNOSTR_BIN" "${query_args[@]}"
