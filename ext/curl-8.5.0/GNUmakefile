.PHONY:configure
default:
	git update-index --assume-unchanged configure
	[ -x $(shell which cmake) ] && cmake . && $(MAKE) all install
##configure:configure-with-openssl
##configure-with-openssl:
##	 ./configure --with-openssl
##configure-with-openssl-rustls:
##	 echo TODO#./configure --with-openssl --with-rustls

-include Makefile
