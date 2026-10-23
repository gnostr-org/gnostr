#!/usr/bin/env bash

ui_prompt() {
  local label="$1" default="${2:-}"
  printf '%s [%s]: ' "$label" "$default"
  if ! IFS= read -r UI_VALUE; then
    printf '\nCancelled.\n'
    exit 0
  fi
  UI_VALUE="${UI_VALUE:-$default}"
}

ui_confirm() {
  local answer
  printf '%s [y/N]: ' "$1"
  if ! IFS= read -r answer; then
    printf '\nCancelled.\n'
    exit 0
  fi
  case "$answer" in
    y|Y|yes|YES) ;;
    *) echo "Cancelled."; exit 0 ;;
  esac
}

ui_select() {
  local label="$1"
  shift
  local options=("$@") choices=() picked=()
  local count="${#options[@]}" index selection choice
  UI_SELECTED=()
  if [[ "$count" -eq 0 ]]; then
    echo "No matching entries."
    exit 0
  fi
  printf '=== %s ===\n' "$label"
  for index in "${!options[@]}"; do
    printf '%3d) %s\n' "$((index + 1))" "${options[index]}"
  done
  printf 'Select numbers (spaces or commas), all, or q to quit: '
  if ! IFS= read -r selection; then
    printf '\nCancelled.\n'
    exit 0
  fi
  read -r -a choices <<<"${selection//,/ }"
  if [[ "${#choices[@]}" -eq 0 ]]; then
    echo "Cancelled."
    exit 0
  fi
  for choice in "${choices[@]}"; do
    case "$choice" in
      q|quit) echo "Cancelled."; exit 0 ;;
      all)
        for index in "${!options[@]}"; do picked[index]=1; done
        ;;
      *)
        if [[ ! "$choice" =~ ^[1-9][0-9]*$ || ${#choice} -gt ${#count} ]] ||
          (( choice > count )); then
          printf 'Error: invalid selection "%s"; choose numbers 1-%s.\n' "$choice" "$count" >&2
          return 1
        fi
        picked[choice-1]=1
        ;;
    esac
  done
  for index in "${!picked[@]}"; do
    UI_SELECTED+=("${options[index]}")
  done
}

ui_choose() {
  ui_select "$@"
  if [[ "${#UI_SELECTED[@]}" -ne 1 ]]; then
    echo "Error: select exactly one entry." >&2
    return 1
  fi
  UI_VALUE="${UI_SELECTED[0]}"
}
