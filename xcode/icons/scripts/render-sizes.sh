#!/usr/bin/env bash

set -euo pipefail

resize_png() {
  local source_file="$1"
  local width="$2"
  local height="$3"
  local output_file="$4"

  case "${source_file##*.}" in
    svg|SVG)
      rsvg-convert -w "$width" -h "$height" "$source_file" -o "$output_file"
      ;;
    *)
      sips -z "$height" "$width" "$source_file" --out "$output_file" >/dev/null
      ;;
  esac
}

generate_square_sizes() {
  local source_file="$1"
  local output_dir="$2"
  local sizes=(1024 512 256 180 170 167 152 128 120 102 87 85 80 76 73 64 60 58 40 32 29 24 20 16 8 4)

  mkdir -p "$output_dir"
  for side in "${sizes[@]}"; do
    resize_png "$source_file" "$side" "$side" "$output_dir/${side}x${side}.png"
  done
}

generate_banner_sizes() {
  local source_file="$1"
  local output_dir="$2"

  mkdir -p "$output_dir"
  resize_png "$source_file" 3072 1024 "$output_dir/icon3072x1024.png"
  resize_png "$source_file" 1536 512 "$output_dir/1536x512.png"
  resize_png "$source_file" 1024 341 "$output_dir/1024x341.png"
}

generate_appicon_sizes() {
  local source_file="$1"
  local output_dir="$2"

  local sizes=(40 60 58 87 76 80 120 180 152 167 1024 16 32 64 128 256 512)

  mkdir -p "$output_dir"
  for size in "${sizes[@]}"; do
    resize_png "$source_file" "$size" "$size" "$output_dir/${size}x${size}.png"
  done
}

main() {
  local mode="${1:-}"
  local source_file="${2:-}"
  local output_dir="${3:-}"

  case "$mode" in
    square)
      [[ -n "$source_file" && -n "$output_dir" ]] || {
        echo "usage: $0 square <source_png> <output_dir>" >&2
        exit 1
      }
      generate_square_sizes "$source_file" "$output_dir"
      ;;
    banner)
      [[ -n "$source_file" && -n "$output_dir" ]] || {
        echo "usage: $0 banner <source_png> <output_dir>" >&2
        exit 1
      }
      generate_banner_sizes "$source_file" "$output_dir"
      ;;
    appicon)
      [[ -n "$source_file" && -n "$output_dir" ]] || {
        echo "usage: $0 appicon <source_png> <output_dir>" >&2
        exit 1
      }
      generate_appicon_sizes "$source_file" "$output_dir"
      ;;
    *)
      echo "usage: $0 {square|banner|appicon} <source_png> <output_dir>" >&2
      exit 1
      ;;
  esac
}

main "$@"
