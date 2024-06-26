##clean
##	remove gnostr *.o *.a gnostr.1
clean-gnostr:## 	remove gnostr *.o *.a gnostr.1
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
	cd ext/secp256k1-0.3.2 && find . -type f -name '*.o' -print0 | rm -f || echo

##clean-gnostr-git
##	remove deps/gnostr-git/gnostr-git
##	remove gnostr-git
clean-gnostr-git:## 	remove deps/gnostr-git gnostr-git
	rm -rf libgit.a || echo
	cd git && find . -type f -name '*.o' -print0 | rm -f || echo

##clean-gnostr-legit
##	remove deps/gnostr-git/gnostr-legit
##	remove gnostr-legit
clean-gnostr-legit:## 	remove deps/gnostr-legit gnostr-legit
	rm -rf gnostr-legit || echo
	cd legit && find . -type f -name '*.o' -print0 | rm -f || echo

##clean-gnostr-cat
##	remove deps/gnostr-cat
clean-gnostr-cat:## 	remove deps/gnostr-cat
	cd cat && find . -type f -name '*.o' -print0 | rm -f || echo

clean-all:clean clean-hyper-nostr clean-secp clean-gnostr-git clean-gnostr-legit clean-gnostr-cat## 	clean clean-*
##clean-all
##	clean clean-hyper-nostr clean-secp clean-gnostr-git clean-tcl clean-jq
	find . -type f -name '*.o' -print0 | rm -f
	find . -type f -name '*.a' -print0 | rm -f
