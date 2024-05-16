ifeq ($(project),)
PROJECT_NAME                            := $(notdir $(PWD))
else
PROJECT_NAME                            := $(project)
endif
export PROJECT_NAME

## https://doc.rust-lang.org/cargo/reference/profiles.html#custom-profiles
## CARGO_PROFILE_RELEASE_DEBUG
ifeq ($(profile),)
PROFILE=release
else
PROFILE=release-with-debug
endif

OS                                      :=$(shell uname -s)
export OS
OS_VERSION                              :=$(shell uname -r)
export OS_VERSION
ARCH                                    :=$(shell uname -m)
export ARCH
ifeq ($(ARCH),x86_64)
TRIPLET                                 :=x86_64-linux-gnu
export TRIPLET
endif
ifeq ($(ARCH),arm64)
TRIPLET                                 :=aarch64-linux-gnu
export TRIPLET
endif
ifeq ($(ARCH),arm64)
TRIPLET                                 :=aarch64-linux-gnu
export TRIPLET
endif

ifeq ($(reuse),true)
REUSE                                   :=-r
else
REUSE                                   :=
endif
export REUSE
ifeq ($(bind),true)
BIND                                   :=-b
else
BIND                                   :=
endif
export BIND

ifeq ($(token),)
GITHUB_TOKEN                            :=$(shell cat ~/GITHUB_TOKEN.txt || echo "0")
else
GITHUB_TOKEN                            :=$(shell echo $(token))
endif
export GITHUB_TOKEN

export $(cat ~/GITHUB_TOKEN) && make act

PYTHON                                  := $(shell which python)
export PYTHON
PYTHON2                                 := $(shell which python2)
export PYTHON2
PYTHON3                                 := $(shell which python3)
export PYTHON3

PIP                                     := $(shell which pip)
export PIP
PIP2                                    := $(shell which pip2)
export PIP2
PIP3                                    := $(shell which pip3)
export PIP3

PYTHON_VENV                             := $(shell python -c "import sys; sys.stdout.write('1') if hasattr(sys, 'base_prefix') else sys.stdout.write('0')")
PYTHON3_VENV                            := $(shell python3 -c "import sys; sys.stdout.write('1') if hasattr(sys, 'real_prefix') else sys.stdout.write('0')")

python_version_full := $(wordlist 2,4,$(subst ., ,$(shell python3 --version 2>&1)))
python_version_major := $(word 1,${python_version_full})
python_version_minor := $(word 2,${python_version_full})
python_version_patch := $(word 3,${python_version_full})

my_cmd.python.3 := $(PYTHON3) some_script.py3
my_cmd := ${my_cmd.python.${python_version_major}}

PYTHON_VERSION                         := ${python_version_major}.${python_version_minor}.${python_version_patch}
PYTHON_VERSION_MAJOR                   := ${python_version_major}
PYTHON_VERSION_MINOR                   := ${python_version_minor}

export python_version_major
export python_version_minor
export python_version_patch
export PYTHON_VERSION

CARGO:=$(shell which cargo)
export CARGO

-:
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?##/ {printf "\033[36m%-15s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)
help:## 	help
	@sed -n 's/^##//p' ${MAKEFILE_LIST} | column -t -s ':' |  sed -e 's/^/ /'
rustup-install:## 	rustup-install
##	install rustup sequence
	$(shell echo which rustup) || curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | bash -s -- -y --no-modify-path --default-toolchain nightly --profile default & . "$(HOME)/.cargo/env"
	$(shell echo which rustup) && rustup default nightly


cargo-b:## 	cargo-b
	@type -P rustc || $(MAKE) rustup-install
	cargo b
cargo-build-release:cargo-b-release## 	cargo-build-release
cargo-b-release:## 	cargo-b-release
	@type -P rustc || $(MAKE) rustup-install
	cargo build --release
cargo-c:## 	cargo-c
	@type -P rustc || $(MAKE) rustup-install
	cargo c
install: cargo-install## 	install
cargo-i:## 	cargo-i
	@type -P rustc || $(MAKE) rustup-install
	cargo install --path .



test-weeble:## 	test-weeble
	@export weeble=$(shell gnostr-sha256 $(shell gnostr-weeble)) && \
    gnostr --sec $$weeble -t gnostr --tag weeble $(shell gnostr-weeble) --envelope --content "sha256($(shell gnostr-weeble))" | gnostr-cat wss://nos.lol
