fmt:
    rustup install nightly-2024-01-11
    rustup component add rustfmt --toolchain nightly-2024-01-11
    cargo +nightly-2024-01-11 fmt --all -- --config format_code_in_doc_comments=true

check-fmt:
	@rustup install nightly-2024-01-11
	@rustup component add rustfmt --toolchain nightly-2024-01-11
	cargo +nightly-2024-01-11 fmt --all -- --config format_code_in_doc_comments=true --check

check: fmt check-crates check-crates-msrv

check-crates:
    @bash contrib/scripts/check-crates.sh

check-crates-msrv:
    @bash contrib/scripts/check-crates.sh msrv