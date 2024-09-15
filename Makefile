
CFLAGS = -Wall -O2 -Iext/secp256k1/include
OBJS = sha256.o nostril.o aes.o base64.o
HEADERS = hex.h random.h config.h sha256.h ext/secp256k1/include/secp256k1.h
PREFIX ?= /usr/local
ARS = libsecp256k1.a

SUBMODULES = ext/secp256k1

default: all
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?##/ {printf "\033[36m%-15s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)
help:## 	print verbose help
	@echo ''
	@echo 'Usage: make [TARGET] [EXTRA_ARGUMENTS]'
	@echo ''
	@sed -n 's/^##//p' ${MAKEFILE_LIST} | column -t -s ':   ' |  sed -e 's/^/ /' ## verbose help ideas
	@sed -n 's/^##  //p' ${MAKEFILE_LIST} | column -t -s ':' |  sed -e 's/^/ /'
	@echo ""
	@echo "Useful Commands:"
	@echo ""

all: libsecp256k1.a nostril docs## 	nostril docs

docs: doc/nostril.1## 	docs
doc/nostril.1: README.md## 	doc/nostril.1
	@scdoc < $^ > $@ || help2man ./nostril > doc/nostril.1

version: nostril.c## 	version
	@git fetch --all --tags -f
	@grep '^#define VERSION' $< | sed -En 's,.*"([^"]+)".*,\1,p' > $@

dist: docs version## 	dist
	@mkdir -p dist
	@touch dist/SHA256SUMS.txt
	@touch dist/dist.json
	git add Makefile dist/*.txt* dist/dist.json
	git ls-files --recurse-submodules | $(shell which gtar || which tar) --transform 's/^/nostril-$(shell cat version)\//' -T- -caf dist/nostril-$(shell cat version).tar.gz
	@ls -dt dist/* | head -n1 | xargs echo "tgz "
	git add Makefile dist/SHA256SUMS.txt* dist/dist.json
	cd dist;\
	sha256sum *.tar.gz > SHA256SUMS.txt;\
	git add *.txt* dist.json
	gpg -u $(shell gpg --list-signatures --with-colons | grep 'sig' | grep 'E616FA7221A1613E5B99206297966C06BB06757B' | head -n 1 | cut -d':' -f5) --sign --armor --detach-sig --output SHA256SUMS.txt.asc SHA256SUMS.txt | true
	cp CHANGELOG dist/CHANGELOG.txt
	git add dist/*.txt* dist/dist.json
	./nostril --sec $(shell ./nostril --hash $(shell date +%s)) -t $(shell git rev-parse --short HEAD) -t gnostr --tag weeble $(shell gnostr-weeble) --tag blockheight $(shell gnostr-blockheight) --tag wobble $(shell gnostr-wobble) > dist/dist.json
	git add dist/*.txt* dist/dist.json
	git commit -m $(shell gnostr-weeble)/$(shell gnostr-blockheight)/$(shell gnostr-wobble)
	cat dist/dist.json | gnostr-post-event --relay wss://relay.damus.io
	#rsync -avzP dist/ charon:/www/cdn.jb55.com/tarballs/nostril/


%.o: %.c $(HEADERS)
	@echo "cc $<"
	@$(CC) $(CFLAGS) -c $< -o $@

.PHONY:nostril
nostril: $(HEADERS) $(OBJS)## 	nostril
	@$(CC) $(CFLAGS) $(OBJS) $(ARS) -o $@ || $(MAKE) $(ARS)
	cp nostril gnostr

install: all## 	install
	@mkdir -p $(PREFIX)/bin || true
	@install -m644 doc/nostril.1 $(PREFIX)/share/man/man1/nostril.1 || true
	@install -m755 ./nostril $(PREFIX)/bin/nostril || true
	@install -m755 nostril-query $(PREFIX)/bin/nostril-query || true
	@install -m755 ./nostril $(PREFIX)/bin/gnostr || true
	#@$(shell which nostril)
	#@$(shell which gnostr)

config.h: configurator## 	config.h
	./configurator > $@

configurator: configurator.c## 	configurator
	$(CC) $< -o $@

clean:## 	clean
	rm -f nostril *.o *.a
	rm -rf ext/secp256k1/.lib
	rm -f configurator
	rm -rf configurator.out.dSYM

tags: fake
	ctags *.c *.h

test: nostril
	./nostril --hash ''
	./nostril --hash ""
	./nostril --hash ' '
	./nostril --hash " "
	type -P gnostr-sha256 >/dev/null && gnostr-sha256 ''
	type -P gnostr-sha256 "" && gnostr-sha256 ""
	type -P gnostr-sha256 && gnostr-sha256 ' '
	type -P gnostr-sha256 " " && gnostr-sha256 " "
.PHONY:docs doc/nostril.1 fake nostril version
