CFLAGS                                  = -Wall -O2 -Isecp256k1/include
CFLAGS                                 += -I/include
LDFLAGS                                 = -Wl
GNOSTR_OBJS                             = gnostr.o       sha256.o aes.o base64.o
#GNOSTR_GIT_OBJS                         = gnostr-git.o   sha256.o aes.o base64.o libgit.a
#GNOSTR_RELAY_OBJS                       = gnostr-relay.o sha256.o aes.o base64.o
#GNOSTR_XOR_OBJS                         = gnostr-xor.o   sha256.o aes.o base64.o libsecp256k1.a
HEADER_INCLUDE                          = include
HEADERS                                 = $(HEADER_INCLUDE)/hex.h \
                                         $(HEADER_INCLUDE)/random.h \
                                         $(HEADER_INCLUDE)/config.h \
                                         $(HEADER_INCLUDE)/sha256.h \
                                         secp256k1/include/secp256k1.h

ifneq ($(prefix),)
	PREFIX                             :=$(prefix)
else
	PREFIX                             :=/usr/local
endif
#export PREFIX

ARS                                    := libsecp256k1.a
LIB_ARS                                := libsecp256k1.a libgit.a


SUBMODULES=$(shell cat .gitmodules | grep path | cut -d ' ' -f 3 | sed 's/bits//g' | sed 's/cli//g')

VERSION                                :=$(shell cat version)
export VERSION
GTAR                                   :=$(shell which gtar || true)
export GTAR
TAR                                    :=$(shell which tar || true)
export TAR
ifeq ($(GTAR),)
#we prefer gtar but...
GTAR                                   :=$(shell which tar || true)
endif
export GTAR

##skip gnostr-act gnostr-cat gnostr-git
DOCS=\
gnostr-bits\
gnostr-blockheight\
gnostr-cli\
gnostr-client\
gnostr-db\
gnostr-db-cli\
gnostr-get-relays\
gnostr-git-log\
gnostr-git-reflog\
gnostr-gnode\
gnostr-keyconv\
gnostr-modal\
gnostr-nip\
gnostr-post\
gnostr-post-event\
gnostr-proxy\
gnostr-query\
gnostr-relays\
gnostr-req\
gnostr-send\
gnostr-set-relays\
gnostr-sha256\
gnostr-tests\
gnostr-tui\
gnostr-weeble\
gnostr-wobble\
gnostr\

##all:
#all: submodules gnostr gnostr-git gnostr-docs## 	make gnostr gnostr-cat gnostr-git gnostr-relay gnostr-xor docs
all: submodules gnostr gnostr-git ##gnostr-docs## 	make gnostr gnostr-cat gnostr-git gnostr-relay gnostr-xor docs
##	build gnostr tool and related dependencies

