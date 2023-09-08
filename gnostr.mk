CFLAGS                                  = -Wall -O2 -Ideps/secp256k1/include
CFLAGS                                 += -I/include
LDFLAGS                                 = -Wl -V
GNOSTR_OBJS                             = gnostr.o       sha256.o aes.o base64.o libsecp256k1.a
#GNOSTR_GIT_OBJS                         = gnostr-git.o   sha256.o aes.o base64.o libgit.a
#GNOSTR_RELAY_OBJS                       = gnostr-relay.o sha256.o aes.o base64.o
#GNOSTR_XOR_OBJS                         = gnostr-xor.o   sha256.o aes.o base64.o libsecp256k1.a
HEADER_INCLUDE                          = include
HEADERS                                 = $(HEADER_INCLUDE)/hex.h \
                                         $(HEADER_INCLUDE)/random.h \
                                         $(HEADER_INCLUDE)/config.h \
                                         $(HEADER_INCLUDE)/sha256.h \
                                         deps/secp256k1/include/secp256k1.h

ifneq ($(prefix),)
	PREFIX                             :=$(prefix)
else
	PREFIX                             :=/usr/local
endif
#export PREFIX

ARS                                    := libsecp256k1.a
LIB_ARS                                := libsecp256k1.a libgit.a

#SUBMODULES                              = deps/secp256k1
SUBMODULES                              = deps/secp256k1 deps/git deps/gnostr-cat deps/hyper-nostr deps/hyper-sdk deps/gnostr-act deps/openssl deps/gnostr-py deps/gnostr-aio deps/gnostr-legit deps/gnostr-relay deps/gnostr-proxy deps/gnostr-relay ext/boost_1_82_0

VERSION                                :=$(shell cat version)
export VERSION
GTAR                                   :=$(shell which gtar)
export GTAR
TAR                                    :=$(shell which tar)
export TAR
ifeq ($(GTAR),)
#we prefer gtar but...
GTAR                                   :=$(shell which tar)
endif
export GTAR


##all:
#all: submodules gnostr gnostr-git gnostr-get-relays gnostr-docs## 	make gnostr gnostr-cat gnostr-git gnostr-relay gnostr-xor docs
all: submodules gnostr gnostr-git gnostr-get-relays gnostr-docs## 	make gnostr gnostr-cat gnostr-git gnostr-relay gnostr-xor docs
##	build gnostr tool and related dependencies

