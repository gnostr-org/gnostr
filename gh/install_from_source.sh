#!/bin/bash

set -u
# print descriptor
exec 3>&1

repo_name="gh.rs"
repo_path="gh0st-work/gh.rs"
program_name="gh"
program_rename="gh.rs"
test_command="--help"
start_dir=$(pwd)
tmp_dir_prefix="gh-rs-install-source"


join_by() {
    local -n sep="$1"
    local -n parts="${@:2}"
    local result_str=""
    for i in "${!parts[@]}"; do
        local part = "${part[$i]}"
        if [[ i != 0 ]]; then
            result_str+="$sep"
        fi
        result_str+="$part"
    done
    echo "$result_str"
}

requote() {
    local res=""
    for x in "$@"; do
        [[ $x = *[[:space:]]* ]] && res="${res} \"${x}\"" || res="${res} ${x}"
    done
    printf '%s\n' "${res# }"
}

prefix_first_ident_wrap() {
    echo "$1$2" | awk -v w="$COLUMNS" -v "s=${#1}" 'BEGIN{p=sprintf("%*s",s,"")} NF{while(length>w){print substr($0,1,w);$0=p substr($0,w+1)} if($0!="") print;next} 1'
}

say() {
    echo -e "$@" >&3
}

err() {
    say "$@" >&2
    clean
    exit 1
}

run_or_err() {
    if ! "$@"; then err "\n$(hr)\n\n[ERROR] Command failed: $(requote "$@")\n\nExiting...\n\n$(hr)"; fi
}

log() {
    say "$(prefix_first_ident_wrap "    - " "$(requote "$@")" )"
}

log_sudo() {
    say "$(prefix_first_ident_wrap "    - [sudo] " "$(requote "$@")" )"
}

run_log() {
    log "$@"
    echo "$@"
}

run_log_or_err() {
    log "$@"
    run_or_err "$@"
}

run_or_err_sudo() {
    run_or_err sudo "$@"
}

run_log_or_err_sudo() {
    log_sudo "$@"
    run_or_err_sudo "$@"
}

check_cmd() {
    run_log command -v "$1" > /dev/null 2>&1
}

required_cmd() {
    if ! check_cmd "$1"; then
        err "required '$1' (command not found)"
    fi
}

required_sudo() {
    run_log_or_err sudo -v
}

assert_nz() {
    if [ -z "$1" ]; then err "assert_nz $2"; fi
}

hr() {
    echo "$(run_or_err printf %"$COLUMNS"s | run_or_err tr " " "-")"
}


wget_with_status() {
    local _wget_status=($( run_log_or_err wget --server-response "$1" 2>&1 | run_log_or_err awk '{ if (match($0, /.*HTTP\/[0-9\.]+ ([0-9]+).*/, m)) print m[1] }' ))
    _wget_status="${_wget_status[${#_wget_status[@]} - 1]}"
    echo "$_wget_status"
}


path_prefix=""
no_tmp_dir="__no_tmp_dir__"
tmp_dir="$no_tmp_dir"
clean() {
    say "\nCleaning up..."
    run_log_or_err cd $start_dir
    if [[ "$tmp_dir" != "$no_tmp_dir" ]] && [[ -d "$tmp_dir" ]]; then
        say "$tmp_dir"
        run_log_or_err_sudo rm -rf $tmp_dir
    fi
    export PATH="${PATH:${#path_prefix}}"
}

