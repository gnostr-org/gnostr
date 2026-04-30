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

NPROC="$(nproc)"
LIST_ONLY=false
RUN_ALL=true
FEATURES=()
PACKAGES=()
SELECTED_VARIANTS=()

DEFAULT_PACKAGES=(
  gnostr
  gnostr-asyncgit
  gnostr-ngit
  gnostr-web
)

usage() {
  cat <<'EOF'
Usage: cargo-check.sh [variant ...] [--feature VALUE] [--package NAME] [--list] [--help]

Without variants, runs a broad check matrix:
  - workspace
  - workspace --all-features
  - workspace --no-default-features
  - root package checks
  - asyncgit/ngit/web package checks
  - selected package feature permutations

Variants:
  all                  Run the full default matrix (default)
  workspace            Run workspace-only checks
  packages             Run package-only checks
  features             Run feature permutations only
  root                 Run root package checks only
  asyncgit             Run gnostr-asyncgit checks only
  ngit                 Run gnostr-ngit checks only
  web                  Run gnostr-web checks only
  matrix               Alias for all

Options:
  --feature VALUE      Add a Cargo feature for feature permutations
  --feature=VALUE      Add a Cargo feature for feature permutations
  --package NAME       Add a package to the package list (repeatable)
  --list               Print the planned commands and exit
  --help               Show this help

Examples:
  ./scripts/cargo-check.sh
  ./scripts/cargo-check.sh workspace
  ./scripts/cargo-check.sh packages --package gnostr --package gnostr-ngit
  ./scripts/cargo-check.sh features --feature nostr --feature vendored-openssl
  ./scripts/cargo-check.sh --list
EOF
}

join_by_comma() {
  local IFS=,
  echo "$*"
}

run_check() {
  local label="$1"
  shift

  printf '==> %s\n' "$label"
  cargo check -j$NPROC "$@"
}

add_package() {
  local package="$1"
  PACKAGES+=("$package")
}

add_feature() {
  local feature="$1"
  FEATURES+=("$feature")
}

normalize_variants() {
  if [[ ${#SELECTED_VARIANTS[@]} -eq 0 ]]; then
    SELECTED_VARIANTS+=(all)
  fi
}

append_variant() {
  local variant="$1"
  case "$variant" in
    all|matrix)
      SELECTED_VARIANTS+=(all)
      ;;
    workspace|packages|features|root|asyncgit|ngit|web)
      SELECTED_VARIANTS+=("$variant")
      ;;
    *)
      echo "Unsupported variant: $variant" >&2
      exit 1
      ;;
  esac
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --feature)
      shift
      [[ $# -gt 0 ]] || { echo "--feature requires a value" >&2; exit 1; }
      add_feature "$1"
      ;;
    --feature=*)
      add_feature "${1#*=}"
      ;;
    --package)
      shift
      [[ $# -gt 0 ]] || { echo "--package requires a value" >&2; exit 1; }
      add_package "$1"
      ;;
    --package=*)
      add_package "${1#*=}"
      ;;
    --list)
      LIST_ONLY=true
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    workspace|packages|features|root|asyncgit|ngit|web|all|matrix)
      append_variant "$1"
      ;;
    *)
      echo "Unsupported argument: $1" >&2
      exit 1
      ;;
  esac
  shift
done

normalize_variants

if [[ ${#PACKAGES[@]} -eq 0 ]]; then
  PACKAGES=("${DEFAULT_PACKAGES[@]}")
fi

declare -a COMMANDS=()

add_command() {
  local label="$1"
  shift
  COMMANDS+=("$label"$'\n'"$*")
}

build_matrix() {
  local package
  local feature_args=()

  if [[ ${#FEATURES[@]} -gt 0 ]]; then
    feature_args=(--features "$(join_by_comma "${FEATURES[@]}")")
  fi

  for variant in "${SELECTED_VARIANTS[@]}"; do
    case "$variant" in
      all)
        add_command "workspace default" cargo check --workspace
        add_command "workspace all-features" cargo check --workspace --all-features
        add_command "workspace no-default-features" cargo check --workspace --no-default-features

        add_command "root default" cargo check -p gnostr
        add_command "root all-features" cargo check -p gnostr --all-features
        add_command "root no-default-features" cargo check -p gnostr --no-default-features
        add_command "root nostr feature" cargo check -p gnostr --features nostr

        add_command "asyncgit default" cargo check -p gnostr-asyncgit
        add_command "asyncgit all-features" cargo check -p gnostr-asyncgit --all-features

        add_command "ngit default" cargo check -p gnostr-ngit
        add_command "ngit all-features" cargo check -p gnostr-ngit --all-features
        add_command "ngit nostr feature" cargo check -p gnostr-ngit --features nostr
        add_command "ngit vendored-openssl feature" cargo check -p gnostr-ngit --features vendored-openssl

        add_command "web default" cargo check -p gnostr-web
        add_command "web all-features" cargo check -p gnostr-web --all-features
        ;;
      workspace)
        add_command "workspace default" cargo check --workspace
        add_command "workspace all-features" cargo check --workspace --all-features
        add_command "workspace no-default-features" cargo check --workspace --no-default-features
        ;;
      packages)
        for package in "${PACKAGES[@]}"; do
          add_command "$package default" cargo check -p "$package"
          add_command "$package all-features" cargo check -p "$package" --all-features
        done
        ;;
      features)
        if [[ ${#FEATURES[@]} -eq 0 ]]; then
          echo "features variant requires at least one --feature" >&2
          exit 1
        fi
        for package in "${PACKAGES[@]}"; do
          add_command "$package features:$(join_by_comma "${FEATURES[@]}")" cargo check -p "$package" "${feature_args[@]}"
        done
        ;;
      root)
        add_command "root default" cargo check -p gnostr
        add_command "root all-features" cargo check -p gnostr --all-features
        add_command "root no-default-features" cargo check -p gnostr --no-default-features
        add_command "root nostr feature" cargo check -p gnostr --features nostr
        ;;
      asyncgit)
        add_command "asyncgit default" cargo check -p gnostr-asyncgit
        add_command "asyncgit all-features" cargo check -p gnostr-asyncgit --all-features
        ;;
      ngit)
        add_command "ngit default" cargo check -p gnostr-ngit
        add_command "ngit all-features" cargo check -p gnostr-ngit --all-features
        add_command "ngit nostr feature" cargo check -p gnostr-ngit --features nostr
        add_command "ngit vendored-openssl feature" cargo check -p gnostr-ngit --features vendored-openssl
        ;;
      web)
        add_command "web default" cargo check -p gnostr-web
        add_command "web all-features" cargo check -p gnostr-web --all-features
        ;;
    esac
  done
}

build_matrix

if [[ "$LIST_ONLY" == true ]]; then
  for entry in "${COMMANDS[@]}"; do
    label="${entry%%$'\n'*}"
    command="${entry#*$'\n'}"
    printf '%s\n  %s\n' "$label" "$command"
  done
  exit 0
fi

for entry in "${COMMANDS[@]}"; do
  label="${entry%%$'\n'*}"
  command="${entry#*$'\n'}"
  # shellcheck disable=SC2086
  run_check "$label" $command
done
