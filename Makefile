
CFLAGS = -Wall -O2 -Ideps/secp256k1/include
OBJS = sha256.o nostril.o aes.o base64.o
HEADERS = hex.h random.h config.h sha256.h deps/secp256k1/include/secp256k1.h
PREFIX ?= /usr/local
ARS = libsecp256k1.a

SUBMODULES = deps/secp256k1

all: nostril docs## make nostril docs

docs: doc/nostril.1 git-add docker-start##   docs: convert README to doc/nostril.1
	#@echo docs
	bash -c 'if pgrep MacDown; then pkill MacDown; fi'
	bash -c 'cat $(PWD)/sources/HEADER.md                >  $(PWD)/README.md'
	bash -c 'cat $(PWD)/sources/COMMANDS.md              >> $(PWD)/README.md'
	bash -c 'cat $(PWD)/sources/FOOTER.md                >> $(PWD)/README.md'
	@if hash pandoc 2>/dev/null; then \
		bash -c 'pandoc -s README.md -o index.html'; \
		fi || if hash docker 2>/dev/null; then \
		docker run --rm --volume "`pwd`:/data" --user `id -u`:`id -g` pandoc/latex:2.6 README.md; \
		fi
	git add --ignore-errors sources/*.md
	git add --ignore-errors *.md
	#git ls-files -co --exclude-standard | grep '\.md/$\' | xargs git

doc/nostril.1: README##
	scdoc < $^ > $@

version: nostril.c## VERSION > $@
	grep '^#define VERSION' $< | sed -En 's,.*"([^"]+)".*,\1,p' > $@

dist: docs version## create tar distribution
	@mkdir -p dist
	git ls-files --recurse-submodules | tar --transform 's/^/nostril-$(shell cat version)\//' -T- -caf dist/nostril-$(shell cat version).tar.gz
	@ls -dt dist/* | head -n1 | xargs echo "tgz "
	cd dist;\
	sha256sum *.tar.gz > SHA256SUMS.txt;\
	gpg -u 0x8A478B64FFE30F1095A8736BF5F27EFD1B38DABB --sign --armor --detach-sig --output SHA256SUMS.txt.asc SHA256SUMS.txt
	cp CHANGELOG dist/CHANGELOG.txt
	rsync -avzP dist/ charon:/www/cdn.jb55.com/tarballs/nostril/

submodules:deps/secp256k1/.git## refresh submodules
deps/secp256k1/.git:
	@devtools/refresh-submodules.sh $(SUBMODULES)

deps/secp256k1/include/secp256k1.h: deps/secp256k1/.git

deps/secp256k1/configure: deps/secp256k1/.git
	cd deps/secp256k1; \
	./autogen.sh

deps/secp256k1/config.log: deps/secp256k1/configure
	cd deps/secp256k1; \
	./configure --disable-shared --enable-module-ecdh --enable-module-schnorrsig --enable-module-extrakeys

deps/secp256k1/.libs/libsecp256k1.a: deps/secp256k1/config.log
	cd deps/secp256k1; \
	make -j libsecp256k1.la

libsecp256k1.a: deps/secp256k1/.libs/libsecp256k1.a
	cp $< $@

%.o: %.c $(HEADERS)
	@echo "cc $<"
	@$(CC) $(CFLAGS) -c $< -o $@

nostril: $(HEADERS) $(OBJS) $(ARS)## make nostril binary
	$(CC) $(CFLAGS) $(OBJS) $(ARS) -o $@

install: all## install docs/nostril.1 nostril nostril-query
	mkdir -p $(PREFIX)/bin
	install -m644 doc/nostril.1 $(PREFIX)/share/man/man1/nostril.1
	install -m755 nostril $(PREFIX)/bin/nostril
	install -m755 nostril-query $(PREFIX)/bin/nostril-query

.PHONY:config.h
config.h: configurator
	./configurator > $@

.PHONY:configurator
configurator: configurator.c## make configurator
	rm -f configurator
	$(CC) $< -o $@

clean:## remove nostril *.o *.a nostril.1 deps/secp256k1
	rm -rf $(shell which nostril)
	rm -rf /usr/local/share/man/man1/nostril.1
	rm -f nostril *.o *.a
	rm -rf deps/secp256k1

tags: fake## ctags *.c *.h
	ctags *.c *.h

.PHONY: fake
