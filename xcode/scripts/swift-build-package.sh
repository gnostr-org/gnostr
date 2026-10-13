#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
project_root="$(cd "${script_dir}/.." && pwd)"

usage() {
  cat <<EOF
usage: $(basename "$0") <package-dir> [swift build args...]

Run swift build for a package in this repository.

examples:
  $(basename "$0") swift-libp2p
  $(basename "$0") swift-libp2p test
  $(basename "$0") swift-libp2p --product LibP2P
EOF
}

case "${1:-}" in
  -h|--help|help|"")
    usage
    exit 0
    ;;
esac

package_name="$1"
shift

if [[ "${1:-}" == "build" ]]; then
  shift
fi

export GIT_CONFIG_COUNT=1
export GIT_CONFIG_KEY_0=safe.bareRepository
export GIT_CONFIG_VALUE_0=all

exec swift build --package-path "${project_root}/${package_name}" "$@"
