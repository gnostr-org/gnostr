#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
project_root="$(cd "${script_dir}/.." && pwd)"

usage() {
  cat <<EOF
usage: $(basename "$0") [redis.conf ...]

Start one or more Redis servers for this repository.

If no config is provided, the default is:
  swift-libp2p-queues-redis-driver/.github/redis1.conf

Examples:
  $(basename "$0")
  $(basename "$0") swift-libp2p-redis/.github/redis1.conf swift-libp2p-redis/.github/redis2.conf
EOF
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" || "${1:-}" == "help" ]]; then
  usage
  exit 0
fi

install_redis() {
  if command -v redis-server >/dev/null 2>&1 && command -v redis-cli >/dev/null 2>&1; then
    return
  fi

  case "$(uname -s)" in
    Darwin)
      if ! command -v brew >/dev/null 2>&1; then
        echo "brew is required to install Redis on macOS" >&2
        exit 1
      fi
      brew install redis
      ;;
    Linux)
      if [[ $EUID -eq 0 ]]; then
        sudo_cmd=()
      elif command -v sudo >/dev/null 2>&1; then
        sudo_cmd=(sudo)
      else
        echo "sudo is required to install Redis on Linux" >&2
        exit 1
      fi

      if command -v apt-get >/dev/null 2>&1; then
        "${sudo_cmd[@]}" apt-get update
        "${sudo_cmd[@]}" apt-get install -y redis-server
      elif command -v dnf >/dev/null 2>&1; then
        "${sudo_cmd[@]}" dnf install -y redis
      elif command -v yum >/dev/null 2>&1; then
        "${sudo_cmd[@]}" yum install -y redis
      elif command -v apk >/dev/null 2>&1; then
        "${sudo_cmd[@]}" apk add redis
      elif command -v pacman >/dev/null 2>&1; then
        "${sudo_cmd[@]}" pacman -Sy --noconfirm redis
      else
        echo "no supported package manager found to install Redis" >&2
        exit 1
      fi
      ;;
    *)
      echo "unsupported platform: $(uname -s)" >&2
      exit 1
      ;;
  esac
}

config_port() {
  local config="$1"
  awk '$1 == "port" && $2 ~ /^[0-9]+$/ { print $2; exit }' "$config"
}

wait_for_redis() {
  local port="$1"
  local attempts=50
  while (( attempts > 0 )); do
    if redis-cli -p "$port" ping >/dev/null 2>&1; then
      return
    fi
    sleep 0.1
    attempts=$((attempts - 1))
  done
  echo "redis-server did not become ready on port $port" >&2
  exit 1
}

start_redis() {
  local config="$1"
  if [[ ! -f "$config" ]]; then
    echo "missing redis config: $config" >&2
    exit 1
  fi

  local port
  port="$(config_port "$config")"
  if [[ -z "$port" ]]; then
    echo "could not determine port from config: $config" >&2
    exit 1
  fi

  if redis-cli -p "$port" ping >/dev/null 2>&1; then
    echo "redis already running on port $port"
    return
  fi

  echo "starting redis-server with $config on port $port"
  redis-server "$config" --daemonize yes
  wait_for_redis "$port"
}

install_redis

configs=("$@")
if [[ ${#configs[@]} -eq 0 ]]; then
  configs=("${project_root}/swift-libp2p-queues-redis-driver/.github/redis1.conf")
fi

for config in "${configs[@]}"; do
  start_redis "$config"
done