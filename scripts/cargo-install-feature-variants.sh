#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

FEATURES=()
RUN_ALLOW_ALL=true
RUN_NO_DEFAULT_FEATURES=true

usage() {
  cat <<'EOF'
Usage: cargo-install-feature-variants.sh [--features VALUE] [--allow-all|--all-features] [--no-default-features]

By default, runs cargo install for:
  - the default feature set
  - --all-features
  - --no-default-features

Options:
  --features VALUE       Add a Cargo feature (repeatable)
  --features=VALUE       Add a Cargo feature
  --allow-all            Keep the --all-features install variant enabled
  --all-features         Alias for --allow-all
  --no-default-features  Keep the --no-default-features install variant enabled
  --help                 Show this help
EOF
}

join_features() {
  local IFS=,
  echo "${FEATURES[*]}"
}

run_install() {
  local label="$1"
  shift

  local build_dir
  build_dir="$(mktemp -d "${TMPDIR:-/tmp}/gnostr-install-${label//[^A-Za-z0-9]/_}.XXXXXX")"
  printf '==> %s\n' "$label"
  CARGO_TARGET_DIR="$build_dir" cargo install --path . --locked --force "$@"
  rm -rf "$build_dir"
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --features)
      shift
      [[ $# -gt 0 ]] || { echo "--features requires a value" >&2; exit 1; }
      FEATURES+=("$1")
      ;;
    --features=*)
      FEATURES+=("${1#*=}")
      ;;
    --allow-all|--all-features)
      RUN_ALLOW_ALL=true
      ;;
    --no-default-features)
      RUN_NO_DEFAULT_FEATURES=true
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

declare -a FEATURE_ARGS=()
if [[ ${#FEATURES[@]} -gt 0 ]]; then
  FEATURE_ARGS=(--features "$(join_features)")
fi

if [[ ${#FEATURE_ARGS[@]} -gt 0 ]]; then
  run_install "default install" "${FEATURE_ARGS[@]}"
else
  run_install "default install"
fi

if [[ "$RUN_ALLOW_ALL" == true ]]; then
  run_install "allow-all install" --all-features
fi

if [[ "$RUN_NO_DEFAULT_FEATURES" == true ]]; then
  if [[ ${#FEATURE_ARGS[@]} -gt 0 ]]; then
    run_install "no-default-features install" --no-default-features "${FEATURE_ARGS[@]}"
  else
    run_install "no-default-features install" --no-default-features
  fi
fi
