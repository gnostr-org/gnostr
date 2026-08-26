#!/usr/bin/env bash
set -euo pipefail

BASH_VERSION_CURRENT="${BASH_VERSION:-unknown}"
BASH_MAJOR="${BASH_VERSINFO[0]:-0}"
BASH_MINOR="${BASH_VERSINFO[1]:-0}"
if ! bash -n "${BASH_SOURCE[0]}"; then
  exit 1
fi

GNOSTR_BIN="${GNOSTR_BIN:-./target/debug/gnostr}"

EXPECTED_COMMANDS=(
  award-badge
  bech32-to-any
  broadcast-events
  chat
  convert-key
  create-badge
  create-public-channel
  crawler
  custom-event
  delete-event
  delete-profile
  dm
  fetch-by-id
  generate-keypair
  git
  hide-public-channel-message
  legit
  list-events
  mute-public-key
  ngit
  nip34
  note
  privkey-to-bech32
  profile-badges
  publish-contact-list-csv
  query
  react
  relay
  server
  send-channel-message
  set-channel-metadata
  set-metadata
  set-user-status
  sniper
  tui
  vanity
  xor
)

if [[ ! -x "$GNOSTR_BIN" ]] && ! command -v "$GNOSTR_BIN" >/dev/null 2>&1; then
  echo "gnostr binary not found: $GNOSTR_BIN" >&2
  exit 1
fi

HELP_OUTPUT="$("$GNOSTR_BIN" --help)"

mapfile -t ACTUAL_COMMANDS < <(
  printf '%s\n' "$HELP_OUTPUT" | awk '
    /^Commands:/ { in_commands=1; next }
    /^Options:/ { exit }
    in_commands && /^[[:space:]]+[a-zA-Z]/ {
      if (match($0, /^[[:space:]]+([a-zA-Z][a-zA-Z0-9_-]*)/, m)) {
        print m[1]
      }
    }
  ' | sort -u
)

missing=()
for cmd in "${EXPECTED_COMMANDS[@]}"; do
  if ! printf '%s\n' "${ACTUAL_COMMANDS[@]}" | grep -qx "$cmd"; then
    missing+=("$cmd")
  fi
done

if (( ${#missing[@]} > 0 )); then
  printf 'Missing commands: %s\n' "${missing[*]}" >&2
  exit 1
fi

printf 'Checked %d commands.\n' "${#EXPECTED_COMMANDS[@]}"
