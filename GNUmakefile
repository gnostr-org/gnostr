default: all
ext/secp256k1/include/secp256k1.h:
	git checkout ext
	git checkout ext/secp256k1

ext/secp256k1/configure: ext/secp256k1/include/secp256k1.h
	@cd ext/secp256k1; \
	./autogen.sh

ext/secp256k1/Makefile: ext/secp256k1/configure
	@cd ext/secp256k1; \
	./configure \
        --disable-shared \
        --enable-module-ecdh \
        --enable-module-schnorrsig \
        --enable-module-extrakeys

ext/secp256k1/.libs/libsecp256k1.a: ext/secp256k1/Makefile
	@cd ext/secp256k1; \
	make -j libsecp256k1.la >/dev/null

.PHONY:libsecp256k1.a
libsecp256k1.a: ext/secp256k1/.libs/libsecp256k1.a
	@cp $< $@ || make ext/secp256k1/.libs/libsecp256k1.a

-include Makefile

detect:## 	install sequence - Darwin and Linux
##detect
	bash -c "[ '$(shell uname -s)' == 'Darwin' ] && type -P brew >/tmp/gnostr.log && \
		export LIBRARY_PATH='$(LIBRARY_PATH):$(brew --prefix)/lib' || echo"
##	detect uname -s uname -m uname -p and install sequence

## 	Darwin
ifneq ($(shell id -u),0)
	@echo
	@echo $(shell id -u -n) 'not root'
	@echo
endif
	#bash -c "[ '$(shell uname -s)' == 'Darwin' ] && brew update                     || echo "
	bash -c "[ '$(shell uname -s)' == 'Darwin' ] && brew install automake            || echo "
	bash -c "[ '$(shell uname -s)' == 'Darwin' ] && brew install autoconf            || echo "
	bash -c "[ '$(shell uname -s)' == 'Darwin' ] && brew install automake            || echo "
##	bash -c "[ '$(shell uname -s)' == 'Darwin' ] && brew install boost               || echo "
	bash -c "[ '$(shell uname -s)' == 'Darwin' ] && brew install cmake --cask        || echo "
	bash -c "[ '$(shell uname -s)' == 'Darwin' ] && brew install coreutils           || echo "
	#bash -c "[ '$(shell uname -s)' == 'Darwin' ] && brew install --cask docker       || echo "
	#bash -c "[ '$(shell uname -s)' == 'Darwin' ] && brew install gcc                || echo "
	bash -c "[ '$(shell uname -s)' == 'Darwin' ] && brew install expat               || echo "
	bash -c "[ '$(shell uname -s)' == 'Darwin' ] && brew install gettext             || echo "
	bash -c "[ '$(shell uname -s)' == 'Darwin' ] && brew install git-archive-all     || echo "
	bash -c "[ '$(shell uname -s)' == 'Darwin' ] && brew install git-gui             || echo "
	bash -c "[ '$(shell uname -s)' == 'Darwin' ] && brew install glib-openssl        || echo "
	bash -c "[ '$(shell uname -s)' == 'Darwin' ] && brew install golang              || echo "
	bash -c "[ '$(shell uname -s)' == 'Darwin' ] && brew install help2man            || echo "
	bash -c "[ '$(shell uname -s)' == 'Darwin' ] && brew install libtool             || echo "
	bash -c "[ '$(shell uname -s)' == 'Darwin' ] && brew install mercurial           || echo "
	bash -c "[ '$(shell uname -s)' == 'Darwin' ] && brew install node@18             || echo "
	bash -c "[ '$(shell uname -s)' == 'Darwin' ] && brew install pandoc              || echo "
	bash -c "[ '$(shell uname -s)' == 'Darwin' ] && brew install pkg-config          || echo "
	bash -c "[ '$(shell uname -s)' == 'Darwin' ] && brew install protobuf            || echo "
	bash -c "[ '$(shell uname -s)' == 'Darwin' ] && brew install pipx                || echo "
	bash -c "[ '$(shell uname -s)' == 'Darwin' ] && brew install python3             || echo "
	bash -c "[ '$(shell uname -s)' == 'Darwin' ] && brew install rustup              || echo "
	bash -c "[ '$(shell uname -s)' == 'Darwin' ] && brew install scdoc               || echo "
	bash -c "[ '$(shell uname -s)' == 'Darwin' ] && brew install secp256k1           || echo "
	bash -c "[ '$(shell uname -s)' == 'Darwin' ] && brew install tcl-tk              || echo "
	bash -c "[ '$(shell uname -s)' == 'Darwin' ] && brew install virtualenv          || echo "
	bash -c "[ '$(shell uname -s)' == 'Darwin' ] && brew link --overwrite virtualenv || echo "
	bash -c "[ '$(shell uname -s)' == 'Darwin' ] && brew install zlib                || echo "
	#bash -c "[ '$(shell uname -s)' == 'Darwin' ] && /Applications/Docker.app/Contents/Resources/bin/docker system info || echo "







