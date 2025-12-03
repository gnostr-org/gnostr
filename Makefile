export HOMEBREW_NO_INSTALL_CLEANUP=1
ifeq ($(TAG),)
TAG := v$(shell cat Cargo.toml | grep 'version = "' | head -n 1 | sed 's/version = "\(.*\)".*/\1/')
endif
export TAG

ifeq ($(NPROC),)
NPROC := $(shell sysctl -n hw.logicalcpu 2>/dev/null || nproc 2>/dev/null || echo 1)
endif
export NPROC
JFLAGS := -j$(NPROC)

ifeq ($(FORCE),)
       FORCE :=-f
endif
export FORCE

help:
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?##/ {printf "\033[36m%-15s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)
	@echo

##
##===============================================================================
##all
## 	bin
all: 	bin### 	all
##bin
## 	cargo b -j $(NPROC)
bin: 	### 	bin
	cargo b -j $(NPROC)

##
##===============================================================================
##make cargo-*
cargo-help: 	### 	cargo-help
	@awk 'BEGIN {FS = ":.*?###"} /^[a-zA-Z_-]+:.*?###/ {printf "\033[36m%-15s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)
cargo-release-all: 	### 	cargo-release-all
## 	cargo-release-all recursively cargo build --release
	for t in **Cargo.toml;  do echo $$t; cargo b -r -j $(NPROC) -vv --manifest-path $$t; done
cargo-clean-release: 	### 	cargo-clean-release - clean release artifacts
## 	cargo-clean-release 	recursively cargo clean --release
	for t in **Cargo.toml;  do echo $$t && cargo clean -r  -j $(NPROC) -vv --manifest-path $$t 2>/dev/null; done
cargo-publish-all: 	### 	cargo-publish-all
## 	cargo-publish-all 	recursively publish rust projects
	for t in *Cargo.toml;  do echo $$t; cargo publish -j $(NPROC) -vv --manifest-path $$t; done

cargo-install-bins:### 	cargo-install-bins
## 	cargo-install-all 	recursively cargo install -vv $(SUBMODULES)
## 	*** cargo install -vv --force is NOT used.
## 	*** FORCE=--force cargo install -vv $(FORCE) is used.
## 	*** FORCE=--force cargo install -vv $(FORCE) --path <path>
## 	*** to overwrite deploy cargo.io crates.
	export RUSTFLAGS=-Awarning;  for t in $(SUBMODULES); do echo $$t; cargo  -j $(NPROC) install --bins --path  $$t -vv $(FORCE) 2>/dev/null || echo ""; done
	#for t in $(SUBMODULES); do echo $$t; cargo install -j $(NPROC) -vv gnostr-$$t --force || echo ""; done

cargo-build: 	## 	cargo build
## 	cargo-build q=true
	@. $(HOME)/.cargo/env
	@RUST_BACKTRACE=all cargo b -j $(NPROC) $(QUIET)
cargo-install: 	crawler asyncgit 	###         cargo install --path . $(FORCE)
	@. $(HOME)/.cargo/env
	@cargo install -j $(NPROC) --path . $(FORCE)

cargo-sort: 	cargo-sort
	for cargo_toml in $(shell ls */Cargo.toml); do cargo sort -n $(cargo_toml);done

.PHONY:crawler asyncgit relay query
crawler: 	###     crawler
	@cargo install -j $(NPROC) --path ./crawler $(FORCE)
asyncgit: 	###     asyncgit
	@cargo  install -j $(NPROC) --path ./asyncgit $(FORCE)
relay: 	###     relay
	@cargo install -j $(NPROC) --path ./relay $(FORCE)
query: 	###     query
	@cargo install -j $(NPROC) --path ./query $(FORCE)

## 	cargo-br q=true
cargo-build-release: 	### 	cargo-build-release
## 	cargo-build-release q=true
	@. $(HOME)/.cargo/env
	@cargo b -r -j $(NPROC) $(QUIET)
