#!/usr/bin/env bash
set -euo pipefail

if ! bash -n "${BASH_SOURCE[0]}"; then
  exit 1
fi

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

MODE="notes"
FEATURES=""
ALL_FEATURES=false
NO_DEFAULT_FEATURES=false
RELEASE=false
NOCAPTURE=true
TEST_FLAGS=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --all)
      MODE="all"
      ;;
    --notes)
      MODE="notes"
      ;;
    --ignored)
      TEST_FLAGS+=(--ignored)
      ;;
    --nocapture)
      NOCAPTURE=true
      ;;
    --capture)
      NOCAPTURE=false
      ;;
    --all-features)
      ALL_FEATURES=true
      ;;
    --no-default-features)
      NO_DEFAULT_FEATURES=true
      ;;
    --release)
      RELEASE=true
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

build_cargo_flags() {
  local -a cargo_flags=(test -p gnostr-ngit)
  if [[ "$ALL_FEATURES" == true ]]; then
    cargo_flags+=(--all-features)
  elif [[ "$NO_DEFAULT_FEATURES" == true ]]; then
    cargo_flags+=(--no-default-features)
    if [[ -n "$FEATURES" ]]; then
      cargo_flags+=(--features "$FEATURES")
    fi
  else
    if [[ -n "$FEATURES" ]]; then
      cargo_flags+=(--features "$FEATURES")
    else
      cargo_flags+=(--features nostr)
    fi
  fi
  if [[ "$RELEASE" == true ]]; then
    cargo_flags+=(--release)
  fi
  printf '%s\n' "${cargo_flags[@]}"
}

run_cargo() {
  local -a cargo_flags=()
  cargo_flags=($(build_cargo_flags))
  cargo "${cargo_flags[@]}" "$@"
}

run_notes_suite() {
  local -a test_args=()
  if [[ "$NOCAPTURE" == true ]]; then
    test_args+=(--nocapture)
  fi
  if [[ ${#TEST_FLAGS[@]} -gt 0 ]]; then
    test_args+=("${TEST_FLAGS[@]}")
  fi

  if [[ ${#test_args[@]} -gt 0 ]]; then
    run_cargo --lib repo_state::tests::notes_refs_round_trip_through_repo_state -- "${test_args[@]}"
    run_cargo --test git_notes -- "${test_args[@]}"
  else
    run_cargo --lib repo_state::tests::notes_refs_round_trip_through_repo_state
    run_cargo --test git_notes
  fi
}

run_all_suite() {
  local -a test_args=()
  if [[ "$NOCAPTURE" == true ]]; then
    test_args+=(--nocapture)
  fi
  if [[ ${#TEST_FLAGS[@]} -gt 0 ]]; then
    test_args+=("${TEST_FLAGS[@]}")
  fi

  if [[ ${#test_args[@]} -gt 0 ]]; then
    run_cargo --lib -- "${test_args[@]}"
    run_cargo --tests -- "${test_args[@]}"
  else
    run_cargo --lib
    run_cargo --tests
  fi
}

case "$MODE" in
  notes)
    run_notes_suite
    ;;
  all)
    run_all_suite
    ;;
  *)
    echo "Unsupported mode: $MODE" >&2
    exit 1
    ;;
esac
