cargo-build:## 	cargo-build
	@cargo b
cargo-install:## 	cargo-install
	@cargo install --path $(PWD)
	@echo "export PATH=$(CARGO_PATH)/bin:$(PATH)"
cargo-build-release:## 	cargo-build-release
	@cargo b --release
cargo-check:## 	cargo-check
	@cargo c
# vim: set noexpandtab:
# vim: set setfiletype make