test-weeble-blockheight:## 	test-weeble-blckheight
	@export weeble_blockheight=$(shell gnostr-sha256 $(shell gnostr-weeble)$(shell gnostr-blockheight)) && gnostr --sec $$weeble_blockheight -t gnostr --tag weeble $(shell gnostr-weeble) --envelope --content "sha256($(shell gnostr-weeble)||$(shell gnostr-blockheight))" | gnostr-cat wss://nos.lol

nip-zero-nos-weeble:## 	nip-zero-nos-weeble
	@export weeble=$(shell gnostr-sha256 $(shell gnostr-weeble)) && \
    gnostr \
    --sec $$weeble \
    -t gnostr \
    --tag weeble $(shell gnostr-weeble) \
    --tag weeble $(shell gnostr-blockheight) \
    --tag wobble $(shell gnostr-wobble) \
    --kind 0 \
    --envelope \
    --content "{\"content\":\"{\"name\":\"gnostr-weeble\",\"about\": \"#gnostr\\ngnostr-sha256 $(gnostr-weeble)\",\"picture\":\"https://avatars.githubusercontent.com/u/135379339?s=200&v=4\",\"nip05\":\"null\"}" \
  | gnostr-cat wss://nos.lol | jq .[1]
nip-zero-nos-weeble-blockheight:## 	nip-zero-nos-weeble-blockheight
	@export weeble_blockheight=$(shell gnostr-sha256 $(shell gnostr-weeble)$(shell gnostr-blockheight)) && \
    gnostr \
    --sec $$weeble_blockheight \
    -t gnostr \
    --tag weeble $(shell gnostr-weeble) \
    --tag weeble $(shell gnostr-blockheight) \
    --tag wobble $(shell gnostr-wobble) \
    --kind 0 \
    --envelope \
    --content "{\"content\":\"{\"name\":\"gnostr-weeble\",\"about\": \"#gnostr\\ngnostr-sha256 $(gnostr-weeble)\",\"picture\":\"https://avatars.githubusercontent.com/u/135379339?s=200&v=4\",\"nip05\":\"null\"}" \
   | gnostr-cat wss://nos.lol | jq .[1]
nip-zero-damus-weeble:## 	nip-zero-damus-weeble
	@export weeble=$(shell gnostr-sha256 $(shell gnostr-weeble)) && \
    gnostr \
    --sec $$weeble \
    -t gnostr \
    --tag weeble $(shell gnostr-weeble) \
    --tag weeble $(shell gnostr-blockheight) \
    --tag wobble $(shell gnostr-wobble) \
    --kind 0 \
    --envelope \
    --content "{\"content\":\"{\"name\":\"gnostr-weeble\",\"about\": \"#gnostr\\ngnostr-sha256 $(gnostr-weeble)\",\"picture\":\"https://avatars.githubusercontent.com/u/135379339?s=200&v=4\",\"nip05\":\"null\"}" \
  | gnostr-cat wss://relay.damus.io | jq .[1] && \
  echo $$weeble
nip-zero-damus-weeble-blockheight:## 	nip-zero-damus-weeble-blockheight
	@export weeble_blockheight=$(shell gnostr-sha256 $(shell gnostr-weeble)$(shell gnostr-blockheight)) && \
    gnostr \
    --sec $$weeble_blockheight \
    -t gnostr \
    --tag weeble $(shell gnostr-weeble) \
    --tag weeble $(shell gnostr-blockheight) \
    --tag wobble $(shell gnostr-wobble) \
    --kind 0 \
    --envelope \
    --content "{\"content\":\"{\"name\":\"gnostr-weeble_blockheight\",\"about\": \"#gnostr\\ngnostr-sha256 $(gnostr-weeble)$(gnostr-blockheight)\",\"picture\":\"https://avatars.githubusercontent.com/u/135379339?s=200&v=4\",\"nip05\":\"null\"}" \
  | gnostr-cat wss://relay.damus.io | jq .[1] && \
  echo $$weeble_blockheight

nip-zero-roundtrip-nos:## 	nip-0-roundtrip
	@gnostr-query -i $(shell echo $(shell echo $(shell make nip-zero-nos-weeble) | sed 's/\"//g')) | gnostr-cat wss://nos.lol | jq

nip-zero-roundtrip-damus:## 	nip-zero-roundtrip-damus
	@gnostr-query -i $(shell echo $(shell echo $(shell make nip-zero-nos-weeble) | sed 's/\"//g')) | gnostr-cat wss://relay.damus.io | jq

-include Makefile
-include cargo.mk
-include act.mk
