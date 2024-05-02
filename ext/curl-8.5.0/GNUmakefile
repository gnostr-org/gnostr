.PHONY:configure
default:
	git update-index --assume-unchanged configure
	[ -x $(shell which cmake) ] && cmake . && $(MAKE) all install
	cp src/curl src/gnostr-curl
	install src/gnostr-curl /usr/local/bin
##configure:configure-with-openssl
##configure-with-openssl:
##	 ./configure --with-openssl
##configure-with-openssl-rustls:
##	 echo TODO#./configure --with-openssl --with-rustls

-include Makefile
