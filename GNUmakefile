default:
	git update-index --assume-unchanged deps/secp256k1
	git update-index --assume-unchanged Makefile
	$(MAKE) secp256k1 || $(MAKE) libsecp256k1.a
-include nostril.mk
-include Makefile