main() {
    say "Checking required commands & permissions..."
    required_sudo
    required_cmd uname
    required_cmd printf
    required_cmd tr
    required_cmd pwd
    required_cmd cd
    required_cmd mktemp
    required_cmd curl
    required_cmd awk
    required_cmd wget
    required_cmd tar
    required_cmd rm
    required_cmd chmod
    required_cmd sh

    say "\nCreating temporary directory..."
    tmp_dir=$(run_log_or_err mktemp -d -t $tmp_dir_prefix.XXXXXXXXXX)
    run_log_or_err cd $tmp_dir

    say "\nChecking if Rust & cargo installed..."
    local _rust_from_env=$(check_cmd cargo)
    if ! "$_rust_from_env"; then
        say "\nInstalling Rust & cargo..."
        run_log_or_err mkdir "$tmp_dir/rustup"
        run_log_or_err mkdir "$tmp_dir/cargo"
        run_log_or_err wget https://sh.rustup.rs -O rustup-init.sh -q
        run_log_or_err_sudo RUSTUP_HOME="$tmp_dir/rustup" CARGO_HOME="$tmp_dir/cargo" bash -c 'sh rustup-init.sh -y'
        path_prefix="$tmp_dir/cargo/bin:"
        export PATH="$path_prefix$PATH"
        run_log_or_err rustup default stable
    fi
    
    say "\nFetching the latest release version number..."
    local _ver=$( run_log_or_err curl --silent -qI https://github.com/$repo_path/releases/latest | run_log_or_err awk -F '/' '/^location/ {print substr($NF, 1, length($NF)-1)}')
    _ver="${_ver#v}"
    if [[ "$_ver" == "" ]]; then
        err "$(hr)\nError occured, check your internet connection & repo (https://github.com/$repo_path) availability"
    fi
   
    say "\nDownloading the tarball archive with the latest (v$_ver) release version of the source code..."
    local _wget_status=$( wget_with_status https://github.com/$repo_path/archive/refs/tags/v$_ver.tar.gz )
    if [[ "$_wget_status" != 200 ]]; then
        err "$(hr)\nWrong respone status code ($_wget_status), check your internet connection & file (https://github.com/$repo_path/releases/download/v$_ver/$_tar_name) availability "
    fi
    
    say "\nUnpacking the tarball archive..."
    run_log_or_err tar -xzf v$_ver.tar.gz
    run_log_or_err rm v$_ver.tar.gz
    echo $(ls)
    
    say "\nBuilding binary from the source code with Rust..."
    local _src_dir_name="$repo_name-$_ver" 
    run_log_or_err cd $_src_dir_name
    run_log_or_err cargo build --release --bin $program_name 
    run_log_or_err cp target/release/$program_name $program_name
    
    if [[ "$program_name" != "$program_rename" ]]; then
        say "\nRenaming \"$program_name\" -> \"$program_rename\"..."
        run_log_or_err mv $program_name $program_rename
    fi
    local _program_name="$program_rename"
    
    say "\nGiving execute (+x) permissions to $_program_name binary..."
    run_log_or_err chmod +x $_program_name
    
    say "\nTrying to run \"./$_program_name $test_command\" as test command to verify successful installation..."
    log ./$_program_name $test_command
    local _help_output=$( ./$_program_name $test_command )
    _help_exit_code=$?
    if [[ $_help_exit_code != 0 ]]; then
        err "$(hr)\n\"./$_program_name $test_command\" exited with code $_help_exit_code:\n$(hr)\n$_help_output\n$(hr)\nAssuming not compatable with your machine or broken, cancelling install"
    fi
    
    say "\nMoving ./$_program_name to /bin/$_program_name..."
    run_log_or_err_sudo mv ./$_program_name /bin/
    run_log_or_err cd $start_dir
    run_log_or_err_sudo rm -rf $tmp_dir
    
    say "\nAdding /bin/$_program_name to PATH (/etc/profile.d/$_program_name.sh)..."
    # TODO: make it fn
    local _write_path_update_sh_output=$(echo -e '#!/bin/sh\n\nexport PATH=$PATH:/bin/'"$_program_name" | sudo tee /etc/profile.d/$_program_name.sh)
    _write_path_update_sh_code=$?
    if [[ $_write_path_update_sh_code != 0 ]]; then
        err "$(hr)\nWritting to /etc/profile.d/$_program_name.sh exited with code $_write_path_update_sh_code:\n$(hr)\n$_write_path_update_sh_output\n$(hr)\nAssuming not compatable with your machine or broken, cancelling install"
    fi
    run_log_or_err_sudo bash /etc/profile.d/$_program_name.sh

    clean

    say "\n$(hr)\n\nSUCCESS!\nInstalled $_program_name and added to PATH.\n\nTo use just call:\n    $_program_name ...\n\nHappy hacking & have a nice day :)"
}

main "$@" || exit 1
