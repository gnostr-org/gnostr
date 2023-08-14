##clean
##	remove gnostr *.o *.a gnostr.1
cleann:## 	remove gnostr *.o *.a gnostr.1
	rm -rf $(shell which gnostr) || echo
	rm -rf /usr/local/share/man/man1/gnostr.1 || echo
	rm -f gnostr *.o *.a || echo

##clean-hyper-nostr
##	remove deps/hyper-nostr
clean-hyper-nostr:## 	remove deps/hyper-nostr
	rm -rf deps/hyper-nostr || echo

##clean-hyper-sdk
##	remove deps/hypersdk
clean-hyper-sdk:## 	remove deps/hyper-sdk
	rm -rf deps/hyper-sdk || echo

##clean-secp
##	remove deps/secp256k1/.libs/libsecp256k1.*
clean-secp:## 	remove deps/secp256k1/.libs/libsecp256k1.* libsecp256k1.a
	rm -rf libsecp256k1.a || echo
	cd deps/secp256k1 && find . -type f -name '*.o' -print0 | rm -f || echo

##clean-gnostr-git
##	remove deps/gnostr-git/gnostr-git
##	remove gnostr-git
clean-gnostr-git:## 	remove deps/gnostr-git gnostr-git
	rm -rf libgit.a || echo
	cd deps/gnostr-git && find . -type f -name '*.o' -print0 | rm -f || echo
##clean-gnostr-legit
##	remove deps/gnostr-git/gnostr-legit
##	remove gnostr-legit
clean-gnostr-legit:## 	remove deps/gnostr-legit gnostr-legit
	rm -rf gnostr-legit || echo
	cd deps/gnostr-legit && find . -type f -name '*.o' -print0 | rm -f || echo

##clean-gnostr-cat
##	remove deps/gnostr-cat
clean-gnostr-cat:## 	remove deps/gnostr-cat
	cd deps/gnostr-cat && find . -type f -name '*.o' -print0 | rm -f || echo

##clean-gnostr-relay
##	remove deps/gnostr-relay
clean-gnostr-relay:## 	remove deps/gnostr-relay
	cd deps/gnostr-relay && find . -type f -name '*.o' -print0 | rm -f || echo

##clean-tcl
##	remove deps/tcl
clean-tcl:## 	remove deps/tcl
	cd deps/tcl&& find . -type f -name '*.o' -print0 | rm -f || echo

##clean-jq
##	remove deps/jq
clean-jq:## 	remove deps/jq
	cd deps/jq && find . -type f -name '*.o' -print0 | rm -f || echo

##clean-openssl
##	remove deps/openssl
clean-openssl:## 	remove deps/openssl
	cd deps/openssl && rm -rf gost-engine && rm -rf fuzz && find . -type f -name '*.o' -print0 | rm -f || echo

clean-all:clean clean-hyper-nostr clean-secp clean-gnostr-git clean-openssl## 	clean clean-*
##clean-all
##	clean clean-hyper-nostr clean-secp clean-gnostr-git clean-tcl clean-jq
	find . -type f -name '*.o' -print0 | rm -f