cargo-check: 	### 	cargo-check
	@. $(HOME)/.cargo/env
	@cargo  c -j $(NPROC)
cargo-bench: 	### 	cargo-bench
	@. $(HOME)/.cargo/env
	@cargo bench -j $(NPROC)
cargo-test: 	### 	cargo-test
	@. $(HOME)/.cargo/env
	#@cargo test
	cargo  test -j $(NPROC)
cargo-test--ignored: 	### 	cargo-test--ignored
	@. $(HOME)/.cargo/env
	#@cargo test
	cargo  test -j $(NPROC) -- --ignored --nocapture
cargo-test-workspace: 	### 	cargo-test-workspace
	@. $(HOME)/.cargo/env
	#@cargo test
	cargo  test -j $(NPROC) --workspace
cargo-test-nightly: 	### 	cargo-test-nightly
	@. $(HOME)/.cargo/env
	#@cargo test
	cargo  +nightly test -j $(NPROC)
cargo-test-nightly-workspace: 	### 	cargo-test-nightly-workspace
	@. $(HOME)/.cargo/env
	#@cargo test
	cargo  +nightly test -j $(NPROC) --workspace

cargo-test-types-nip_three_four: 	### 	cargo-test-types-nip34
	@. $(HOME)/.cargo/env
	#@cargo test
	cargo test -j $(NPROC) -p gnostr -- --test-threads=1 --test types::nip34

cargo-clippy-workspace: 	### 	cargo-clippy-workspace
	cargo +nightly clippy --workspace --all-targets -- -D warnings
	cargo +nightly clippy --workspace --all-targets --all-features -- -D warnings

cargo-clippy-fix-workspace: 	### 	cargo-clippy-fix-workspace
	cargo +nightly clippy --allow-dirty --fix --workspace --all-targets -- -D warnings
	cargo +nightly clippy --allow-dirty --fix --workspace --all-targets --all-features -- -D warnings

cargo-report: 	### 	cargo-report
	@. $(HOME)/.cargo/env
	cargo report future-incompatibilities --id 1 -j $(NPROC)
cargo-run: 	### 	cargo-run
	@. $(HOME)/.cargo/env
	cargo run -j $(NPROC)  --bin gnostr -- -h

##===============================================================================
cargo-dist: 	### 	make cargo-dist TAG=$(TAG)
	
	@dist host --steps=create --tag=$(TAG) --allow-dirty --output-format=json > plan-dist-manifest.json
cargo-dist-build: 	### 	cargo-dist-build
	RUSTFLAGS="--cfg tokio_unstable" dist build --allow-dirty
cargo-dist-manifest: 	### 	dist manifest --artifacts=all
	dist manifest --artifacts=all

cargo-git-cliff-changelog: 	### 	cargo-git-cliff-changelog
	git-cliff --output CHANGELOG.md || cargo install git-cliff

dep-graph: 	### 	dep-graph
	@cargo  -j $(NPROC) depgraph --depth 1 | dot -Tpng > graph.png

gnostr-chat: 	## 	gnostr-chat
	cargo  b -j $(NPROC) --bin gnostr
	./target/debug/gnostr chat --topic gnostr --name "$(shell gnostr-weeble)/$(shell gnostr-blockheight)/$(shell gnostr-wobble):$(USER)"

fetch-by-id: 	### 	fetch-by-id
	cargo  -j $(NPROC) install --bin gnostr-fetch-by-id --path .
	event_id=$(shell gnostr note -c test --hex | jq .id | sed "s/\"//g") && gnostr-fetch-by-id $;

fetch-by-kind-and-author: 	### 	fetch-by-kind-and-author
	cargo  -j $(NPROC) install --bin fetch_by_kind-and-author --path .
	cargo  -j $(NPROC) install --bin fetch_by_kind_and_author --path .
	fetch_by_kind_and_author wss://relay.nostr.band 1 a34b99f22c790c4e36b2b3c2c35a36db06226e41c692fc82b8b56ac1c540c5bd

