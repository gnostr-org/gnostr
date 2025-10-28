cargo-help:### 	
	@awk 'BEGIN {FS = ":.*?###"} /^[a-zA-Z_-]+:.*?###/ {printf "\033[36m%-15s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)
cargo-build:###	cargo-build
	@. $(HOME)/.cargo/env
	@cargo b
cargo-install:###	cargo-install
	@. $(HOME)/.cargo/env
	@cargo install --path $(PWD)
	@echo "export PATH=$(CARGO_PATH)/bin:$(PATH)"
cargo-build-release:###	cargo-build-release
	@. $(HOME)/.cargo/env
	@cargo b --release
cargo-check:###	cargo-check
	@. $(HOME)/.cargo/env
	@cargo c
cargo-bench:###	cargo-bench
	@. $(HOME)/.cargo/env
	@cargo bench
cargo-test:###	cargo-test
	@. $(HOME)/.cargo/env
	@cargo test
# vim: set noexpandtab:
# vim: set setfiletype make
