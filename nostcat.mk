nostcat:## 
	@type -P rustc && make -C deps/nostcat cargo-install || echo -e "try:\nmake install-rustup"

