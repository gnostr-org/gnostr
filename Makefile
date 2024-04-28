CFLAGS                                  = -Wall -O2 -Ideps/secp256k1/include
CFLAGS                                 += -I/include
LDFLAGS                                 = -Wl -V
OBJS                                    = nostril.o \
										  sha256.o \
										  aes.o \
										  base64.o \
										  libsecp256k1.a

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
LIB_ARS                                := libsecp256k1.a

%.o: src/%.c $(HEADERS)
	@echo "cc $<"
	@$(CC) $(CFLAGS) -c $< -o $@

all: nostril

nostril: $(OBJS) $(HEADERS)
	$(CC) $(OBJS) -lsecp256k1 -o $@

deps/secp256k1/include/secp256k1.h:
deps/secp256k1/configure:
deps/secp256k1/.libs/libsecp256k1.a:deps/secp256k1/configure
	cd deps/secp256k1 && \
		./autogen.sh && \
		./configure --enable-module-ecdh --enable-module-schnorrsig --enable-module-extrakeys --disable-benchmark --disable-tests && make -j
deps/secp256k1/.libs/libsecp256k1.a:deps/secp256k1/configure
libsecp256k1.a:deps/secp256k1/.libs/libsecp256k1.a## libsecp256k1.a
	cp $< $@
.PHONY:secp256k1
secp256k1:libsecp256k1.a




config.h: configurator
	./configurator > $@

configurator: configurator.c
	$(CC) $< -o $@

clean:
	rm -f nostril *.o *.a
	rm -rf CMakeFiles CMakeCache.txt || true

tags: fake
	ctags *.c *.h

.PHONY: fake