## 	Linux
ifneq ($(shell id -u),0)
	@echo
	@echo $(shell id -u -n) 'not root'
	@echo
endif
	bash -c "[ '$(shell uname -s)' == 'Linux' ] && \
		type -P brew >/tmp/gnostr.log && \
		export LIBRARY_PATH='$(LIBRARY_PATH):$(brew --prefix)/lib' || echo"
	bash -c "[ '$(shell uname -s)' == 'Linux' ] && \
		$(SUDO) apt-get update                     2>/dev/null || \
		apk add update || true"
	bash -c "[ '$(shell uname -s)' == 'Linux' ] && \
		$(SUDO) apt-get install autoconf           2>/dev/null || \
		apk add autoconf || true"
	bash -c "[ '$(shell uname -s)' == 'Linux' ] && \
		$(SUDO) apt-get install bison              2>/dev/null || \
		apk add bison || true"
	bash -c "[ '$(shell uname -s)' == 'Linux' ] && \
		$(SUDO) apt-get install bsdmainutils       2>/dev/null || \
		apk add util-linux || true"
	bash -c "[ '$(shell uname -s)' == 'Linux' ] && \
		$(SUDO) apt-get install build-essential    2>/dev/null || \
		apk add alpine-sdk || true"
	bash -c "[ '$(shell uname -s)' == 'Linux' ] && \
		$(SUDO) apt-get install cargo              2>/dev/null || \
		apk add cargo || true"
	bash -c "[ '$(shell uname -s)' == 'Linux' ] && \
		$(SUDO) apt-get install clang              2>/dev/null || \
		apk add clang || true"
	bash -c "[ '$(shell uname -s)' == 'Linux' ] && \
		$(SUDO) apt-get install cmake-curses-gui   2>/dev/null || \
		apk add extra-cmake-modules || true"
	bash -c "[ '$(shell uname -s)' == 'Linux' ] && \
		$(SUDO) apt-get install cmake              2>/dev/null || \
		apk add cmake || true"
	bash -c "[ '$(shell uname -s)' == 'Linux' ] && \
		$(SUDO) apt-get install expat              2>/dev/null || \
		apk add expat || true"
	bash -c "[ '$(shell uname -s)' == 'Linux' ] && \
		$(SUDO) apt-get install gettext            2>/dev/null || \
		apk add gettext || true"
	bash -c "[ '$(shell uname -s)' == 'Linux' ] && \
		$(SUDO) apt-get install golang-go          2>/dev/null || \
		apk add go || true"
	bash -c "[ '$(shell uname -s)' == 'Linux' ] && \
		$(SUDO) apt-get install help2man           2>/dev/null || \
		apk add help2man || true"
	bash -c "[ '$(shell uname -s)' == 'Linux' ] && \
		$(SUDO) apt-get install libcurl4-openssl-dev 2>/dev/null || \
		apk add curl-dev || true"
	bash -c "[ '$(shell uname -s)' == 'Linux' ] && \
		$(SUDO) apt-get install libssl-dev        2>/dev/null || \
		apk add openssl-dev || true"
	bash -c "[ '$(shell uname -s)' == 'Linux' ] && \
		$(SUDO) apt-get install libtool           2>/dev/null || \
		apk add libtool || true"
	bash -c "[ '$(shell uname -s)' == 'Linux' ] && \
		$(SUDO) apt-get install mercurial         2>/dev/null || \
		apk add mercurial || true"
	bash -c "[ '$(shell uname -s)' == 'Linux' ] && \
		$(SUDO) apt-get install npm               2>/dev/null || \
		apk add npm || true"
	bash -c "[ '$(shell uname -s)' == 'Linux' ] && \
		$(SUDO) apt-get install pandoc            2>/dev/null || \
		echo"
	bash -c "[ '$(shell uname -s)' == 'Linux' ] && \
		$(SUDO) apt-get install pipx              2>/dev/null || \
		echo"
	bash -c "[ '$(shell uname -s)' == 'Linux' ] && \
		$(SUDO) apt-get install pkg-config        2>/dev/null || \
		apk add pkgconfig || true"
	bash -c "[ '$(shell uname -s)' == 'Linux' ] && \
		$(SUDO) apt-get install protobuf-compiler 2>/dev/null || \
		apk add protobuf || true"
	bash -c "[ '$(shell uname -s)' == 'Linux' ] && \
		$(SUDO) apt-get install python3           2>/dev/null || \
		apk add python3 || true"
	bash -c "[ '$(shell uname -s)' == 'Linux' ] && \
		$(SUDO) apt-get install python3-pip       2>/dev/null || \
		apk add py3-pip || true"
	bash -c "[ '$(shell uname -s)' == 'Linux' ] && \
		$(SUDO) apt-get install python-is-python3 2>/dev/null || \
		echo   "
	bash -c "[ '$(shell uname -s)' == 'Linux' ] && \
		$(SUDO) apt-get install scdoc           2>/dev/null || \
		apk add scdoc || true"
	bash -c "[ '$(shell uname -s)' == 'Linux' ] && \
		$(SUDO) apt-get install tcl-dev           2>/dev/null || \
		apk add tcl-dev || true"
	bash -c "[ '$(shell uname -s)' == 'Linux' ] && \
		$(SUDO) apt-get install util-linux        2>/dev/null || \
		apk add util-linux || true"
	bash -c "[ '$(shell uname -s)' == 'Linux' ] && \
		$(SUDO) apt-get install virtualenv        2>/dev/null || \
		apk add py3-virtualenv || true"
	bash -c "[ '$(shell uname -s)' == 'Linux' ] && \
		$(SUDO) apt-get install zlib1g-dev        2>/dev/null || \
		echo   "

