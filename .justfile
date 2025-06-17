default:
  just --choose

help:
  @make help

all:
  @make all

bin:
  @make bin

cargo-help:
  @make cargo-help

cargo-release-all:
  @make cargo-release-all

cargo-clean-release:
  @make cargo-clean-release

cargo-publish-all:
  @make cargo-publish-all

cargo-install-bins:
  @make cargo-install-bins

cargo-build:
  @make cargo-build

cargo-install:
  @make cargo-install

cargo-build-release:
  @make cargo-build-release

cargo-check:
  @make cargo-check

cargo-bench:
  @make cargo-bench

cargo-test:
  @make cargo-test

cargo-test-nightly:
  @make cargo-test-nightly

cargo-report:
  @make cargo-report

cargo-run:
  @make cargo-run

cargo-dist:
  @make cargo-dist

cargo-dist-build:
  @make cargo-dist-build

cargo-dist-manifest:
  @make cargo-dist-manifest

crawler:
  @make crawler

asyncgit:
  @make asyncgit

dep-graph:
  @make dep-graph

fetch-by-id:
  @make fetch-by-id

fetch-by-kind-and-author:
  @make fetch-by-kind-and-author

crawler-test-relays:
  @make crawler-test-relays

