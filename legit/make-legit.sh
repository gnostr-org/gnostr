#!/usr/bin/env bash

#TODO:
if ! hash rustup-init 2>/dev/null; then
sudo -su $(whoami) rm -rf $HOME/.cargo/bin
sudo -su $(whoami) curl https://sh.rustup.rs -sSf > rustup.sh && chmod +x rustup.sh && ./rustup.sh -y
source $HOME/.cargo/env
else
    command -v rustup-init
fi

#ENV VARS
OS=$(uname)
OS_VERSION=$(uname -r)
UNAME_M=$(uname -m)
ARCH=$(uname -m)
export OS
export OS_VERSION
export UNAME_M
export ARCH
report() {
echo OS:
echo "$OS" | awk '{print tolower($0)}'
echo OS_VERSION:
echo "$OS_VERSION" | awk '{print tolower($0)}'
echo UNAME_M:
echo "$UNAME_M" | awk '{print tolower($0)}'
echo ARCH:
echo "$ARCH" | awk '{print tolower($0)}'
echo OSTYPE:
echo "$OSTYPE" | awk '{print tolower($0)}'
}
checkbrew() {
    if hash brew 2>/dev/null; then
        if !hash $AWK 2>/dev/null; then
            brew install $AWK
        fi
        if !hash git 2>/dev/null; then
            brew install git
        fi
        if hash openssl 2>/dev/null; then
            brew install openssl@1.1
        fi
        if ! hash rustc 2>/dev/null; then
            brew install rustup-init
            rustup-init
            source ~/.bashrc
        fi
        true
    else
        /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install.sh)"
        checkbrew
    fi
}
checkraspi(){
    echo 'Checking Raspi'
    if [ -e /etc/rpi-issue ]; then
    echo "- Original Installation"
    cat /etc/rpi-issue
    fi
    if [ -e /usr/bin/lsb_release ]; then
    echo "- Current OS"
    lsb_release -irdc
    fi
    echo "- Kernel"
    uname -r
    echo "- Model"
    cat /proc/device-tree/model && echo
    echo "- hostname"
    hostname
    echo "- Firmware"
    /opt/vc/bin/vcgencmd version
}

if [[ "$OSTYPE" == "linux"* ]]; then
    #CHECK APT
    if [[ "$OSTYPE" == "linux-gnu" ]]; then
        PACKAGE_MANAGER=apt
        export PACKAGE_MANAGER
        INSTALL=install
        export INSTALL
        AWK=gawk
        export AWK
        if hash apt 2>/dev/null; then
            $PACKAGE_MANAGER $INSTALL $AWK
            report
        fi
		sudo apt-get install libssl-dev rustc cargo --fix-missing
    fi
    if [[ "$OSTYPE" == "linux-musl" ]]; then
        PACKAGE_MANAGER=apk
        export PACKAGE_MANAGER
        INSTALL=install
        export INSTALL
        AWK=gawk
        export AWK
        if hash apk 2>/dev/null; then
            $PACKAGE_MANAGER $INSTALL $AWK
            report
        fi
    fi
    if [[ "$OSTYPE" == "linux-arm"* ]]; then
        PACKAGE_MANAGER=apt
        export PACKAGE_MANAGER
        INSTALL=install
        export INSTALL
        AWK=gawk
        echo $AWK
        export AWK
        checkraspi
        if hash apt 2>/dev/null; then
            $PACKAGE_MANAGER $INSTALL $AWK
            report
        fi
    fi
elif [[ "$OSTYPE" == "darwin"* ]]; then
        report
        PACKAGE_MANAGER=brew
        export PACKAGE_MANAGER
        INSTALL=install
        export INSTALL
        AWK=awk
        export AWK
    checkbrew
    #brew reinstall --force openssl@1.1
    #symlink on your machine too...
    #echo brew list --versions
    #brew list --versions
    echo
    OPENSSL_VERSION=$(brew list --versions | grep -i -E  "openssl" | sed 's%openssl@1.1% %')
    echo openssl version
    echo $OPENSSL_VERSION
    export OPENSSL_VERSION
    #sudo mkdir -p /usr/local/include/openssl/$OPENSSL_VERSION
    #rm -rf /usr/local/include/openssl/$OPENSSL_VERSION
    #sudo ln -s /usr/local/opt/openssl/include/openssl /usr/local/include/openssl/$OPENSSL_VERSION
    #rm -rf /usr/bin/openssl
    #sudo ln -s /usr/local/Cellar/openssl/$OPENSSL_VERSION/include/openssl    /usr/bin/openssl
    #rm -rf /usr/local/bin/openssl
    #sudo ln -s /usr/local/Cellar/openssl/$OPENSSL_VERSION/include/openssl    /usr/local/bin/openssl
    #sudo rm -rf /usr/local/lib/libssl.*.*.dylib
    #sudo rm -rf /usr/local/lib/libcrypto.*.*.dylib
    #sudo rm -rf /usr/local/lib/libcrypto.*.*.a
    #ln -s /usr/local/opt/openssl/lib/libssl.1.1.dylib                   /usr/local/lib/libssl.1.1.dylib
    #ln -s /usr/local/opt/openssl/lib/libcrypto.1.1.dylib                /usr/local/lib/libcrypto.1.1.dylib
    #ln -s /usr/local/opt/openssl/lib/libcrypto.a                        /usr/local/lib/libcrypto.a
    which openssl
    cargo build
        checkbrew
elif [[ "$OSTYPE" == "cygwin" ]]; then
    echo TODO add support for $OSTYPE
elif [[ "$OSTYPE" == "msys" ]]; then
    echo TODO add support for $OSTYPE
elif [[ "$OSTYPE" == "win32" ]]; then
    echo TODO add support for $OSTYPE
elif [[ "$OSTYPE" == "freebsd"* ]]; then
    echo TODO add support for $OSTYPE
else
    echo TODO add support for $OSTYPE
fi

sudo -su $(whoami) cargo build
sudo -su $(whoami) cargo build --release
