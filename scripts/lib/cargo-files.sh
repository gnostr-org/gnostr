#!/usr/bin/env bash

cargo_paths() {
    local path
    git ls-files --cached --others -z -- \
        Cargo.toml Cargo.lock '**/Cargo.toml' '**/Cargo.lock' \
        ':(exclude)vendor/**' ':(exclude)**/vendor/**' \
        ':(exclude)target/**' ':(exclude)**/target/**' |
    while IFS= read -r -d '' path; do
        [[ -f "$path" ]] || continue
        printf './%s\0' "$path"
    done
}

managed_manifests() {
    local path
    while IFS= read -r -d '' path; do
        [[ "$path" != */Cargo.toml ]] || printf '%s\n' "$path"
    done < <(cargo_paths)
}

stage_cargo_files() {
    local paths=() path
    while IFS= read -r -d '' path; do
        paths+=("$path")
    done < <(cargo_paths)
    if [[ "${#paths[@]}" -gt 0 ]]; then
        git add "$@" -- "${paths[@]}"
    fi
}
