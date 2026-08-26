#!/bin/bash

set -u
# print descriptor
exec 3>&1

repo_path="gh0st-work/gh.rs"
program_name="gh"
program_rename="gh.rs"
test_command="--help"
start_dir=$(pwd)
tmp_dir_prefix="gh-rs-install-source"

check_proc() {
    # Check for /proc by looking for the /proc/self/exe link
    # This is only run on Linux
    if ! $(run_log test -L /proc/self/exe) ; then
        err "fatal: Unable to find /proc/self/exe.  Is /proc mounted?  Installation cannot proceed without /proc."
    fi
}

get_bitness() {
    required_cmd head
    # Architecture detection without dependencies beyond coreutils.
    # ELF files start out "\x7fELF", and the following byte is
    #   0x01 for 32-bit and
    #   0x02 for 64-bit.
    # The printf builtin on some shells like dash only supports octal
    # escape sequences, so we use those.
    local _current_exe_head
    _current_exe_head=$(run_log_or_err head -c 5 /proc/self/exe )
    if [ "$_current_exe_head" = "$(run_log_or_err printf '\177ELF\001')" ]; then
        echo 32
    elif [ "$_current_exe_head" = "$(run_log_or_err printf '\177ELF\002')" ]; then
        echo 64
    else
        err "unknown platform bitness"
    fi
}

is_host_amd64_elf() {
    required_cmd head
    required_cmd tail
    # ELF e_machine detection without dependencies beyond coreutils.
    # Two-byte field at offset 0x12 indicates the CPU,
    # but we're interested in it being 0x3E to indicate amd64, or not that.
    local _current_exe_machine
    _current_exe_machine=$(run_log_or_err head -c 19 /proc/self/exe | run_log_or_err tail -c 1)
    [ "$_current_exe_machine" = "$(run_log_or_err printf '\076')" ]
}

get_endianness() {
    local cputype=$1
    local suffix_eb=$2
    local suffix_el=$3

    # detect endianness without od/hexdump, like get_bitness() does.
    required_cmd head
    required_cmd tail

    local _current_exe_endianness
    _current_exe_endianness="$(run_log_or_err head -c 6 /proc/self/exe | run_log_or_err tail -c 1)"
    if [ "$_current_exe_endianness" = "$(run_log_or_err printf '\001')" ]; then
        echo "${cputype}${suffix_el}"
    elif [ "$_current_exe_endianness" = "$(run_log_or_err printf '\002')" ]; then
        echo "${cputype}${suffix_eb}"
    else
        err "unknown platform endianness"
    fi
}

