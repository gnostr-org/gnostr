CFLAGS = -Wall -O2 -Ideps/secp256k1/include
LDFLAGS = -Wl -V
OBJS = sha256.o nostril.o aes.o base64.o
HEADERS = hex.h random.h config.h sha256.h deps/secp256k1/include/secp256k1.h
PREFIX ?= /usr/local
ARS = libsecp256k1.a libgit.a libjq.a libtclstub.a

SUBMODULES = deps/secp256k1 deps/git deps/jq deps/nostcat deps/hyper-nostr deps/tcl

VERSION:=$(shell cat version)
export VERSION
GTAR:=$(shell which gtar)
export GTAR
TAR:=$(shell which tar)
export TAR

##all:
##	nostril docs
all: nostril docs## 	make nostril docs

##docs:
##	doc/nostril.1 git-add docker-start
docs: doc/nostril.1 git-add docker-start## 	docs: convert README to doc/nostril.1
#@echo docs
	@bash -c 'if pgrep MacDown; then pkill MacDown; fi; 2>/dev/null'
	@bash -c 'cat $(PWD)/sources/HEADER.md                >  $(PWD)/README.md 2>/dev/null'
	@bash -c 'cat $(PWD)/sources/COMMANDS.md              >> $(PWD)/README.md 2>/dev/null'
	@bash -c 'cat $(PWD)/sources/FOOTER.md                >> $(PWD)/README.md 2>/dev/null'
	if hash pandoc 2>/dev/null; then \
		bash -c 'pandoc -s README.md -o index.html' 2>/dev/null; \
		fi || if hash docker 2>/dev/null; then \
		docker run --rm --volume "`pwd`:/data" --user `id -u`:`id -g` pandoc/latex:2.6 README.md; \
		fi
	@git add --ignore-errors sources/*.md 2>/dev/null
	@git add --ignore-errors *.md 2>/dev/null
#@git ls-files -co --exclude-standard | grep '\.md/$\' | xargs git

doc/nostril.1: README## 	
	scdoc < $^ > $@

.PHONY: version
version: nostril.c## 	print version
	@grep '^#define VERSION' $< | sed -En 's,.*"([^"]+)".*,\1,p' > $@
	@cat $@

dist: docs version## 	create tar distribution
	touch deps/tcl/unix/dltest/pkgπ.c
	touch deps/tcl/unix/dltest/pkg\317\200.c
	cp deps/tcl/unix/dltest/pkgπ.c deps/tcl/unix/dltest/pkg\317\200.c
	mkdir -p dist
	cat version > CHANGELOG && git add -f CHANGELOG && git commit -m "CHANGELOG: update" 2>/dev/null || echo
	git log $(shell git describe --tags --abbrev=0)..@^1 --oneline | sed '/Merge/d' >> CHANGELOG
	cp CHANGELOG dist/CHANGELOG.txt
	git ls-files --recurse-submodules | $(GTAR) --exclude='"deps/tcl/unix/dltest/*.c"'  --transform  's/^/nostril-$(VERSION)\//' -T- -caf dist/nostril-$(VERSION).tar.gz
	ls -dt dist/* | head -n1 | xargs echo "tgz "
	cd dist;\
	sha256sum *.tar.gz > SHA256SUMS.txt;\
	gpg -u 0xE616FA7221A1613E5B99206297966C06BB06757B --sign --armor --detach-sig --output SHA256SUMS.txt.asc SHA256SUMS.txt
	##rsync -avzP dist/ charon:/www/cdn.jb55.com/tarballs/nostril/

submodules:deps/secp256k1/.git deps/jq/.git deps/git/.git deps/nostcat/.git deps/tcl/.git## 	refresh-submodules

deps/secp256k1/.git:
deps/secp256k1/include/secp256k1.h: deps/secp256k1/.git
deps/secp256k1/configure: deps/secp256k1/include/secp256k1.h
	cd deps/secp256k1; \
	./autogen.sh
deps/secp256k1/config.log: deps/secp256k1/configure
.PHONY:deps/secp256k1/config.log
	cd deps/secp256k1; \
	./configure --disable-shared --enable-module-ecdh --enable-module-schnorrsig --enable-module-extrakeys
deps/secp256k1/.libs/libsecp256k1.a: deps/secp256k1/config.log
	cd deps/secp256k1; \
	make -j libsecp256k1.la
libsecp256k1.a: deps/secp256k1/.libs/libsecp256k1.a## libsecp256k1.a
	cp $< $@
##libsecp256k1.a
##	deps/secp256k1/.git
##	deps/secp256k1/include/secp256k1.h
##	deps/secp256k1/./autogen.sh
##	deps/secp256k1/./configure

deps/jq/.git:
	@devtools/refresh-submodules.sh $(SUBMODULES)
.PHONY:deps/jq/.libs/libjq.a
deps/jq/.libs/libjq.a:deps/jq/.git
	cd deps/jq; \
	autoreconf -fi && ./configure  --disable-maintainer-mode &&  make install
##libjq.a
##	cp $< deps/jq/libjq.a .
libjq.a: deps/jq/.libs/libjq.a## 	libjq.a
	cp $< $@

deps/git/.git:
	@devtools/refresh-submodules.sh $(SUBMODULES)
deps/git/libgit.a:deps/git/.git
	cd deps/git; \
	make install
##libgit.a
##	deps/git/libgit.a deps/git/.git
##	cd deps/git; \
##	make install
libgit.a: deps/git/libgit.a## 	libgit.a
	cp $< $@

deps/tcl/.git:
	@devtools/refresh-submodules.sh $(SUBMODULES)
deps/tcl/unix/libtclstub.a:deps/tcl/.git
	cd deps/tcl/unix; \
	./autogen.sh configure && ./configure && make install
libtclstub.a:deps/tcl/unix/libtclstub.a## 	deps/tcl/unix/libtclstub.a
	cp $< $@
##tcl-unix
##	deps/tcl/unix/libtclstub.a deps/tcl/.git
##	cd deps/tcl/unix; \
##	./autogen.sh configure && ./configure && make install
tcl-unix:libtclstub.a## 	deps/tcl/unix/libtclstub.a

deps/nostcat/.git:
	@devtools/refresh-submodules.sh $(SUBMODULES)
deps/nostcat:deps/nostcat/.git
	cd deps/nostcat; \
	make cargo-install
deps/nostcat/target/release/nostcat:deps/nostcat
	cp $@ nostcat
.PHONY:deps/nostcat
##nostcat
##deps/nostcat deps/nostcat/.git
##	cd deps/nostcat; \
##	make cargo-install
nostcat:deps/nostcat/target/release/nostcat## 	nostcat

%.o: %.c $(HEADERS)
	@echo "cc $<"
	@$(CC) $(CFLAGS) -c $< -o $@

##initialize
##	git submodule update --init --recursive
initialize:## 	ensure submodules exist
	git submodule update --init --recursive
nostril:initialize $(HEADERS) $(OBJS) $(ARS)## 	make nostril binary
##nostril initialize
##	git submodule update --init --recursive
##	$(CC) $(CFLAGS) $(OBJS) $(ARS) -o $@
	git submodule update --init --recursive
	$(CC) $(CFLAGS) $(OBJS) $(ARS) -o $@

##install all
##	install docs/nostril.1 nostril nostril-query
install: all## 	install docs/nostril.1 nostril nostril-query
	@mkdir -p $(PREFIX)/bin
	@install -m644 doc/nostril.1 $(PREFIX)/share/man/man1/nostril.1
	@install -m755 nostril $(PREFIX)/bin/nostril
	@install -m755 nostril-query $(PREFIX)/bin/nostril-query

.PHONY:config.h
config.h: configurator
	./configurator > $@

.PHONY:configurator
##configurator
##	rm -f configurator
##	$(CC) $< -o $@
configurator: configurator.c
	rm -f configurator
	$(CC) $< -o $@

##clean
##	remove nostril *.o *.a nostril.1
clean:## 	remove nostril *.o *.a nostril.1
	rm -rf $(shell which nostril)
	rm -rf /usr/local/share/man/man1/nostril.1
	rm -f nostril *.o *.a

##clean-hyper-nostr
##	remove deps/hyper-nostr
clean-hyper-nostr:## 	remove deps/hyper-nostr
	rm -rf deps/hyper-nostr

##clean-secp
##	remove deps/secp256k1
clean-secp:## 	remove deps/secp256k1
	rm -rf deps/secp256k1

##clean-git
##	remove deps/git
clean-git:## 	remove deps/git
	rm -rf deps/git

##clean-tcl
##	remove deps/tcl
clean-tcl:## 	remove deps/tcl
	rm -rf deps/tcl
##clean-jq
##	remove deps/jq
clean-jq:## 	remove deps/jq
	rm -rf deps/jq
clean-all:clean clean-hyper-nostr clean-secp clean-git clean-tcl clean-jq## 	
##clean-all
##	clean clean-hyper-nostr clean-secp clean-git clean-tcl clean-jq

##tags 	ctags *.c *.h
tags: fake
	ctags *.c *.h

.PHONY: fake
