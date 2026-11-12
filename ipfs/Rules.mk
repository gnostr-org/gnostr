TGT_BIN :=
CLEAN :=
COVERAGE :=
DISTCLEAN :=
TEST :=
TEST_SHORT :=
GOCC ?= go
PROTOC ?= protoc

all: help    # all has to be first defined target
.PHONY: all

include mk/git.mk # has to be before tarball.mk
include mk/tarball.mk
include mk/util.mk
include mk/golang.mk

# -------------------- #
#   extra properties   #
# -------------------- #

ifeq ($(TEST_FUSE),0)
	GOTAGS += nofuse
endif
export LIBP2P_TCP_REUSEPORT=false

# -------------------- #
#   act (GitHub Actions local runner)   #
# -------------------- #
ACT ?= act
ACT_ARCH ?= linux/amd64
ACT_EVENT ?= workflow_dispatch

# -------------------- #
#       sub-files      #
# -------------------- #
dir := bin
include $(dir)/Rules.mk

# tests need access to rules from plugin
dir := plugin
include $(dir)/Rules.mk

dir := test
include $(dir)/Rules.mk

dir := cmd/ipfs
include $(dir)/Rules.mk

dir := cmd/ipfswatch
include $(dir)/Rules.mk
# include this file only if coverage target is executed
# it is quite expensive
ifneq ($(filter coverage% clean distclean test/unit/gotest.junit.xml,$(MAKECMDGOALS)),)
	# has to be after cmd/ipfs due to PATH
	dir := coverage
	include $(dir)/Rules.mk
endif

# -------------------- #
#   universal rules    #
# -------------------- #

%.pb.go: %.proto bin/protoc-gen-gogofaster
	$(PROTOC) --gogofaster_out=. --proto_path=.:$(GOPATH)/src:$(dir $@) $<

# -------------------- #
#     core targets     #
# -------------------- #

build: $(TGT_BIN)
.PHONY: build

clean:
	rm -rf $(CLEAN)
.PHONY: clean

mod_tidy:
	@find . -name go.mod -execdir $(GOCC) mod tidy \;
.PHONY: mod_tidy

coverage: $(COVERAGE)
.PHONY: coverage



ffi:
	cd ffi && $(GOCC) build ./...
.PHONY: ffi

ffi-shared:
	cd ffi && $(GOCC) build -buildmode=c-shared -o libkubo.dylib .
.PHONY: ffi-shared

rust-ffi:
	cd ffi/rust && cargo build
.PHONY: rust-ffi

rust-ffi-test:
	cd test/rust-ffi-test && cargo run
.PHONY: rust-ffi-test

test_rust_ffi: rust-ffi-test
.PHONY: test_rust_ffi

swift-ffi:
	cd ffi/swift && make build
.PHONY: swift-ffi

test_swift_ffi:
	cd ffi/swift && make test
.PHONY: test_swift_ffi

xcode-ffi:
	cd ffi/swift && make build
	cd ffi/xcode && swift build -Xlinker -L../swift -Xlinker -lkubo_ffi -Xlinker -framework -Xlinker Security -Xlinker -framework -Xlinker CoreFoundation -Xlinker -lresolv -Xlinker -lpthread -Xlinker -ldl
.PHONY: xcode-ffi

xcode-app: xcode-ffi
	cd ffi/xcode && make app
.PHONY: xcode-app

distclean: clean
	rm -rf $(DISTCLEAN)
	git clean -ffxd
.PHONY: distclean

test: $(TEST)
.PHONY: test

test_short: $(TEST_SHORT)
.PHONY: test_short

deps:
.PHONY: deps

nofuse: GOTAGS += nofuse
nofuse: build
.PHONY: nofuse

install-all: cmd/ipfs-install
	$(MAKE) install-ipfswatch
.PHONY: install-all

install: cmd/ipfs-install
.PHONY: install

install-ipfswatch: cmd/ipfswatch-install
.PHONY: install-ipfswatch
install_unsupported: install
	@echo "/=======================================================================\\"
	@echo '|                                                                       |'
	@echo '| `make install_unsupported` is deprecated, use `make install` instead. |'
	@echo '|                                                                       |'
	@echo "\\=======================================================================/"
.PHONY: install_unsupported

uninstall:
	$(GOCC) clean -i ./cmd/ipfs
.PHONY: uninstall

supported:
	@echo "Currently supported platforms (from .github/build-platforms.yml):"
	@grep '^  - ' .github/build-platforms.yml | sed 's/^  - //' || (echo "Error: .github/build-platforms.yml not found"; exit 1)
.PHONY: supported

# -------------------- #
#   act targets        #
# -------------------- #

