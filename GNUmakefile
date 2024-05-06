-include Makefile
#libsecp256k1.a:
#	cd ext/secp256k1 && ./autogen.sh && ./configure
#Makefile:ext/secp256k1/src/libsecp256k1.a
#	git update-index --assume-unchanged Makefile
#	cmake .
#	git update-index --assume-unchanged Makefile
ext/secp256k1/.git:
	#git checkout ext/secp256k1
	@devtools/refresh-submodules.sh $(SUBMODULES)

ext/secp256k1/include/secp256k1.h: ext/secp256k1/.git

ext/secp256k1/configure: ext/secp256k1/include/secp256k1.h
	git checkout ext/secp256k1
	cd ext/secp256k1; \
	./autogen.sh #&& \
	#automake --add-missing; \
	#autoreconf; \

ext/secp256k1/Makefile: ext/secp256k1/configure
	cd ext/secp256k1; \
	./configure --disable-shared --enable-module-ecdh --enable-module-schnorrsig --enable-module-extrakeys

ext/secp256k1/.libs/libsecp256k1.a: ext/secp256k1/Makefile
	cd ext/secp256k1; \
	make -j libsecp256k1.la

libsecp256k1.a: ext/secp256k1/.libs/libsecp256k1.a
	cp $< $@


