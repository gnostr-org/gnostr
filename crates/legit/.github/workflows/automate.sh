#!/usr/bin/env bash
if ! hash rust 2>/dev/null; then
sudo rm -rf $HOME/.cargo/bin
sudo curl https://sh.rustup.rs -sSf > rustup.sh && chmod +x rustup.sh && ./rustup.sh -y
source $HOME/.cargo/env
fi
#ENV VARS
RUSTC=$(which rustc)
CARGO=$(which cargo)

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
        if !hash rustc 2>/dev/null; then
			brew install rust
		fi
        #if !hash pandoc 2>/dev/null; then
            brew install pandoc
        #fi
        #pip install -U sphinx
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
            sudo $PACKAGE_MANAGER $INSTALL $AWK pandoc
            report
            pandoc -s README.md -o index.html
        fi
        checkbrew
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
#sudo curl https://sh.rustup.rs -sSf > rustup.sh && chmod +x rustup.sh && ./rustup.sh -y
$CARGO build
$CARGO build --release
#sudo install -v ././target/release/legit /usr/local/bin/legit
$CARGO install --path .
$CARGO run
$CARGO run . -p 00000 -m "gnostr-legit: automate"
gnostr-legit -h
