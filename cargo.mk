cargo-build:## 	cargo-build
	@echo "export PATH=$(CARGO_PATH)/bin:$(PATH)"
	@. $(HOME)/.cargo/env
	@cargo b
cargo-install:## 	cargo-install
	@echo "export PATH=$(CARGO_PATH)/bin:$(PATH)"
	@. $(HOME)/.cargo/env
	@cargo install --path $(PWD)
cargo-build-release:## 	cargo-build-release
	@echo "export PATH=$(CARGO_PATH)/bin:$(PATH)"
	@. $(HOME)/.cargo/env
	@cargo b --release
cargo-check:## 	cargo-check
	@echo "export PATH=$(CARGO_PATH)/bin:$(PATH)"
	@. $(HOME)/.cargo/env
	@cargo c
cargo-bench:## 	cargo-bench
	@echo "export PATH=$(CARGO_PATH)/bin:$(PATH)"
	@. $(HOME)/.cargo/env
	@cargo bench
cargo-test:## 	cargo-test
	@echo "export PATH=$(CARGO_PATH)/bin:$(PATH)"
	@. $(HOME)/.cargo/env
	@cargo test
# vim: set noexpandtab:
# vim: set setfiletype make