get_architecture() {
    # Possible arches: 
    # 
    # aarch64-linux-android
    # aarch64-unknown-linux-gnu
    # aarch64-unknown-linux-musl
    # arm-linux-androideabi
    # arm-unknown-linux-gnueabi
    # arm-unknown-linux-gnueabihf
    # armv7-linux-androideabi
    # armv7-unknown-linux-gnueabihf
    # i686-apple-darwin
    # i686-linux-android
    # i686-pc-windows-gnu
    # i686-pc-windows-msvc
    # i686-unknown-linux-gnu
    # mips-unknown-linux-gnu
    # mips64-unknown-linux-gnuabi64
    # mips64el-unknown-linux-gnuabi64
    # mipsel-unknown-linux-gnu
    # powerpc-unknown-linux-gnu
    # powerpc64-unknown-linux-gnu
    # powerpc64le-unknown-linux-gnu
    # s390x-unknown-linux-gnu
    # x86_64-apple-darwin
    # x86_64-linux-android
    # x86_64-pc-windows-gnu
    # x86_64-pc-windows-msvc
    # x86_64-unknown-freebsd
    # x86_64-unknown-illumos
    # x86_64-unknown-linux-gnu
    # x86_64-unknown-linux-musl
    # x86_64-unknown-netbsd
    
    local _ostype _cputype _bitness _arch _clibtype
    _ostype="$(run_log_or_err uname -s)"
    _cputype="$(run_log_or_err uname -m)"
    _clibtype="gnu"

    if [ "$_ostype" = Linux ]; then
        if [ "$(run_log_or_err uname -o)" = Android ]; then
            _ostype=Android
        fi
        if run_log ldd --version 2>&1 | run_log grep -q 'musl'; then
            _clibtype="musl"
        fi
    fi

    if [ "$_ostype" = Darwin ] && [ "$_cputype" = i386 ]; then
        # Darwin `uname -m` lies
        if run_log sysctl hw.optional.x86_64 | run_log grep -q ': 1'; then
            _cputype=x86_64
        fi
    fi

    if [ "$_ostype" = SunOS ]; then
        # Both Solaris and illumos presently announce as "SunOS" in "uname -s"
        # so use "uname -o" to disambiguate.  We use the full path to the
        # system uname in case the user has coreutils uname first in PATH,
        # which has historically sometimes printed the wrong value here.
        if [ "$(run_log_or_err /usr/bin/uname -o)" = illumos ]; then
            _ostype=illumos
        fi

        # illumos systems have multi-arch userlands, and "uname -m" reports the
        # machine hardware name; e.g., "i86pc" on both 32- and 64-bit x86
        # systems.  Check for the native (widest) instruction set on the
        # running kernel:
        if [ "$_cputype" = i86pc ]; then
            _cputype="$(run_log_or_err isainfo -n)"
        fi
    fi

    case "$_ostype" in

        Android)
            _ostype=linux-android
            ;;

        Linux)
            check_proc
            _ostype=unknown-linux-$_clibtype
            _bitness=$(get_bitness)
            ;;

        FreeBSD)
            _ostype=unknown-freebsd
            ;;

        NetBSD)
            _ostype=unknown-netbsd
            ;;

        DragonFly)
            _ostype=unknown-dragonfly
            ;;

        Darwin)
            _ostype=apple-darwin
            ;;

        illumos)
            _ostype=unknown-illumos
            ;;

        MINGW* | MSYS* | CYGWIN* | Windows_NT)
            _ostype=pc-windows-gnu
            ;;

        *)
            err "unrecognized OS type: $_ostype"
            ;;

    esac

    case "$_cputype" in

        i386 | i486 | i686 | i786 | x86)
            _cputype=i686
            ;;

        xscale | arm)
            _cputype=arm
            if [ "$_ostype" = "linux-android" ]; then
                _ostype=linux-androideabi
            fi
            ;;

        armv6l)
            _cputype=arm
            if [ "$_ostype" = "linux-android" ]; then
                _ostype=linux-androideabi
            else
                _ostype="${_ostype}eabihf"
            fi
            ;;

        armv7l | armv8l)
            _cputype=armv7
            if [ "$_ostype" = "linux-android" ]; then
                _ostype=linux-androideabi
            else
                _ostype="${_ostype}eabihf"
            fi
            ;;

        aarch64 | arm64)
            _cputype=aarch64
            ;;

        x86_64 | x86-64 | x64 | amd64)
            _cputype=x86_64
            ;;

        mips)
            _cputype=$(get_endianness mips '' el)
            ;;

        mips64)
            if [ "$_bitness" -eq 64 ]; then
                # only n64 ABI is supported for now
                _ostype="${_ostype}abi64"
                _cputype=$(get_endianness mips64 '' el)
            fi
            ;;

        ppc)
            _cputype=powerpc
            ;;

        ppc64)
            _cputype=powerpc64
            ;;

        ppc64le)
            _cputype=powerpc64le
            ;;

        s390x)
            _cputype=s390x
            ;;
        riscv64)
            _cputype=riscv64gc
            ;;
        loongarch64)
            _cputype=loongarch64
            ;;
        *)
            err "unknown CPU type: $_cputype"

    esac

    # Detect 64-bit linux with 32-bit userland
    if [ "${_ostype}" = unknown-linux-gnu ] && [ "${_bitness}" -eq 32 ]; then
        case $_cputype in
            x86_64)
                if [ -n "${RUSTUP_CPUTYPE:-}" ]; then
                    _cputype="$RUSTUP_CPUTYPE"
                else {
                    # 32-bit runutable for amd64 = x32
                    if is_host_amd64_elf; then {
                         echo "This host is running an x32 userland; as it stands, x32 support is poor," 1>&2
                         echo "and there isn't a native toolchain -- you will have to install" 1>&2
                         echo "multiarch compatibility with i686 and/or amd64, then select one" 1>&2
                         echo "by re-running this script with the RUSTUP_CPUTYPE environment variable" 1>&2
                         echo "set to i686 or x86_64, respectively." 1>&2
                         echo 1>&2
                         echo "You will be able to add an x32 target after installation by running" 1>&2
                         echo "  rustup target add x86_64-unknown-linux-gnux32" 1>&2
                         exit 1
                    }; else
                        _cputype=i686
                    fi
                }; fi
                ;;
            mips64)
                _cputype=$(get_endianness mips '' el)
                ;;
            powerpc64)
                _cputype=powerpc
                ;;
            aarch64)
                _cputype=armv7
                if [ "$_ostype" = "linux-android" ]; then
                    _ostype=linux-androideabi
                else
                    _ostype="${_ostype}eabihf"
                fi
                ;;
            riscv64gc)
                err "riscv64 with 32-bit userland unsupported"
                ;;
        esac
    fi

    # Detect armv7 but without the CPU features Rust requireds in that build,
    # and fall back to arm.
    # See https://github.com/rust-lang/rustup.rs/issues/587.
    if [ "$_ostype" = "unknown-linux-gnueabihf" ] && [ "$_cputype" = armv7 ]; then
        if run_log grep '^Features' /proc/cpuinfo | run_log grep -q -v neon; then
            # At least one processor does not have NEON.
            _cputype=arm
        fi
    fi

    _arch="${_cputype}-${_ostype}"

    RETVAL="$_arch"
}

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
    run_log_or_err command -v "$1" > /dev/null 2>&1
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