crawler-test-relays: 	### crawler-test-relays
	for relay in $(shell echo $(shell gnostr-crawler));do echo $$relay;done
	for relay in $(shell echo $(shell gnostr-crawler));do test_relay $$relay;done

gnostr-note-debug: 	### 	gnostr-note-debug
	@gnostr --debug --hash "" note -c "gnostr --debug" --hex -s "gnostr --debug subject" --ptag a34b99f22c790c4e36b2b3c2c35a36db06226e41c692fc82b8b56ac1c540c5bd --etag 8bd85322d47f896c1cc4b20887b08513a0c6065b997debe7f4e87cc949ee7686 -t "gnostr--debug|tag" --verbose --expiration 144000

gnostr-note-trace: 	### 	gnostr-note-debug
	@gnostr --trace --hash "" note -c "gnostr --trace" --hex -s "gnostr --trace subject" --ptag a34b99f22c790c4e36b2b3c2c35a36db06226e41c692fc82b8b56ac1c540c5bd --etag 8bd85322d47f896c1cc4b20887b08513a0c6065b997debe7f4e87cc949ee7686 -t "gnostr--trace|tag" --verbose --expiration 144000

post_event: 	### 	post_event
	cat tests/events/json/first-gnostr-commit.json | post_event wss://relay.nostr.band

post_from_files: 	### 	post_from_files
	post_from_files ./tests/events/json wss://relay.nostr.band

broadcast_event_list: 	### 	nip-0034-events syndication example
	@RUST_LOG=debug gnostr --debug list-events -k 30617 -k 30618 -k 1617 -k 1621 -k 1630 -k 1631 -k 1632 -k 1633 -o nip-0034.json && gnostr --debug  --nsec $(shell gnostr-sha256) broadcast-events  -f ./nip-0034.json

nip_thirty_four_requests: 	### 	nip_thirty_four_requests
	@echo '["REQ","nip-0034",{"kinds":[1630,1632,1621,30618,1633,1631,1617,30617]}]' | gnostr-cat -k -t -n -B 210000 wss://relay.nostr.band | jq


plan-dist-manifest: 	### 	plan-dist-manifest
	dist host --allow-dirty --steps=create --tag=$(TAG) --output-format=json | sed 's/windows-2019/windows-latest/g' | sed 's/ubuntu-20.04/ubuntu-latest/g' > plan-dist-manifest.json

docker: 	### 	gnostr in a docker container
	docker buildx build . -t gnostr:latest && docker run  -it gnostr:latest -c "git init && git config --global init.defaultBranch gnostr && gnostr chat --name gnostr-docker-$(shell gnostr-wobble) --topic gnostr"
docker-tui: 	### 	gnostr tui in a docker container
	docker buildx build . -t gnostr:latest && docker run  -it gnostr:latest -c "git init && git config --global init.defaultBranch gnostr && gnostr tui --gitdir ."
docker-chat: 	### 	gnostr chat in a docker container
	docker buildx build . -t gnostr:latest && docker run  -it gnostr:latest -c "git init && git config --global init.defaultBranch gnostr && gnostr chat --name gnostr-docker-$(shell gnostr-wobble) --topic gnostr"
docker-shared: 	### 	docker container with volumes
	docker buildx build . -t gnostr:latest && docker run -it --privileged -v /Users/Shared:/Users/Shared -v /Users/git:/Users/git gnostr:latest

gh-act-run-all: 	### 	gh-act-run-all
	gh extension install nektos/gh-act
	gh act -vv -W .github/workflows/run-all-workflows.yml --container-architecture linux/amd64 || 	act -vv -W .github/workflows/run-all-workflows.yml --container-architecture linux/amd64
gnostr-bot-matrix: 	### 	gnostr-bot-matrix
	act -vv -W .github/workflows/gnostr-bot-matrix.yml --container-architecture linux/amd64 || 	gh act -vv --container-architecture linux/amd64 -W .github/workflows/gnostr-bot-matrix.yml

# vim: set noexpandtab:
# vim: set setfiletype make