##gnostr-docs:
##	docker-start doc/gnostr.1
gnostr-docs:doc/gnostr.1 doc## 	docs: convert README to doc/gnostr.1
#@echo docs
	@bash -c 'if pgrep MacDown; then pkill MacDown; fi; 2>/dev/null'
	@bash -c 'cat $(PWD)/sources/HEADER.md                >  $(PWD)/README.md 2>/dev/null'
	@bash -c 'cat $(PWD)/sources/COMMANDS.md              >> $(PWD)/README.md 2>/dev/null'
	@bash -c 'cat $(PWD)/sources/FOOTER.md                >> $(PWD)/README.md 2>/dev/null'
	##@type -P pandoc && pandoc -s README.md -o index.html 2>/dev/null || \
	##	type -P docker && docker pull pandoc/latex:2.6 && \
	##	docker run --rm --volume "`pwd`:/data" --user `id -u`:`id -g` pandoc/latex:2.6 README.md
	git add --ignore-errors sources/*.md 2>/dev/null || echo && git add --ignore-errors *.md 2>/dev/null || echo
#@git ls-files -co --exclude-standard | grep '\.md/$\' | xargs git

doc-gnostr-act:gnostr-act
	$(MAKE) gnostr-act || false
	help2man gnostr-act | sed 's/act /gnostr\-act /g' | sed 's/ACT /GNOSTR\-ACT /g' > doc/gnostr-act.1 && cat doc/gnostr-act.1 > $(PREFIX)/share/man/man1/gnostr-act.1
doc-gnostr-cat:gnostr-cat
	$(MAKE) gnostr-cat || false
	help2man gnostr-cat > doc/gnostr-cat.1 && cat doc/gnostr-cat.1 > $(PREFIX)/share/man/man1/gnostr-cat.1
doc-gnostr-git:gnostr-git
	$(MAKE) gnostr-git || false
	help2man gnostr-git | sed 's/ git / gnostr\-git /g' | sed 's/ GIT / GNOSTR\-GIT /g' > doc/gnostr-git.1 && cat doc/gnostr-git.1 > $(PREFIX)/share/man/man1/gnostr-git.1
.PHONY:doc
##We stream edit certain tools man pages
##	make goals are processed from left to right
##	doc-gnostr-act doc-gnostr-cat doc-gnostr-git gnostr-install##
docs:doc-gnostr-act doc-gnostr-cat doc-gnostr-git gnostr-install## 	doc - generate man pages
##help2man < $^ > $@
	##[[ -x "$(shell which gnostr-act)" ]] || $(MAKE) doc-gnostr-act
	@(\
	for b in $(DOCS);\
	do touch doc/$$b.1;\
	done;\
	exit;\
	)
	(\
	for b in $(DOCS);\
	do echo doc/$$b.1 > /tmp/make-doc.log;\
	done;\
	exit;\
	)
	(\
	for b in $(DOCS);\
	do help2man $$b > doc/$$b.1;\
	install -m 0644 -v doc/$$b.1 $(PREFIX)/share/man/man1/$$b.1;\
	echo $$b;\
	done;\
	exit;\
	)
	#for b in $(DOCS); do echo $b; done; exit
	#for b in $(DOCS); do touch doc/$(DOCS); done;exit
	#for n in $(DOCS); do touch doc/$n.1; done
	#bash -c "for n in $(ls gnostr-*   ); do touch doc/$n.1; done"
	#for n in $(DOCS); do [ -x $n ] &&  help2man $n > doc/$n.1 || true; done
	#bash -c "for n in $(ls gnostr-* ); do [ -x $n ] &&  help2man $n > doc/$n.1 || true; done"

.PHONY: version
version: src/gnostr.c## 	print version
	@grep '^#define VERSION' $< | sed -En 's,.*"([^"]+)".*,\1,p' > $@
#	@cat $@
.PHONY:GIT-VERSION-FILE git-version
git-version:GIT-VERSION-FILE
GIT-VERSION-FILE:git/GIT-VERSION-FILE
	@. git/GIT-VERSION-GEN
	@grep '^GIT_VERSION' <GIT-VERSION-FILE | sed -En 's,..............([^"]+).*,\1,p' > git-version
	@cat git-version #&& rm GIT-VERSION-FILE
.PHONY:versions
versions:version git-version

chmod:## 	chmod
##chmod
## 	find . -type f ! -name 'deps/**' -print0     | xargs -0 chmod 644
## 	find . -type f ! -name 'deps/**' --name *.sh | xargs -0 chmod +rwx
## 	find . -type d ! -name 'deps/**' -print0     | xargs -0 chmod 755
## 	if issues or before 'make dist'
##all files first
#find . -type f -print0 -maxdepth 2
	find . -type f -print0 -maxdepth 2 | xargs -0 chmod 0644
##*.sh template/gnostr-* executables
#find . -type f -name '*.sh' -name 'template/gnostr-*' -maxdepth 2
	find . -type f -name '*.sh' -name 'template/gnostr-*' -maxdepth 2 | xargs -0 chmod +rwx
##not deps not configurator* not .venv
#find . -type d ! -name 'deps' ! -name 'configurator*' ! -name '.venv' -print0 -maxdepth 1
	find . -type d ! -name 'deps' ! -name 'configurator*' ! -name '.venv' -print0 -maxdepth 1 | xargs -0 chmod 0755
	chmod +rwx devtools/refresh-submodules.sh

diff-log:
	@mkdir -p tests && diff template/gnostr-git-reflog template/gnostr-git-log > tests/diff.log || \
		git diff tests/diff.log
	@gnostr -h > tests/gnostr-h.log
	@gnostr-git -h > tests/gnostr-git-h.log
	@gnostr-git-log -h > tests/gnostr-git-log-h.log
	@gnostr-git-reflog -h > tests/gnostr-git-reflog-h.log
	@gnostr-relay -h > tests/gnostr-relay-h.log
.PHONY:submodules
submodules:### recursively initialize git submodules
##gnostr-bits needs ~/bin
	mkdir -p ~/bin
	git submodule update --init --recursive || echo "install git..."
	$(MAKE) $(SUBMODULES)

.PHONY:secp256k1/.git
.ONESHELL:
secp256k1/.git:
	devtools/refresh-submodules.sh secp256k1
secp256k1/include/secp256k1.h:secp256k1/.git
## force configure if build on host then in docker vm
#.PHONY:secp256k1/configure## 	This MUST be PHONY for docker builds
secp256k1/configure:secp256k1/.git
secp256k1/.libs/libsecp256k1.a:

secp256k1/configure:
	cd secp256k1 && \
	git fetch --all && git checkout c-lang && \
		./autogen.sh

secp256k1/.libs/libsecp256k1.a:secp256k1/configure
	cd secp256k1 && \
	git fetch --all && \
	git checkout c-lang && \
		./configure \
		--enable-module-ecdh \
		--enable-module-schnorrsig \
		--enable-module-extrakeys \
		--disable-benchmark \
		--disable-tests && \
		make -j

secp256k1/src/libsecp256k1.a:secp256k1/configure
	cd secp256k1 && \
	git fetch --all && \
	git checkout c-lang && \
		./configure \
		--enable-module-ecdh \
		--enable-module-schnorrsig \
		--enable-module-extrakeys \
		--disable-benchmark \
		--disable-tests && \
		make -j

libsecp256k1.a:secp256k1/.libs/libsecp256k1.a
	cp $< $@ #|| $(MAKE) secp256k1/src/libsecp256k1.a || $(MAKE) secp256k1/.libs/libsecp256k1.a

.PHONY:secp256k1
secp256k1:libsecp256k1.a

jq/modules/oniguruma.git:
	devtools/refresh-submodules.sh jq
jq/.git:jq/modules/oniguruma.git
jq/.libs/libjq.a:jq/.git
	cd jq && \
		autoreconf -fi && ./configure --disable-maintainer-mode && make install gnostr-jq-install && cd ../..
jq/gnostr-jq:jq/.libs/libjq.a
	cd jq && \
		autoreconf -fi && ./configure --disable-maintainer-mode && make install gnostr-jq-install && cd ../..
##libjq.a
##	cp $< jq/libjq.a .
libjq.a: jq/.libs/libjq.a## 	libjq.a
	cp $< $@
#.PHONY:gnostr-jq jq/gnostr-jq
gnostr-jq:jq/gnostr-jq
	cp $< $@




.PHONY:web
web-deploy:gnostr-web-deploy
web:
	(\
	if [ ! $(uname -s) = 'Darwin' ]; then \
		if grep -q Microsoft /proc/version; then \
			alias open='explorer.exe'; \
		else \
			alias open='xdg-open'; \
		fi \
	fi \
	)
	#bash -c "echo $(shell which open)"

	@cmake .  -DWT_INCLUDE="${WX_PREFIX}/lib/include" -DWT_CONFIG_H="${WX_PREFIX}/include" -DBUILD_WEB=ON -DBUILD_GUI=OFF -DCMAKE_C_FLAGS=-g -DCMAKE_BUILD_TYPE=Release
	@$(MAKE) gnostr-web
gnostr-web-deploy:
	gnostr-web --http-address=0.0.0.0 --http-port=80 --deploy-path=/web --docroot=. & \
    open http://0.0.0.0:80



.PHONY:git/targets/release/gnostr-git git/.git
git/.git:
	@devtools/refresh-submodules.sh git
git/targets/release/gnostr-git:git/.git
	cd git && make cargo-install
git:gnostr-git
gnostr-git:git/targets/release/gnostr-git
	$(MAKE) gnostr-install
	#cp $< $@ || true
	#install $@ /usr/local/bin/

ext/curl-8.5.0/src/curl:
	cd ext/curl-8.5.0 && make install
gnostr-curl:ext/curl-8.5.0/src/curl
	cp $< $@ || true
	install $@ /usr/local/bin/

gnostr-cargo-binstall:
	type -P cargo && \
		cargo install cargo-binstall && \
		cargo-binstall \
		--no-confirm \
		--no-discover-github-token \
		gnostr-cat \
		gnostr-cli \
		gnostr-grep \
		gnostr-legit \
		gnostr-sha256

.PHONY:gnostr-build
gnostr-build:## 	gnostr-build
	rm build/CMakeCache.txt || echo
	cmake -S . -B build && cd build && cmake ../ && make



.PHONY:gnostr-build-install
gnostr-build-install:gnostr-build## 	gnostr-build-install
	cd build && make all install && install gnostr-tests /usr/local/bin || echo
	$(MAKE) gnostr-install || echo

##.PHONY:command gnostr-command
##command/.git:gnostr-git
##	@devtools/refresh-submodules.sh command
##gnostr-command:command
##command:command/.git
##	cd command && \
##		make rustup-install-stable && \
##		make cargo-br-async-std

.PHONY:bins gnostr-bins
gnostr-bins:bins
bins: cli/.git nips/.git
	cargo install --path $@
.PHONY:cat gnostr-cat
gnostr-cat:cat
cat:
	cargo install --path $@

.PHONY:lookup gnostr-lookup
#lookup/.git:
#	@devtools/refresh-submodules.sh lookup
gnostr-lookup:lookup
lookup:#bins/.git
	cargo install --path lookup --force

.PHONY:xq gnostr-xq

.PHONY:xq gnostr-xq
gnostr-xq:xq
xq:
	@cargo install --path xq

.PHONY:core gnostr-core
core/.git:
	@devtools/refresh-submodules.sh core
gnostr-core:core
core:core/.git
	@cd core && make cargo-br

.PHONY:py gnostr-py
py/.git:
	@devtools/refresh-submodules.sh py
gnostr-py:py
py:py/.git venv
	$(. .venv/bin/activate & cd py && make)

bins-test-post-event:
	cat test/first-gnostr-commit.json | gnostr-post-event wss://relay.damus.io
bins-test-fetch-by-id:
		gnostr-fetch-by-id wss://relay.damus.io fbf73a17a4e0fe390aba1808a8d55f1b50717d5dd765b2904bf39eba18c51f7c | jq || true
		#gnostr-fetch-by-id wss://relay.damus.io fbf73a17a4e0fe390aba1808a8d55f1b50717d5dd765b2904bf39eba18c51f7c | jq .content || true

.PHONY:ffi gnostr-ffi
ffi/.git:
	@devtools/refresh-submodules.sh ffi
ffi:
	@cd ffi && make all && cd .. || echo "make ffi failed..."
gnostr-ffi:ffi

.PHONY:gui gnostr-gui
gnostr-gui:gui
gui:
	@cargo install --path gui

.PHONY:bits gnostr-bits
gnostr-bits:bits
bits/.git:
	@devtools/refresh-submodules.sh bits
bits:bits/.git
	@cd bits && make build-release install && cd ..
bits-desktop:bits/.git
	@cd bits && make desktop && cd ..
.PHONY:modal gnostr-modal
gnostr-modal:modal
modal:
	@devtools/refresh-submodules.sh modal
	@cd modal && make build-release install && cd ..
.PHONY:tui gnostr-tui
gnostr-tui:tui
tui:
	#@devtools/refresh-submodules.sh tui
	@cd tui && make cargo-i && cd ..
.PHONY:db gnostr-db
gnostr-db:db
db:
	@devtools/refresh-submodules.sh db
	@cd db && make build-release install && cd ..

.PHONY:gnostr-legit legit
legit:gnostr-legit
gnostr-legit:## 	gnostr-legit
	@cargo install --path ./bins --bin gnostr-legit --force
	##cp $< $@ && exit;
	install -v template/gnostr-* /usr/local/bin >/tmp/gnostr-legit.log

.PHONY:sha256.git sha256
sha256/.git:
	@devtools/refresh-submodules.sh sha256
#.PHONY:sha256/gnostr-sha256
sha256:sha256/.git
	cargo install --path sha256 --force
gnostr-sha256:sha256



deps/gnostr-org/.git:
	@devtools/refresh-submodules.sh deps/gnostr-org
.PHONY:deps/gnostr-org
deps/gnostr-org:deps/gnostr-org/.git
	cd deps/gnostr-org && \
		$(MAKE) install
gnostr-org:deps/gnostr-org
	install deps/gnostr-org/gnostr-org template
	install deps/gnostr-org/gnostr-org /usr/local/bin



.PHONY:proxy
proxy/.git:
	@devtools/refresh-submodules.sh proxy

gnostr-proxy:proxy## 	gnostr-proxy
proxy:proxy/.git
	install ./proxy/gnostr-proxy template
	install ./proxy/gnostr-proxy-relay-list template
	install ./proxy/gnostr-proxy /usr/local/bin
	install ./proxy/gnostr-proxy-relay-list /usr/local/bin
	cd proxy && \
		$(MAKE) install

.PHONY:nips
nips/.git:
	@devtools/refresh-submodules.sh nips
nips: nips/.git
	cargo install --path $@

.PHONY: relay gnostr-relay
relay/.git:
	@devtools/refresh-submodules.sh relay
gnostr-relay:relay
relay:relay/.git
	cd relay && \
		make cargo-install

.PHONY:gnostr-cli cli
cli/.git:
	@devtools/refresh-submodules.sh cli
.PHONY:cli/target/release/gnostr-cli
gnostr-cli:cli
cli:cli/.git
	cd cli && \
		make cargo-build-release cargo-i
.PHONY:gnostr-cli cli

.PHONY:gnostrd d
gnostrd:d
d:
	cargo install --path $@

.PHONY:gnostrd/target/release/gnostrd
gnostrd/target/release/gnostrd:
	cd d && \
		cargo b -r --bin gnostrd && \
		cargo install --bin gnostrd --path . --force
.PHONY:gnostrd/target/release/gnostr-chat
gnostrd/target/release/gnostr-chat:
	cd d && \
		cargo b -r --bin gnostr-chat && \
		cargo install --bin gnostr-chat --path . --force
gnostr-chat:chat gnostrd
chat:gnostrd/target/release/gnostr-chat

.PHONY:grep/.git gnostr-grep grep
grep/.git:
	@devtools/refresh-submodules.sh grep
.PHONY:grep/target/release/gnostr-grep
gnostr-grep:grep
grep:grep/.git
	cd grep && \
		make cargo-install
.PHONY:gnostr-grep grep






#deps/hyper-sdk/.git:
#	@devtools/refresh-submodules.sh deps/hyper-sdk
#deps/hyper-nostr/.git:
#	@devtools/refresh-submodules.sh deps/hyper-nostr
deps/openssl/.git:
	@devtools/refresh-submodules.sh deps/openssl
deps/gnostr-py/.git:
	@devtools/refresh-submodules.sh deps/gnostr-py
deps/gnostr-aio/.git:
	@devtools/refresh-submodules.sh deps/gnostr-aio



.PHONY:act gnostr-act
act/.git:
	@devtools/refresh-submodules.sh act
gnostr-act:act
act/bin/gnostr-act:act/.git
act:act/bin/gnostr-act
	cd act && ./install.sh || ./install-gnostr-act

src/libcjson/.git:
	@devtools/refresh-submodules.sh src/libcjson
src/libcjson:src/libcjson/.git
	cd src/libcjson && make
src/libcjson/bin/libcjson.a:src/libcjson
libcjson:src/libcjson/bin/libcjson.a
	cp $^ .

#gnostr-am
#gnostr-am
gnostr-am:$(HEADERS) $(GNOSTR_OBJS) $(ARS) ## 	make gnostr binary
	$(CC) $(CFLAGS) $(GNOSTR_OBJS) $(ARS) -o $@ && $(MAKE) gnostr-install
#gnostr
#gnostr
gnostr:$(HEADERS) $(GNOSTR_OBJS) $(ARS)## 	make gnostr binary
	$(CC) $(CFLAGS) $(GNOSTR_OBJS) $(ARS) -o $@ && $(MAKE) gnostr-install



#gnostr-relay:initialize $(HEADERS) $(GNOSTR_RELAY_OBJS) $(ARS)## 	make gnostr-relay
###gnostr-relay
#	git submodule update --init --recursive
#	$(CC) $(CFLAGS) $(GNOSTR_RELAY_OBJS) $(ARS) -o $@

## #.PHONY:gnostr-xor
## gnostr-xor: $(HEADERS) $(GNOSTR_XOR_OBJS) $(ARS)## 	make gnostr-xor
## ##gnostr-xor
## 	echo $@
## 	touch $@
## 	rm -f $@
## 	$(CC) $@.c -o $@

.ONESHELL:gnostr-install
##install all
##	install doc/gnostr.1 gnostr gnostr-query
gnostr-install:
	mkdir -p $(PREFIX)/bin                                                         || true
	mkdir -p $(PREFIX)/include                                                     || true
	@install -m755 -v include/*.*                    $(PREFIX)/include 2>/dev/null || true
	@install -m755 -v gnostr                         $(PREFIX)/bin     2>/dev/null || echo "Try:\nmake gnostr"
	@install -m755 -v gnostr-client                  $(PREFIX)/bin     2>/dev/null || echo "Try:\nmake gnostr"
	@install -m755 -v gnostr-am                      $(PREFIX)/bin     2>/dev/null || echo "Try:\nmake gnostr"
	@install -m755 -v template/gnostr-*              $(PREFIX)/bin     2>/dev/null || true
	@install -m755 -v template/gnostr-query          $(PREFIX)/bin     2>/dev/null || true
	@install -m755 -v template/gnostr-set-relays     $(PREFIX)/bin     2>/dev/null || true
	@install -m755 -v template/gnostr-*-*            $(PREFIX)/bin     2>/dev/null || true
	@install -m755 -v ext/curl-8.5.0/src/gnostr-curl $(PREFIX)/bin     2>/dev/null || true
	@install -m755 -v web/gnostr-*                   $(PREFIX)/bin     2>/dev/null || true

.ONESHELL:
##install-doc
##	install-doc
install-doc:## 	install-doc
## 	install -m 0644 -vC doc/gnostr.1 $(PREFIX)/share/man/man1/gnostr.1
	@mkdir -p /usr/local/share/man/man1 || echo "try 'sudo mkdir -p /usr/local/share/man/man1'"
	@install -m 0644 -v doc/gnostr.1 $(PREFIX)/share/man/man1/gnostr.1 || \
		echo "doc/gnostr.1 failed to install..."
	@install -m 0644 -v doc/gnostr-about.1 $(PREFIX)/share/man/man1/gnostr-about.1 || \
		echo "doc/gnostr-about.1 failed to install..."
	@install -m 0644 -v doc/gnostr-act.1 $(PREFIX)/share/man/man1/gnostr-act.1 || \
		echo "doc/gnostr-act.1 failed to install..."
	@install -m 0644 -v doc/gnostr-cat.1 $(PREFIX)/share/man/man1/gnostr-cat.1 || \
		echo "doc/gnostr-cat.1 failed to install..."
	@install -m 0644 -v doc/gnostr-cli.1 $(PREFIX)/share/man/man1/gnostr-cli.1 || \
		echo "doc/gnostr-cli.1 failed to install..."
	@install -m 0644 -v doc/gnostr-client.1 $(PREFIX)/share/man/man1/gnostr-client.1 || \
		echo "doc/gnostr-client.1 failed to install..."
	@install -m 0644 -v doc/gnostr-get-relays.1 $(PREFIX)/share/man/man1/gnostr-get-relays.1 || \
		echo "doc/gnostr-get-relays.1 failed to install..."
	@install -m 0644 -v doc/gnostr-git-log.1 $(PREFIX)/share/man/man1/gnostr-git-log.1 || \
		echo "doc/gnostr-git-log.1 failed to install..."
	@install -m 0644 -v doc/gnostr-git-reflog.1 $(PREFIX)/share/man/man1/gnostr-git-reflog.1 || \
		echo "doc/gnostr-git-reflog.1 failed to install..."
	@install -m 0644 -v doc/gnostr-git.1 $(PREFIX)/share/man/man1/gnostr-git.1 || \
		echo "doc/gnostr-git.1 failed to install..."
	@install -m 0644 -v doc/gnostr-gnode.1 $(PREFIX)/share/man/man1/gnostr-gnode.1 || \
		echo "doc/gnostr-gnode.1 failed to install..."
	@install -m 0644 -v doc/gnostr-grep.1 $(PREFIX)/share/man/man1/gnostr-grep.1 || \
		echo "doc/gnostr-grep.1 failed to install..."
	@install -m 0644 -v doc/gnostr-hyp.1 $(PREFIX)/share/man/man1/gnostr-hyp.1 || \
		echo "doc/gnostr-hyp.1 failed to install..."
	@install -m 0644 -v doc/gnostr-legit.1 $(PREFIX)/share/man/man1/gnostr-legit.1 || \
		echo "doc/gnostr-legit.1 failed to install..."
	@install -m 0644 -v doc/gnostr-nonce.1 $(PREFIX)/share/man/man1/gnostr-nonce.1 || \
		echo "doc/gnostr-nonce.1 failed to install..."
	@install -m 0644 -v doc/gnostr-post.1 $(PREFIX)/share/man/man1/gnostr-post.1 || \
		echo "doc/gnostr-post.1 failed to install..."
	@install -m 0644 -v doc/gnostr-proxy.1 $(PREFIX)/share/man/man1/gnostr-proxy.1 || \
		echo "doc/gnostr-proxy.1 failed to install..."
	@install -m 0644 -v doc/gnostr-query.1 $(PREFIX)/share/man/man1/gnostr-query.1 || \
		echo "doc/gnostr-query.1 failed to install..."
	@install -m 0644 -v doc/gnostr-readme.1 $(PREFIX)/share/man/man1/gnostr-readme.1 || \
		echo "doc/gnostr-readme.1 failed to install..."
	@install -m 0644 -v doc/gnostr-relays.1 $(PREFIX)/share/man/man1/gnostr-relays.1 || \
		echo "doc/gnostr-relays.1 failed to install..."
	@install -m 0644 -v doc/gnostr-repo.1 $(PREFIX)/share/man/man1/gnostr-repo.1 || \
		echo "doc/gnostr-repo.1 failed to install..."
	@install -m 0644 -v doc/gnostr-req.1 $(PREFIX)/share/man/man1/gnostr-req.1 || \
		echo "doc/gnostr-req.1 failed to install..."
	@install -m 0644 -v doc/gnostr-send.1 $(PREFIX)/share/man/man1/gnostr-send.1 || \
		echo "doc/gnostr-send.1 failed to install..."
	@install -m 0644 -v doc/gnostr-set-relays.1 $(PREFIX)/share/man/man1/gnostr-set-relays.1 || \
		echo "doc/gnostr-set-relays.1 failed to install..."
	@install -m 0644 -v doc/gnostr-sha256.1 $(PREFIX)/share/man/man1/gnostr-sha256.1 || \
		echo "doc/gnostr-sha256.1 failed to install..."
	@install -m 0644 -v doc/gnostr-tests.1 $(PREFIX)/share/man/man1/gnostr-tests.1 || \
		echo "doc/gnostr-tests.1 failed to install..."
	@install -m 0644 -v doc/gnostr-web.1 $(PREFIX)/share/man/man1/gnostr-web.1 || \
		echo "doc/gnostr-web.1 failed to install..."
	@install -m 0644 -v doc/gnostr-weeble.1 $(PREFIX)/share/man/man1/gnostr-weeble.1 || \
		echo "doc/gnostr-weeble.1 failed to install..."
	@install -m 0644 -v doc/gnostr-wobble.1 $(PREFIX)/share/man/man1/gnostr-wobble.1 || \
		echo "doc/gnostr-wobble.1 failed to install..."

.PHONY:config.h
gnostr-config.h: configurator
	./configurator > $@

.PHONY:configurator
##configurator
##	rm -f configurator
##	$(CC) $< -o $@
gnostr-configurator: configurator.c
	rm -f configurator
	$(CC) $< -o $@

gnostr-query:
	@install -m755 -vC template/gnostr-query          $(PREFIX)/bin     2>/dev/null

gnostr-query-test:gnostr-cat gnostr-query gnostr-install
	./template/gnostr-query -t gnostr | gnostr-cat -u wss://relay.damus.io
	./template/gnostr-query -t weeble | gnostr-cat -u wss://relay.damus.io
	./template/gnostr-query -t wobble | gnostr-cat -u wss://relay.damus.io
	./template/gnostr-query -t blockheight | gnostr-cat -u wss://relay.damus.io

## 	for CI purposes we build largest apps last
## 	rust based apps first after gnostr
## 	nodejs apps second
##
## 	gnostr-legit relies on gnostr-git and gnostr-install sequence
##
gnostr-all:
	rm CMakeCache.txt || echo
	cmake .
	$(MAKE) -j libsecp256k1.a
	type -P gnostr         || $(MAKE) -j gnostr
	$(MAKE) -j gnostr-install
	type -P gnostr-post-event || $(MAKE) -j bins
	#type -P gnostr-tui        || $(MAKE) -j tui
	type -P gnostr-cat     || $(MAKE) -j gnostr-cat
	type -P gnostr-cli     || $(MAKE) -j gnostr-cli
	type -P gnostr-curl    || $(MAKE) -j gnostr-curl
	type -P gnostr-grep    || $(MAKE) -j gnostr-grep
	type -P gnostr-sha256  || $(MAKE) -j gnostr-sha256
#type -P gnostr-command || $(MAKE) -j gnostr-command
	type -P gnostr-proxy   || $(MAKE) -j gnostr-proxy
	type -P gnostr-query   || $(MAKE) -j gnostr-query
	type -P gnostr-git     || $(MAKE) -j gnostr-git
	type -P gnostr-legit   || $(MAKE) -j gnostr-legit
	type -P gnostr-client  || $(MAKE) -j gnostr-client
	type -P gnostr-gnode   || $(MAKE) -j gnostr-gnode
	$(MAKE) -j gnostr-build
	$(MAKE) -j gnostr-install
	type -P gnostr-act && echo "'make gnostr-act' to install gnostr-act " || $(MAKE) -j gnostr-act

## git log $(git describe --tags --abbrev=0)...@^1

dist: gnostr-docs version## 	create tar distribution
	source .venv/bin/activate; pip install -r requirements.txt;
	test -d .venv || python3 -m virtualenv .venv;
	( \
	   python3 -m pip install -r requirements.txt;\
	   python3 -m pip install git-archive-all;\
	   mv dist dist-$(VERSION)-$(OS)-$(ARCH)-$(TIME) || echo;\
	   mkdir -p dist && touch dist/.gitkeep;\
	   cat version > CHANGELOG && git add -f CHANGELOG && git commit -m "CHANGELOG: update" 2>/dev/null || echo;\
	   git log $(shell git describe --tags --abbrev=0)...@^1 --oneline  >> CHANGELOG;\
	   cp CHANGELOG dist/CHANGELOG.txt;\
	   git-archive-all -C . dist/gnostr-$(VERSION)-$(OS)-$(ARCH).tar >/tmp/gnostr-make-dist.log;\
	);

dist-sign:## 	dist-sign
	cd dist && \touch SHA256SUMS-$(VERSION)-$(OS)-$(ARCH).txt && \
		touch gnostr-$(VERSION)-$(OS)-$(ARCH).tar.gz && \
		rm **SHA256SUMS**.txt** || echo && \
		sha256sum gnostr-$(VERSION)-$(OS)-$(ARCH).tar.gz > SHA256SUMS-$(VERSION)-$(OS)-$(ARCH).txt && \
		gpg -u 0xE616FA7221A1613E5B99206297966C06BB06757B \
		--sign --armor --detach-sig --output SHA256SUMS-$(VERSION)-$(OS)-$(ARCH).txt.asc SHA256SUMS-$(VERSION)-$(OS)-$(ARCH).txt
#rsync -avzP dist/ charon:/www/cdn.jb55.com/tarballs/gnostr/
dist-test:submodules dist## 	dist-test
##dist-test
## 	cd dist and run tests on the distribution
	cd dist && \
		$(GTAR) -tvf gnostr-$(VERSION)-$(OS)-$(ARCH).tar > gnostr-$(VERSION)-$(OS)-$(ARCH).tar.txt
	cd dist && \
		$(GTAR) -xf  gnostr-$(VERSION)-$(OS)-$(ARCH).tar && \
		cd  gnostr-$(VERSION)-$(OS)-$(ARCH) && cmake . && make chmod all


.PHONY:ext/boost_1_82_0/.git
ext/boost_1_82_0/.git:
	[ ! -d ext/boost_1_82_0 ] && git clone -b boost-1.82.0 --recursive https://github.com/boostorg/boost.git ext/boost_1_82_0 || cd ext/boost_1_82_0 && git reset --hard
.PHONY:ext/boost_1_82_0
ext/boost_1_82_0:ext/boost_1_82_0/.git
	cd ext/boost_1_82_0 && ./bootstrap.sh && ./b2 && ./b2 headers
boost:ext/boost_1_82_0
boostr:boost



%.o: src/%.c $(HEADERS)
	@echo "cc $<"
	@$(CC) $(CFLAGS) -c $< -o $@


#.PHONY: fake
