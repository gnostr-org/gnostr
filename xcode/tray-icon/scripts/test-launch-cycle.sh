#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
crate_dir="$(cd "${script_dir}/.." && pwd)"
manifest="${crate_dir}/Cargo.toml"
display_seconds="${DISPLAY_SECONDS:-10}"

colors=(
  "#ff00ff"
  "#ff007f"
  "#ff0000"
  "#ff7f00"
  "#ffff00"
  "#7fff00"
  "#00ff00"
  "#00ff7f"
  "#00ffff"
  "#007fff"
  "#0000ff"
  "#7f00ff"
  "#bf00ff"
)

cleanup_pid=""
cleanup() {
  if [[ -n "$cleanup_pid" ]]; then
    kill "$cleanup_pid" 2>/dev/null || true
    wait "$cleanup_pid" 2>/dev/null || true
  fi
}

trap cleanup EXIT INT TERM

cargo test --manifest-path "$manifest" --lib

for tint in "${colors[@]}"; do
  temp_dir="$(mktemp -d "${TMPDIR:-/tmp}/tray-icon-${tint//#/}.XXXXXX")"
  log_file="$temp_dir/run.log"
  echo "launching tray icon with tint $tint"
  TRAY_ICON_TINT="$tint" cargo run --manifest-path "$manifest" >"$log_file" 2>&1 &
  cleanup_pid="$!"

  sleep "$display_seconds"

  if ! kill -0 "$cleanup_pid" 2>/dev/null; then
    echo "tray icon exited before the $display_seconds second window" >&2
    tail -n 80 "$log_file" >&2 || true
    exit 1
  fi

  kill "$cleanup_pid"
  wait "$cleanup_pid" || true
  cleanup_pid=""
  rm -rf "$temp_dir"
  echo "relaunching with next tint"
done
