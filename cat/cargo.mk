## CARGO
cargo-help:### 	cargo-help
	@awk 'BEGIN {FS = ":.*?###"} /^[a-zA-Z_-]+:.*?###/ {printf "\033[36m%-15s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

cargo-install:### 	cargo install --path .
## cargo install --locked --path `pwd`
	@$(CARGO) install --locked --path $(PWD)

cargo-bench:### 	cargo-bench
## cargo bench
	@$(CARGO) bench

cargo-examples:### 	cargo-examples
## cargo b --examples
	@$(CARGO) b --examples

cargo-report:### 	cargo-report
## cargo report future-incompatibilities --id 1
	$(CARGO) report future-incompatibilities --id 1

cargo-doc:### 	cargo-doc
## cargo doc
## cargo doc --no-deps
## cargo doc --no-deps --open
	 $(CARGO) doc --no-deps #--open

cargo-nightly-udeps:### 	cargo-nightly-udeps
## cargo +nightly udeps
	 $(CARGO) +nightly udeps

## :
# vim: set noexpandtab:
# vim: set setfiletype make
