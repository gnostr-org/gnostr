#!/bin/bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

usage() {
    cat <<'EOF'
Usage: ./build.sh [--macos|-m] [--ios|-i] [--all|-a]

Options:
  -m, --macos   Build the macOS app and extension
  -i, --ios     Build the iOS app and extension
  -a, --all     Build both platforms (default)
  -h, --help    Show this help
EOF
}

build_macos=false
build_ios=false

if [ "$#" -eq 0 ]; then
    build_macos=true
    build_ios=true
else
    while [ "$#" -gt 0 ]; do
        case "$1" in
            -m|--macos)
                build_macos=true
                ;;
            -i|--ios)
                build_ios=true
                ;;
            -a|--all)
                build_macos=true
                build_ios=true
                ;;
            -h|--help)
                usage
                exit 0
                ;;
            *)
                echo "Unknown option: $1" >&2
                usage >&2
                exit 1
                ;;
        esac
        shift
    done
fi

if [ "$build_macos" = false ] && [ "$build_ios" = false ]; then
    usage >&2
    exit 1
fi

if [ "$build_macos" = true ]; then
    bash "$SCRIPT_DIR/build.macos.sh"
fi

if [ "$build_ios" = true ]; then
    bash "$SCRIPT_DIR/build.ios.sh"
fi
