precommit:
	cargo fmt --all -- --config format_code_in_doc_comments=true
	cargo check
	cargo check --target wasm32-unknown-unknown
	cargo clippy
	cargo clippy --target wasm32-unknown-unknown

clean:
	cargo clean

loc:
	@echo "--- Counting lines of .rs files (LOC):" && find src/ -type f -name "*.rs" -exec cat {} \; | wc -l