# Run a single GitHub Actions workflow file with act:
#   make run WORKFLOW=.github/workflows/interop.yml
#   make run .github/workflows/interop.yml
run:
ifdef WORKFLOW
	$(ACT) $(ACT_EVENT) -W $(WORKFLOW) --container-architecture $(ACT_ARCH)
else
	@echo 'Usage: make run WORKFLOW=.github/workflows/<name>.yml'
	@echo '       make run .github/workflows/<name>.yml'
endif
.PHONY: run

# Run all workflows for the default (push) event.
act:
	$(ACT) --container-architecture $(ACT_ARCH)
.PHONY: act

# List the jobs act would run.
act-list:
	$(ACT) -l --container-architecture $(ACT_ARCH)
.PHONY: act-list

# Run a specific job from a workflow:
#   make act-job WORKFLOW=.github/workflows/interop.yml JOB=interop-prep
act-job:
	$(ACT) $(ACT_EVENT) -W $(WORKFLOW) -j $(JOB) --container-architecture $(ACT_ARCH)
.PHONY: act-job

# Pattern rule: building any workflow file runs it through act, so a command like
# `make .github/workflows/interop.yml` (or `make run .github/workflows/interop.yml`)
# executes that workflow locally.
.github/workflows/%.yml: FORCE
	$(ACT) $(ACT_EVENT) -W $@ --container-architecture $(ACT_ARCH)
.github/workflows/%.yaml: FORCE
	$(ACT) $(ACT_EVENT) -W $@ --container-architecture $(ACT_ARCH)

FORCE:
.PHONY: FORCE

help:
	@echo 'DEPENDENCY TARGETS:'
	@echo ''
	@echo '  deps                 - Download dependencies using bundled gx'
	@echo '  test_sharness_deps   - Download and build dependencies for sharness'
	@echo ''
	@echo 'BUILD TARGETS:'
	@echo ''
	@echo '  all                  - print this help message'
	@echo '  build                - Build binary at ./cmd/ipfs/ipfs'
	@echo '  ipfswatch            - Build binary at ./cmd/ipfswatch/ipfswatch'
	@echo '  install-all          - Build ipfs and ipfswatch install into $$GOBIN'
	@echo '  install-ipfswatch    - Build and install ipfswatch into $$GOBIN'	@echo '  nofuse       - Build binary with no fuse support'
	@echo '  install              - Build binary and install into $$GOBIN'
	@echo '  mod_tidy             - Remove unused dependencies from go.mod files'
	@echo '  ffi                  - Build FFI Go package'
	@echo '  ffi-shared           - Build FFI as C shared library'
	@echo '  rust-ffi             - Build Rust FFI bindings'
	@echo '  rust-ffi-test        - Build and run Rust FFI integration tests'
	@echo '  test_rust_ffi        - Alias for rust-ffi-test'
	@echo '  swift-ffi            - Build Swift FFI bindings'
	@echo '  test_swift_ffi       - Run Swift FFI tests'
	@echo '  xcode-ffi            - Build Xcode/SwiftPM sample app'
	@echo '  xcode-app            - Build signed .app bundle'
#	@echo '  dist_install         - TODO: c.f. ./cmd/ipfs/dist/README.md'
	@echo ''
	@echo 'CLEANING TARGETS:'
	@echo ''
	@echo '  clean                - Remove files generated by build'
	@echo '  distclean            - Remove files that are no part of a repository'
	@echo '  uninstall            - Remove binary from $$GOPATH/bin'
	@echo ''
	@echo 'TESTING TARGETS:'
	@echo ''
	@echo '  test                 - Run all tests (test_go_fmt, test_unit, test_cli, test_sharness)'
	@echo '  test_short           - Run fast tests (test_go_fmt, test_unit)'
	@echo '  test_unit            - Run unit tests with coverage (excludes test/cli)'
	@echo '  test_cli             - Run CLI integration tests (requires built binary)'
	@echo '  test_fuse            - Run FUSE tests (requires /dev/fuse and fusermount)'
	@echo '  test_go_fmt          - Check Go source formatting'
	@echo '  test_go_build        - Build kubo for all platforms from .github/build-platforms.yml'
	@echo '  test_go_lint         - Run golangci-lint'
	@echo '  test_sharness        - Run sharness tests'
	@echo '  coverage             - Collect coverage info from unit tests and sharness'
	@echo '  test_rust_ffi        - Run Rust FFI integration tests'
	@echo '  test_swift_ffi       - Run Swift FFI tests'
	@echo
	@echo 'ACT (GITHUB ACTIONS) TARGETS:'
	@echo ''
	@echo '  run                  - Run a workflow with act (WORKFLOW=... or positional .yml)'
	@echo '  act                  - Run all workflows for the default (push) event'
	@echo '  act-list             - List the jobs act would run'
	@echo '  act-job              - Run one job (WORKFLOW=... JOB=...)'
	@echo
.PHONY: help
