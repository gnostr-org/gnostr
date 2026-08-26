#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
project_root="$(cd "${script_dir}/.." && pwd)"

if [[ -z "${BASH_VERSION:-}" ]]; then
  echo "bash is required" >&2
  exit 1
fi

if (( BASH_VERSINFO[0] < 3 )); then
  echo "bash 3 or newer is required" >&2
  exit 1
fi

usage() {
  cat <<EOF
usage: $(basename "$0") [swift test args...]

Run swift test for every package in this repository, one at a time.

examples:
  $(basename "$0")
  $(basename "$0") --filter AppTests
EOF
}

case "${1:-}" in
  -h|--help|help)
    usage
    exit 0
    ;;
esac

export GIT_CONFIG_COUNT=1
export GIT_CONFIG_KEY_0=safe.bareRepository
export GIT_CONFIG_VALUE_0=all

while IFS= read -r package_manifest; do
  package_path="$(dirname "${package_manifest}")"
  package_name="$(basename "${package_path}")"

  case "${package_name}" in
    LibGit2-iOS)
      echo "==> skip swift test --package-path ${package_name} (binary target only)"
      continue
      ;;
    swift-libp2p-redis)
      "${script_dir}/redis-server.sh" \
        "${project_root}/swift-libp2p-redis/.github/redis1.conf" \
        "${project_root}/swift-libp2p-redis/.github/redis2.conf"
      ;;
    swift-libp2p-queues-redis-driver)
      "${script_dir}/redis-server.sh" \
        "${project_root}/swift-libp2p-queues-redis-driver/.github/redis1.conf"
      ;;
  esac

  echo "==> swift test --package-path ${package_name}"
  swift test --package-path "${package_path}" "$@"
done < <(
  find "${project_root}" \
    -path '*/.build' -prune -o \
    -path '*/.git' -prune -o \
    -name Package.swift -print | sort
)