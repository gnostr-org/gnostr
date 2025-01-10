##
##make cargo-*
cargo-help:### 	cargo-help
	@awk 'BEGIN {FS = ":.*?###"} /^[a-zA-Z_-]+:.*?###/ {printf "\033[36m%-15s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

cargo-install:### 	cargo install --path .
#@. $(HOME)/.cargo/env
	#@cargo install --path $(PWD)
	@cargo install --locked --path $(PWD) --force

cargo-check:### 	cargo-check
## cargo c
	@. $(HOME)/.cargo/env
	@cargo c

cargo-bench:### 	cargo-bench
## cargo bench
	@. $(HOME)/.cargo/env
	@cargo bench

cargo-test-benches:### 	cargo-test-benches
	cargo test --benches

cargo-report:### 	cargo-report
	@. $(HOME)/.cargo/env && cargo report future-incompatibilities --id 1

cargo-doc:### 	cargo-doc
	@. $(HOME)/.cargo/env && cargo doc --no-deps #--open

# vim: set noexpandtab:
# vim: set setfiletype make
