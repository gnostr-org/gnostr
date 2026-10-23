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

refresh_cargo_files() {
    local path listing
    listing="$(mktemp "${TMPDIR:-/tmp}/gnostr-cargo-files.XXXXXX")" || return 1
    if ! cargo_paths > "$listing"; then
        rm -f -- "$listing"
        echo "Error: could not discover parent-repository Cargo files." >&2
        return 1
    fi
    CARGO_PATHS=()
    while IFS= read -r -d '' path; do
        CARGO_PATHS+=("$path")
    done < "$listing"
    rm -f -- "$listing"
}

managed_manifests() {
    local path
    for path in ${CARGO_PATHS[@]+"${CARGO_PATHS[@]}"}; do
        [[ "$path" != */Cargo.toml ]] || printf '%s\n' "$path"
    done
}

stage_cargo_files() {
    refresh_cargo_files || return 1
    if [[ "${#CARGO_PATHS[@]}" -gt 0 ]]; then
        git add "$@" -- "${CARGO_PATHS[@]}"
    fi
}