##gnostr-docs:
##	docker-start doc/gnostr.1
gnostr-docs:docker-start doc/gnostr.1## 	docs: convert README to doc/gnostr.1
#@echo docs
	@bash -c 'if pgrep MacDown; then pkill MacDown; fi; 2>/dev/null'
	@bash -c 'cat $(PWD)/sources/HEADER.md                >  $(PWD)/README.md 2>/dev/null'
	@bash -c 'cat $(PWD)/sources/COMMANDS.md              >> $(PWD)/README.md 2>/dev/null'
	@bash -c 'cat $(PWD)/sources/FOOTER.md                >> $(PWD)/README.md 2>/dev/null'
	@type -P pandoc && pandoc -s README.md -o index.html 2>/dev/null || \
		type -P docker && docker pull pandoc/latex:2.6 && \
		docker run --rm --volume "`pwd`:/data" --user `id -u`:`id -g` pandoc/latex:2.6 README.md
	git add --ignore-errors sources/*.md 2>/dev/null || echo && git add --ignore-errors *.md 2>/dev/null || echo
#@git ls-files -co --exclude-standard | grep '\.md/$\' | xargs git

doc/gnostr.1: README##
	scdoc < $^ > $@

.PHONY: version
version: gnostr.c## 	print version
	@grep '^#define VERSION' $< | sed -En 's,.*"([^"]+)".*,\1,p' > $@
#	@cat $@
.PHONY:GIT-VERSION-FILE git-version
git-version:GIT-VERSION-FILE
GIT-VERSION-FILE:deps/gnostr-git/GIT-VERSION-FILE
	@. deps/gnostr-git/GIT-VERSION-GEN
	@grep '^GIT_VERSION' <GIT-VERSION-FILE | sed -En 's,..............([^"]+).*,\1,p' > git-version
	@cat git-version #&& rm GIT-VERSION-FILE
.PHONY:versions
versions:version git-version

chmod:## 	chmod
##chmod
## 	find . -type f ! -name 'deps/**' -print0     | xargs -0 chmod 644
## 	find . -type f ! -name 'deps/**' --name *.sh | xargs -0 chmod +rwx
## 	find . -type d ! -name 'deps/**' -print0     | xargs -0 chmod 755
## 	if isssues or before 'make dist'
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
submodules:deps/secp256k1/.git deps/gnostr-git/.git deps/gnostr-cat/.git deps/hyper-sdk/.git deps/hyper-nostr/.git deps/gnostr-aio/.git deps/gnostr-py/.git deps/gnostr-act/.git deps/gnostr-legit/.git deps/gnostr-proxy/.git #ext/boost_1_82_0/.git ## 	refresh-submodules
	git submodule update --init --recursive

#.PHONY:deps/secp256k1/config.log
.ONESHELL:
deps/secp256k1/.git:
	devtools/refresh-submodules.sh deps/secp256k1
deps/secp256k1/include/secp256k1.h: deps/secp256k1/.git
#.PHONY:deps/secp256k1/configure
## force configure if build on host then in docker vm
deps/secp256k1/configure: deps/secp256k1/include/secp256k1.h
	cd deps/secp256k1 && \
		./autogen.sh && \
		./configure --enable-module-ecdh --enable-module-schnorrsig --enable-module-extrakeys --disable-benchmark --disable-tests && make -j
.PHONY:deps/secp256k1/.libs/libsecp256k1.a
deps/secp256k1/.libs/libsecp256k1.a:deps/secp256k1/configure
libsecp256k1.a:deps/secp256k1/.libs/libsecp256k1.a## libsecp256k1.a
	cp $< $@
##libsecp256k1.a
##	deps/secp256k1/.git
##	deps/secp256k1/include/secp256k1.h
##	deps/secp256k1/./autogen.sh
##	deps/secp256k1/./configure


deps/jq/modules/oniguruma.git:
	devtools/refresh-submodules.sh deps/jq
deps/jq/.git:deps/jq/modules/oniguruma.git
#.PHONY:deps/jq/.libs/libjq.a
deps/jq/.libs/libjq.a:deps/jq/.git
	cd deps/jq && \
		autoreconf -fi && ./configure --disable-maintainer-mode && make install && cd ../..
##libjq.a
##	cp $< deps/jq/libjq.a .
libjq.a: deps/jq/.libs/libjq.a## 	libjq.a
	cp $< $@





deps/gnostr-web/.git:
	@devtools/refresh-submodules.sh deps/gnostr-web




deps/gnostr-git/.git:
	@devtools/refresh-submodules.sh deps/gnostr-git
.PHONY:deps/gnostr-git/gnostr-git
deps/gnostr-git/gnostr-git:deps/gnostr-git/.git
	install -v template/gnostr-* /usr/local/bin >/tmp/gnostr-git.log
	cd deps/gnostr-git && make && make install
.PHONY:gnostr-git
gnostr-git:deps/gnostr-git/gnostr-git## 	gnostr-git
	cp $< $@
	install $@ /usr/local/bin/



gnostr-get-relays:
	$(CC) ./template/gnostr-get-relays.c -o gnostr-get-relays

gnostr-set-relays:
	$(CC) ./template/gnostr-set-relays.c -o gnostr-set-relays



.PHONY:gnostr-build
gnostr-build:## 	gnostr-build
	rm build/CMakeCache.txt || echo
	cmake -S . -B build && cd build && cmake ../ && make



.PHONY:gnostr-build-install
gnostr-build-install:gnostr-build## 	gnostr-build-install
	cd build && make install
	$(MAKE) gnostr-install || echo

deps/gnostr-command/.git:gnostr-git
	@devtools/refresh-submodules.sh deps/gnostr-command
deps/gnostr-command/gnostr-command:deps/gnostr-command/.git
	cd deps/gnostr-command && \
		make install
deps/gnostr-command/target/release/gnostr-command:deps/gnostr-command/gnostr-command## 	gnostr-command
gnostr-command:deps/gnostr-command/target/release/gnostr-command## 	gnostr-command
	cp $< $@ && exit;


deps/gnostr-legit/.git:gnostr-git
	@devtools/refresh-submodules.sh deps/gnostr-legit
#.PHONY:deps/gnostr-legit/gnostr-legit
deps/gnostr-legit/gnostr-legit:deps/gnostr-legit/.git
	cd deps/gnostr-legit && \
		make legit-install
deps/gnostr-legit/target/release/gnostr-legit:deps/gnostr-legit/gnostr-legit## 	gnostr-legit
gnostr-legit:deps/gnostr-legit/target/release/gnostr-legit## 	gnostr-legit
	cp $< $@ && exit;
	install -v template/gnostr-* /usr/local/bin >/tmp/gnostr-legit.log



deps/gnostr-sha256/.git:
	@devtools/refresh-submodules.sh deps/gnostr-sha256
#.PHONY:deps/gnostr-sha256/gnostr-sha256
deps/gnostr-sha256/gnostr-sha256:deps/gnostr-sha256/.git
	cd deps/gnostr-sha256 && \
		make cargo-install
deps/gnostr-sha256/target/release/gnostr-sha256:deps/gnostr-sha256/gnostr-sha256## 	gnostr-sha256
.PHONY:
gnostr-sha256:deps/gnostr-sha256/target/release/gnostr-sha256

deps/gnostr-proxy/.git:
	@devtools/refresh-submodules.sh deps/gnostr-proxy
.PHONY:deps/gnostr-proxy
deps/gnostr-proxy:deps/gnostr-proxy/.git
	cd deps/gnostr-proxy && \
		$(MAKE) install
gnostr-proxy:deps/gnostr-proxy
	install deps/gnostr-proxy/gnostr-proxy /usr/local/bin

#deps/gnostr-relay/.git:
#	@devtools/refresh-submodules.sh deps/gnostr-relay
##.PHONY:deps/gnostr-relay/gnostr-relay
#deps/gnostr-relay:deps/gnostr-relay/.git
#	cd deps/gnostr-relay && \
#		make
##gnostr-relay:deps/gnostr-relay/target/release/gnostr-relay## 	gnostr-relay
#.PHONY:deps/gnostr-relay
##PHONY for now...
#gnostr-relay:deps/gnostr-relay## 	gnostr-relay
#	cp $< $@


deps/gnostr-cat/.git:
	@devtools/refresh-submodules.sh deps/gnostr-cat
#.PHONY:deps/gnostr-cat
deps/gnostr-cat:deps/gnostr-cat/.git
.PHONY:deps/gnostr-cat/target/release/gnostr-cat
deps/gnostr-cat/target/release/gnostr-cat:deps/gnostr-cat
	cd deps/gnostr-cat && \
		make cargo-build-release install
	@cp $@ gnostr-cat || echo "" 2>/dev/null
.PHONY:gnostr-cat
gnostr-cat:deps/gnostr-cat/target/release/gnostr-cat



.PHONY:deps/gnostr-cli/.git
deps/gnostr-cli/.git:
	@devtools/refresh-submodules.sh deps/gnostr-cli
.PHONY:deps/gnostr-cli/target/release/gnostr-cli
deps/gnostr-cli/target/release/gnostr-cli:deps/gnostr-cli
	cd deps/gnostr-cli && \
		make cargo-build-release cargo-install
	@cp $@ gnostr-cli || echo "" 2>/dev/null
.PHONY:gnostr-cli
##gnostr-cli
##deps/gnostr-cli deps/gnostr-cli/.git
##	cd deps/gnostr-cli; \
##	make cargo-build-release cargo-install
gnostr-cli:deps/gnostr-cli/target/release/gnostr-cli## 	gnostr-cli



deps/gnostr-grep/.git:
	@devtools/refresh-submodules.sh deps/gnostr-grep
.PHONY:deps/gnostr-grep
deps/gnostr-grep:deps/gnostr-grep/.git
.PHONY:deps/gnostr-grep/target/release/gnostr-grep
deps/gnostr-grep/target/release/gnostr-grep:deps/gnostr-grep
	cd deps/gnostr-grep && \
		make install
	@cp $@ gnostr-grep || echo "" 2>/dev/null
.PHONY:gnostr-grep
##gnostr-grep
##deps/gnostr-grep deps/gnostr-grep/.git
##	cd deps/gnostr-grep; \
##	make cargo-install
gnostr-grep:deps/gnostr-grep/target/release/gnostr-grep## 	gnostr-grep



deps/hyper-sdk/.git:
	@devtools/refresh-submodules.sh deps/hyper-sdk
deps/hyper-nostr/.git:
	@devtools/refresh-submodules.sh deps/hyper-nostr
deps/openssl/.git:
	@devtools/refresh-submodules.sh deps/openssl
deps/gnostr-py/.git:
	@devtools/refresh-submodules.sh deps/gnostr-py
deps/gnostr-aio/.git:
	@devtools/refresh-submodules.sh deps/gnostr-aio



deps/gnostr-act/.git:
	@devtools/refresh-submodules.sh deps/gnostr-act
gnostr-act:deps/gnostr-act/.git
	cd deps/gnostr-act && ./install-gnostr-act



%.o: %.c $(HEADERS)
	@echo "cc $<"
	@$(CC) $(CFLAGS) -c $< -o $@

.PHONY:gnostr
gnostr:deps/secp256k1/.libs/libsecp256k1.a libsecp256k1.a $(HEADERS) $(GNOSTR_OBJS) $(ARS)## 	make gnostr binary
##gnostr initialize
##	git submodule update --init --recursive
##	$(CC) $(CFLAGS) $(GNOSTR_OBJS) $(ARS) -o $@
#	git submodule update --init --recursive
	$(CC) $(CFLAGS) $(GNOSTR_OBJS) $(ARS) -o $@
	install ./gnostr /usr/local/bin/

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

.ONESHELL:
##install all
##	install docs/gnostr.1 gnostr gnostr-query
gnostr-install:
	mkdir -p $(PREFIX)/bin
	mkdir -p $(PREFIX)/include
	@install -m755 -v include/*.*                    $(PREFIX)/include 2>/dev/null
	@install -m755 -v gnostr                         $(PREFIX)/bin     2>/dev/null || echo "Try:\nmake gnostr"
	@install -m755 -v template/gnostr-*              $(PREFIX)/bin     2>/dev/null
	@install -m755 -v template/gnostr-query          $(PREFIX)/bin     2>/dev/null
	@install -m755 -v template/gnostr-get-relays     $(PREFIX)/bin     2>/dev/null
	@install -m755 -v template/gnostr-set-relays     $(PREFIX)/bin     2>/dev/null
	@install -m755 -v template/gnostr-*-*            $(PREFIX)/bin     2>/dev/null

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
	./template/gnostr-query -t gnostr | $(shell which gnostr-cat) -u wss://relay.damus.io
	./template/gnostr-query -t weeble | $(shell which gnostr-cat) -u wss://relay.damus.io
	./template/gnostr-query -t wobble | $(shell which gnostr-cat) -u wss://relay.damus.io
	./template/gnostr-query -t blockheight | gnostr-cat -u wss://relay.damus.io

## 	for CI purposes we build largest apps last
## 	rust based apps first after gnostr
## 	nodejs apps second
##
## 	gnostr-legit relies on gnostr-git and gnostr-install sequence
##
gnostr-all:gnostr gnostr-install gnostr-cat gnostr-cli gnostr-grep gnostr-sha256 gnostr-command gnostr-proxy gnostr-query gnostr-git gnostr-legit gnostr-act
	$(MAKE) gnostr-build-install

## git log $(git describe --tags --abbrev=0)...@^1

dist: gnostr-docs version## 	create tar distribution
	source .venv/bin/activate; pip install -r requirements.txt;
	test -d .venv || $(shell which python3) -m virtualenv .venv;
	( \
	   $(shell which python3) -m pip install -r requirements.txt;\
	   $(shell which python3) -m pip install git-archive-all;\
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

.PHONY: fake
