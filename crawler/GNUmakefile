-:
	@cargo -q b 2>/dev/null;
	@RUST_LOG=off ./target/debug/git-log | gnostr-xq || \
		cargo install gnostr-xq
#@for relay in $(shell ./target/debug/git-log); do echo $$relay; done
install:
	cargo install --path .
