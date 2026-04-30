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
FEATURES=""
ALL_FEATURES=false
NO_DEFAULT_FEATURES=false
while [[ $# -gt 0 ]]; do
  case "$1" in
    --nocapture)
      TEST_FLAGS+=(--nocapture)
      ;;
    --ignored)
      TEST_FLAGS+=(--ignored)
      ;;
    --all-features)
      ALL_FEATURES=true
      ;;
    --no-default-features)
      NO_DEFAULT_FEATURES=true
      ;;
    --features)
      shift
      [[ $# -gt 0 ]] || { echo "--features requires a value" >&2; exit 1; }
      if [[ -n "$FEATURES" ]]; then
        FEATURES+=","
      fi
      FEATURES+="$1"
      ;;
    --features=*)
      if [[ -n "$FEATURES" ]]; then
        FEATURES+=","
      fi
      FEATURES+="${1#*=}"
      ;;
    *)
      echo "Unsupported flag: $1" >&2
      exit 1
      ;;
  esac
  shift
done

declare -a CARGO_FLAGS=(test -p gnostr-ngit --lib)
if [[ "$ALL_FEATURES" == true ]]; then
  CARGO_FLAGS+=(--all-features)
elif [[ "$NO_DEFAULT_FEATURES" == true ]]; then
  CARGO_FLAGS+=(--no-default-features)
  if [[ -n "$FEATURES" ]]; then
    CARGO_FLAGS+=(--features "$FEATURES")
  fi
else
  if [[ -n "$FEATURES" ]]; then
    CARGO_FLAGS+=(--features "$FEATURES")
  else
    CARGO_FLAGS+=(--features nostr)
  fi
fi

if [[ ${#TEST_FLAGS[@]} -gt 0 ]]; then
  cargo "${CARGO_FLAGS[@]}" -- "${TEST_FLAGS[@]}"
else
  cargo "${CARGO_FLAGS[@]}"
fi