no_tmp_dir="__no_tmp_dir__"
tmp_dir="$no_tmp_dir"
clean() {
    say "\nCleaning up..."
    run_log_or_err cd $start_dir
    if [[ "$tmp_dir" != "$no_tmp_dir" ]] && [[ -d "$tmp_dir" ]]; then
        run_log_or_err rm -rf $tmp_dir
    fi
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

    say "\nExtracting your machine kernel info and architecture..."
    get_architecture || return 1
    local _arch="$RETVAL"
    assert_nz "$_arch" "arch"

    say "\nCreating temporary directory..."
    tmp_dir=$(run_log_or_err mktemp -d -t $tmp_dir_prefix.XXXXXXXXXX)
    run_log_or_err cd $tmp_dir

    say "\nFetching the latest release version of the prebuilt binary..."
    local _tar_name="$program_name.$_arch.tar.gz"
    local _ver=$( run_log_or_err curl --silent -qI https://github.com/$repo_path/releases/latest | run_log_or_err awk -F '/' '/^location/ {print substr($NF, 1, length($NF)-1)}')
    _ver="${_ver#v}"
    if [[ "$_ver" == "" ]]; then
        err "$(hr)\nError occured, check your internet connection & repo (https://github.com/$repo_path) availability"
    fi
    
    say "\nDownloading the tarball archive with the latest (v$_ver) tarball..."
    local _wget_status=$( wget_with_status https://github.com/$repo_path/releases/download/v$_ver/$_tar_name )
    if [[ "$_wget_status" != 200 ]]; then
        err "$(hr)\nWrong respone status code ($_wget_status), check your internet connection & file (https://github.com/$repo_path/releases/download/v$_ver/$_tar_name) availability "
    fi
    
    say "\nUnpacking the tarball archive..."
    run_log_or_err tar -xzf $_tar_name
    run_log_or_err rm $_tar_name
    
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
    run_log_or_err rm -rf $tmp_dir
    
    say "\nAdding /bin/$_program_name to PATH (/etc/profile.d/$_program_name.sh)..."
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
