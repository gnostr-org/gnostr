CFLAGS                                  = -Wall -O2 -Ideps/secp256k1/include
CFLAGS                                 += -I/include
LDFLAGS                                 = -Wl -V
GNOSTR_OBJS                             = gnostr.o       sha256.o aes.o base64.o libsecp256k1.a
#GNOSTR_GIT_OBJS                         = gnostr-git.o   sha256.o aes.o base64.o libgit.a
#GNOSTR_RELAY_OBJS                       = gnostr-relay.o sha256.o aes.o base64.o
GNOSTR_XOR_OBJS                         = gnostr-xor.o   sha256.o aes.o base64.o libsecp256k1.a
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

SUBMODULES                              = deps/secp256k1
SUBMODULES_MORE                         = deps/secp256k1 deps/git deps/gnostr-cat deps/hyper-nostr deps/tcl deps/hyper-sdk deps/act deps/openssl deps/gnostr-py deps/gnostr-aio deps/act deps/gnostr-legit deps/gnostr-relay deps/gnostr-proxy deps/gnostr-relay

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
#all: submodules gnostr gnostr-cat gnostr-git gnostr-relay gnostr-xor gnostr-docs## 	make gnostr gnostr-cat gnostr-git gnostr-relay gnostr-xor gnostr-docs
all: submodules gnostr gnostr-git gnostr-relay gnostr-xor gnostr-get-relays gnostr-docs## 	make gnostr gnostr-cat gnostr-git gnostr-relay gnostr-xor docs
##	build gnostr tool and related dependencies

##gnostr-docs:
##	docker-statt doc/gnostr.1
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
	@cat $@
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

