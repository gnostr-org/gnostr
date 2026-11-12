#!/usr/bin/env bash
# Source this file after bash-completion to replace its Makefile target parser.

_GNOSTR_COMPLETION_ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"

_gnostr_make_targets() {
    local directory=$1 makefile=$2 script name
    [[ -f "$directory/$makefile" ]] || return 0

    # Read only literal rules; never evaluate Make expressions or shell recipes.
    awk -F: '
        /^[[:alnum:]_][[:alnum:]_. \/-]*:/ {
            if ($2 ~ /^[[:space:]]*=/) next
            count = split($1, targets, /[[:space:]]+/)
            for (i = 1; i <= count; i++)
                if (targets[i] != "") print targets[i]
        }
    ' "$directory/$makefile"

    if [[ "$directory/$makefile" -ef "$_GNOSTR_COMPLETION_ROOT/Makefile" ]]; then
        for script in "$directory"/scripts/*.sh "$directory"/scripts/update "$directory"/scripts/release; do
            [[ -f "$script" ]] || continue
            name=${script##*/}
            printf 'script-%s\n' "${name%.sh}"
        done
    fi
}

_gnostr_make_complete() {
    local current=${COMP_WORDS[COMP_CWORD]} directory=$PWD makefile='' word target i
    COMPREPLY=()
    for ((i = 1; i < COMP_CWORD; i++)); do
        word=${COMP_WORDS[i]}
        case "$word" in
            -C|--directory)
                i=$((i + 1))
                ((i < COMP_CWORD)) || return 0
                word=${COMP_WORDS[i]}
                case "$word" in
                    /*) directory=$word ;;
                    *) directory=$directory/$word ;;
                esac
                ;;
            -C*|--directory=*)
                word=${word#-C}
                word=${word#--directory=}
                case "$word" in
                    /*) directory=$word ;;
                    *) directory=$directory/$word ;;
                esac
                ;;
            -f|--file|--makefile)
                i=$((i + 1))
                ((i < COMP_CWORD)) || return 0
                makefile=${COMP_WORDS[i]}
                ;;
            -f*) makefile=${word#-f} ;;
            --file=*|--makefile=*) makefile=${word#*=} ;;
        esac
    done
    [[ "$current" != -* && "$current" != *=* ]] || return 0

    if [[ -z "$makefile" ]]; then
        for word in GNUmakefile makefile Makefile; do
            if [[ -f "$directory/$word" ]]; then
                makefile=$word
                break
            fi
        done
    fi
    if [[ "$makefile" == /* ]]; then
        directory=/
        makefile=${makefile#/}
    fi
    while IFS= read -r target; do
        if [[ "$target" == "$current"* ]]; then
            COMPREPLY[${#COMPREPLY[@]}]=$target
        fi
    done < <(_gnostr_make_targets "$directory" "$makefile" | LC_ALL=C sort -u)
    return 0
}

complete -o bashdefault -o default -F _gnostr_make_complete make gmake