##	install gvm sequence
	@rm -rf $(HOME)/.gvm || echo "not removing ~/.gvm"
	@bash -c "bash < <(curl -s -S -L https://raw.githubusercontent.com/moovweb/gvm/master/binscripts/gvm-installer) || echo 'not installing gvm...'"
	bash -c "[ '$(shell uname -m)' == 'x86_64' ] && echo 'is x86_64' || echo 'not x86_64';"
	bash -c "[ '$(shell uname -m)' == 'arm64' ] && [ '$(shell uname -s)' == 'Darwin' ] && type -P brew && brew install pandoc || echo 'not arm64 AND Darwin';"
	bash -c "[ '$(shell uname -m)' == 'i386' ] && echo 'is i386' || echo 'not i386';"

##	install rustup sequence
	$(shell echo which rustup) || curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | bash -s -- -y --no-modify-path --default-toolchain stable --profile default #& . "$(HOME)/.cargo/env"

##	install nvm sequence
	@bash -c "curl https://raw.githubusercontent.com/creationix/nvm/master/install.sh | bash && export NVM_DIR='$(HOME)/.nvm'; [ -s '$(NVM_DIR)/nvm.sh' ] && \. '$(NVM_DIR)/nvm.sh'; [ -s '$(NVM_DIR)/bash_completion' ] && \. '$(NVM_DIR)/bash_completion' &"

	bash -c "which autoconf                   || echo "
	bash -c "which automake                   || echo "
	bash -c "which brew                       || echo "
	bash -c "which cargo                      || echo "
	bash -c "which cmake                      || echo "
	bash -c "which go                         || echo "
	bash -c "which node                       || echo "
	bash -c "which rustup                     || echo "