dist: docs version## 	create tar distribution
	touch deps/tcl/unix/dltest/pkgπ.c || echo
	touch deps/tcl/unix/dltest/pkg\317\200.c || echo
	cp deps/tcl/unix/dltest/pkgπ.c deps/tcl/unix/dltest/pkg\317\200.c || echo
	mv dist dist-$(VERSION)-$(OS)-$(ARCH)-$(TIME) || echo
	mkdir -p dist && touch dist/.gitkeep
	cat version > CHANGELOG && git add -f CHANGELOG && git commit -m "CHANGELOG: update" 2>/dev/null || echo
	git log $(shell git describe --tags --abbrev=0)..@^1 --oneline | sed '/Merge/d' >> CHANGELOG
	cp CHANGELOG dist/CHANGELOG.txt
	git ls-files --recurse-submodules | $(GTAR) --exclude='"deps/tcl/unix/dltest/*.c"' \
		--transform  's/^/gnostr-$(VERSION)-$(OS)-$(ARCH)\//' -T- -caf dist/gnostr-$(VERSION)-$(OS)-$(ARCH).tar.gz
	ls -dt dist/* | head -n1 | xargs echo "tgz "
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
		$(GTAR) -tvf gnostr-$(VERSION)-$(OS)-$(ARCH).tar.gz > gnostr-$(VERSION)-$(OS)-$(ARCH).tar.gz.txt
	cd dist && \
		$(GTAR) -xf  gnostr-$(VERSION)-$(OS)-$(ARCH).tar.gz && \
		cd  gnostr-$(VERSION)-$(OS)-$(ARCH) && make chmod all install
diff-log:
	@mkdir -p tests && diff template/gnostr-git-reflog template/gnostr-git-log > tests/diff.log || \
		git diff tests/diff.log
	@gnostr -h > tests/gnostr-h.log
	@gnostr-git -h > tests/gnostr-git-h.log
	@gnostr-git-log -h > tests/gnostr-git-log-h.log
	@gnostr-git-reflog -h > tests/gnostr-git-reflog-h.log
	@gnostr-relay -h > tests/gnostr-relay-h.log
.PHONY:submodules
submodules:deps/secp256k1/.git deps/gnostr-git/.git deps/gnostr-cat/.git deps/hyper-sdk/.git deps/hyper-nostr/.git deps/openssl/.git deps/gnostr-aio/.git deps/gnostr-py/.git deps/act/.git deps/gnostr-legit/.git deps/gnostr-proxy/.git## 	refresh-submodules

.PHONY:deps/secp256k1/config.log
.ONESHELL:
deps/secp256k1/.git:
	devtools/refresh-submodules.sh deps/secp256k1
deps/secp256k1/include/secp256k1.h: deps/secp256k1/.git
deps/secp256k1/configure: deps/secp256k1/include/secp256k1.h
	cd deps/secp256k1 && \
		./autogen.sh
deps/secp256k1/config.log: deps/secp256k1/configure
	cd deps/secp256k1 && \
		./configure --enable-module-ecdh --enable-module-schnorrsig --enable-module-extrakeys
deps/secp256k1/.libs/libsecp256k1.a:deps/secp256k1/config.log
	cd deps/secp256k1 && \
		make -j && make install
.PHONY:libsecp256k1.a
libsecp256k1.a: deps/secp256k1/.libs/libsecp256k1.a## libsecp256k1.a
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
	cd deps/gnostr-git && make && make install
gnostr-git:deps/gnostr-git/gnostr-git## 	gnostr-git

.PHONY:
gnostr-get-relays:
	$(CC) ./template/gnostr-get-relays.c -o gnostr-get-relays



gnostr-relay:build## 	gnostr-relay:build
.PHONY:build
gnostr-build:## 		cmake build gnostr-relay
	cmake -S . -B build && cd build && cmake ../ && make




deps/gnostr-legit/.git:
	@devtools/refresh-submodules.sh deps/gnostr-legit
#.PHONY:deps/gnostr-legit/gnostr-legit
deps/gnostr-legit/gnostr-legit:deps/gnostr-legit/.git
	cd deps/gnostr-legit && \
		make legit-install
gnostr-legit:deps/gnostr-legit/target/release/gnostr-legit## 	gnostr-legit
	cp $< $@ && exit;



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
		make run
gnostr-proxy:deps/gnostr-proxy
	cat deps/gnostr-proxy/public/assets/entrypoints.json

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

deps/tcl/.git:
	@devtools/refresh-submodules.sh deps/tcl
deps/tcl/unix/libtclstub.a:deps/tcl/.git
	cd deps/tcl/unix && \
		./autogen.sh configure && ./configure && make install
libtclstub.a:deps/tcl/unix/libtclstub.a## 	deps/tcl/unix/libtclstub.a
	cp $< $@
##tcl-unix
##	deps/tcl/unix/libtclstub.a deps/tcl/.git
##	cd deps/tcl/unix; \
##	./autogen.sh configure && ./configure && make install
tcl-unix:libtclstub.a## 	deps/tcl/unix/libtclstub.a

deps/gnostr-cat/.git:
	@devtools/refresh-submodules.sh deps/gnostr-cat
#.PHONY:deps/gnostr-cat
deps/gnostr-cat:deps/gnostr-cat/.git
	cd deps/gnostr-cat && \
		make cargo-install
.PHONY:deps/gnostr-cat/target/release/gnostr-cat
deps/gnostr-cat/target/release/gnostr-cat:deps/gnostr-cat
	cd deps/gnostr-cat && \
		make cargo-install
	@cp $@ gnostr-cat || echo "" 2>/dev/null
.PHONY:gnostr-cat
##gnostr-cat
##deps/gnostr-cat deps/gnostr-cat/.git
##	cd deps/gnostr-cat; \
##	make cargo-install
gnostr-cat:deps/gnostr-cat/target/release/gnostr-cat## 	gnostr-cat

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
deps/act/.git:
	@devtools/refresh-submodules.sh deps/act

%.o: %.c $(HEADERS)
	@echo "cc $<"
	@$(CC) $(CFLAGS) -c $< -o $@

gnostr:clean $(HEADERS) $(GNOSTR_OBJS) $(ARS)## 	make gnostr binary
##gnostr initialize
##	git submodule update --init --recursive
##	$(CC) $(CFLAGS) $(GNOSTR_OBJS) $(ARS) -o $@
#	git submodule update --init --recursive
	$(CC) $(CFLAGS) $(GNOSTR_OBJS) $(ARS) -o $@

#gnostr-relay:initialize $(HEADERS) $(GNOSTR_RELAY_OBJS) $(ARS)## 	make gnostr-relay
###gnostr-relay
#	git submodule update --init --recursive
#	$(CC) $(CFLAGS) $(GNOSTR_RELAY_OBJS) $(ARS) -o $@

#.PHONY:gnostr-xor
gnostr-xor: $(HEADERS) $(GNOSTR_XOR_OBJS) $(ARS)## 	make gnostr-xor
##gnostr-xor
	echo $@
	touch $@
	rm -f $@
	$(CC) $@.c -o $@

.ONESHELL:
##install all
##	install docs/gnostr.1 gnostr gnostr-query
gnostr-install:
	mkdir -p $(PREFIX)/bin
	mkdir -p $(PREFIX)/include
	shopt -s extglob && install -m755 -vC include/*.*           ${PREFIX}/include 2>/dev/null
	shopt -s extglob && install -m755 -vC gnostr                $(PREFIX)/bin     2>/dev/null
	shopt -s extglob && install -m755 -vC template/gnostr-get-relays     $(PREFIX)/bin     2>/dev/null
	shopt -s extglob && install -m755 -vC template/gnostr-set-relays     $(PREFIX)/bin     2>/dev/null
	shopt -s extglob && install -m755 -vC template/gnostr-*-*            $(PREFIX)/bin     2>/dev/null

.ONESHELL:
##install-doc
##	install-doc
install-doc:## 	install-doc
## 	install -m 0644 -vC doc/gnostr.1 $(PREFIX)/share/man/man1/gnostr.1
	@install -m 0644 -vC doc/gnostr.1 $(PREFIX)/share/man/man1/gnostr.1 || echo "doc/gnostr.1 failed to install..."

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

.PHONY: fake
