#!/bin/bash

# Install cargo-binstall
curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash && cargo binstall just

# Example: Install a configuration file.
INSTALL_DIR="$HOME/.my_app"
CONFIG_FILE="my_config.conf"

mkdir -p "$INSTALL_DIR"
cp "$1" "$INSTALL_DIR/$CONFIG_FILE" # $1 is the first argument passed to the script, likely the config file itself.

echo "Installed configuration to $INSTALL_DIR/$CONFIG_FILE"

# Example: add a directory to the PATH.
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
  echo 'export PATH="$PATH:$INSTALL_DIR"' >> "$HOME/.bashrc"
  echo "Added $INSTALL_DIR to PATH. Reload your shell."
fi

#!/usr/bin/env bash

# Name of the Makefile to be converted
MAKEFILE="Makefile"
rm $MAKEFILE 2>/dev/null || true
touch $MAKEFILE

tee -a $MAKEFILE <<EOF
help:
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?##/ {printf "\033[36m%-15s\033[0m %s\n", $\$1, $\$2}' \$(MAKEFILE_LIST)
	@echo

##
##===============================================================================
##all
## 	bin
all: 	bin### 	all
##bin
## 	cargo b --manifest-path Cargo.toml
bin: 	### 	bin
	cargo b --manifest-path Cargo.toml

##
##===============================================================================
##make cargo-*
cargo-help: 	### 	cargo-help
	@awk 'BEGIN {FS = ":.*?###"} /^[a-zA-Z_-]+:.*?###/ {printf "\033[36m%-15s\033[0m %s\n", $\$1, $\$2}' \$(MAKEFILE_LIST)
cargo-release-all: 	### 	cargo-release-all
## 	cargo-release-all recursively cargo build --release
	for t in */Cargo.toml;  do echo $\$t; cargo b -r -vv --manifest-path $\$t; done
	for t in ffi/*/Cargo.toml;  do echo $\$t; cargo b -r -vv --manifest-path $\$t; done
cargo-clean-release: 	### 	cargo-clean-release - clean release artifacts
## 	cargo-clean-release 	recursively cargo clean --release
	for t in *Cargo.toml;  do echo $\$t && cargo clean --release -vv --manifest-path $\$t 2>/dev/null; done
cargo-publish-all: 	### 	cargo-publish-all
## 	cargo-publish-all 	recursively publish rust projects
	for t in *Cargo.toml;  do echo $\$t; cargo publish -vv --manifest-path $\$t; done

cargo-install-bins:### 	cargo-install-bins
## 	cargo-install-all 	recursively cargo install -vv \$(SUBMODULES)
## 	*** cargo install -vv --force is NOT used.
## 	*** FORCE=--force cargo install -vv \$(FORCE) is used.
## 	*** FORCE=--force cargo install -vv \$(FORCE) --path <path>
## 	*** to overwrite deploy cargo.io crates.
	export RUSTFLAGS=-Awarning;  for t in \$(SUBMODULES); do echo $\$t; cargo install --bins --path  $\$t -vv \$(FORCE) 2>/dev/null || echo ""; done
	#for t in \$(SUBMODULES); do echo $\$t; cargo install -vv gnostr-$\$t --force || echo ""; done

cargo-build: 	## 	cargo build
## 	cargo-build q=true
	@. \$(HOME)/.cargo/env
	@RUST_BACKTRACE=all cargo b \$(QUIET)
cargo-install: 	### 	cargo install --path . \$(FORCE)
	@. \$(HOME)/.cargo/env
	@cargo install --path . \$(FORCE)
## 	cargo-br q=true
cargo-build-release: 	### 	cargo-build-release
## 	cargo-build-release q=true
	@. \$(HOME)/.cargo/env
	@cargo b --release \$(QUIET)
cargo-check: 	### 	cargo-check
	@. \$(HOME)/.cargo/env
	@cargo c
cargo-bench: 	### 	cargo-bench
	@. \$(HOME)/.cargo/env
	@cargo bench
cargo-test: 	### 	cargo-test
	@. \$(HOME)/.cargo/env
	#@cargo test
	cargo test
cargo-test-nightly: 	### 	cargo-test-nightly
	@. \$(HOME)/.cargo/env
	#@cargo test
	cargo +nightly test
cargo-report: 	### 	cargo-report
	@. \$(HOME)/.cargo/env
	cargo report future-incompatibilities --id 1
cargo-run: 	### 	cargo-run
	@. \$(HOME)/.cargo/env
	cargo run --bin make-just

##===============================================================================
cargo-dist: 	### 	cargo-dist -h
	cargo dist -h
cargo-dist-build: 	### 	cargo-dist-build
	RUSTFLAGS="--cfg tokio_unstable" cargo dist build
cargo-dist-manifest: 	### 	cargo dist manifest --artifacts=all
	cargo dist manifest --artifacts=all

# vim: set noexpandtab:
# vim: set setfiletype make
EOF


# Name of the output Justfile
JUSTFILE=".justfile"
rm $JUSTFILE 2>/dev/null || true
touch $JUSTFILE

# Check if the Makefile exists
if [ ! -f "$MAKEFILE" ]; then
    echo "Makefile not found"
    exit 1
fi

# Clear the Justfile content
> "$JUSTFILE"

# Add in the default recipe to Justfile
echo "default:" >> "$JUSTFILE"
echo -e "  just --choose\n" >> "$JUSTFILE"

# Read each line in the Makefile
while IFS= read -r line
do
    # Extract target names (lines ending with ':')
    if [[ "$line" =~ ^[a-zA-Z0-9_-]+: ]]; then
        # Extract the target name
        target_name=$(echo "$line" | cut -d':' -f1)

        # Write the corresponding recipe to Justfile
        echo "$target_name:" >> "$JUSTFILE"
        echo -e "  @make $target_name\n" >> "$JUSTFILE"
    fi
done < "$MAKEFILE"

echo "Successfully ported the Makefile to Justfile."
