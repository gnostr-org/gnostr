LOCAL_KEY=$(shell cat local_key | xxd -ps -cols 256)
LOCAL_KEY_SHA256=$(shell cat local_key | xxd -ps -cols 256 | sha256sum | sed 's/-//g')
default:
#echo $(LOCAL_KEY)
	@echo LOCAL_KEY_SHA256=$(LOCAL_KEY_SHA256)
	@git update-index --assume-unchanged deps/secp256k1
	@git update-index --assume-unchanged Makefile
	@$(MAKE) libsecp256k1.a >/dev/null || $(MAKE) secp256k1 >/dev/null
-include nostril.mk
-include Makefile
