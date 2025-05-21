-:
	@cargo -q b 2>/dev/null;
	@RUST_LOG=off ./target/debug/gnostr-watch | jq || \
	RUST_LOG=off ./target/debug/gnostr-watch | gnostr-xq || \
		cargo install gnostr-xq || true
#@for relay in $(shell ./target/debug/git-log); do echo $$relay; done
install:
	cargo install --path .
