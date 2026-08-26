#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
project_root="$(cd "${script_dir}/.." && pwd)"
template_root="${project_root}/libp2p-app-template"

action="${1:-build}"

if (($#)); then
  shift
fi

export GIT_CONFIG_COUNT="${GIT_CONFIG_COUNT:-1}"
export GIT_CONFIG_KEY_0="${GIT_CONFIG_KEY_0:-safe.bareRepository}"
export GIT_CONFIG_VALUE_0="${GIT_CONFIG_VALUE_0:-all}"

case "${action}" in
  build)
    exec swift build --package-path "${template_root}" "$@"
    ;;
  test)
    exec swift test --package-path "${template_root}" "$@"
    ;;
  run)
    exec swift run --package-path "${template_root}" "$@"
    ;;
  *)
    exec swift "${action}" --package-path "${template_root}" "$@"
    ;;
esac