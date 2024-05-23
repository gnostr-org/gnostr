cargo-build-release::cargo-br
cargo-br:
	cargo build --release -vv
cargo-install:cargo-i
cargo-i:
	cargo install --path . -vv $(FORCE)
