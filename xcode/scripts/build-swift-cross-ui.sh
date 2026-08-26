#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
project_root="$(cd "${script_dir}/.." && pwd)"
package_path="${project_root}/swift-cross-ui"

export GIT_CONFIG_COUNT="${GIT_CONFIG_COUNT:-1}"
export GIT_CONFIG_KEY_0="${GIT_CONFIG_KEY_0:-safe.bareRepository}"
export GIT_CONFIG_VALUE_0="${GIT_CONFIG_VALUE_0:-all}"

have_command() {
  command -v "$1" >/dev/null 2>&1
}

gtk4_ready() {
  have_command gtk4-launch && pkg-config --exists gtk4
}

install_dependencies() {
  if gtk4_ready; then
    return
  fi

  if have_command brew; then
    brew install gtk4 pkg-config
    return
  fi

  if have_command apt-get; then
    sudo apt-get update
    sudo apt-get install -y libgtk-4-dev pkg-config
    return
  fi

  if have_command dnf; then
    sudo dnf install -y gtk4-devel pkgconf-pkg-config
    return
  fi

  if have_command pacman; then
    sudo pacman -Syu --noconfirm gtk4 pkgconf
    return
  fi

  if have_command apk; then
    sudo apk add gtk4-dev pkgconf
    return
  fi

  echo "Missing GTK 4 dependency and no supported package manager was found." >&2
  exit 1
}

install_dependencies

exec swift build --package-path "${package_path}" --target SwiftCrossUI "$@"