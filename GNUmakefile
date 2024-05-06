-include Makefile
#ext/secp256k1/src/libsecp256k1.a:
#	cd ext/secp256k1 && ./autogen.sh && ./configure && make
Makefile:ext/secp256k1/src/libsecp256k1.a
	git update-index --assume-unchanged Makefile
	cmake .
	git update-index --assume-unchanged Makefile
