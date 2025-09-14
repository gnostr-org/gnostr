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

crawler:
  @make crawler

asyncgit:
  @make asyncgit

relay:
  @make relay

query:
  @make query

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

dep-graph:
  @make dep-graph

gnostr-chat:
  @make gnostr-chat

fetch-by-id:
  @make fetch-by-id

fetch-by-kind-and-author:
  @make fetch-by-kind-and-author

crawler-test-relays:
  @make crawler-test-relays

gnostr-note-debug:
  @make gnostr-note-debug

gnostr-note-trace:
  @make gnostr-note-trace

post_event:
  @make post_event

post_from_files:
  @make post_from_files

broadcast_event_list:
  @make broadcast_event_list

nip_thirty_four_requests:
  @make nip_thirty_four_requests

plan-dist-manifest:
  @make plan-dist-manifest

docker:
  @make docker

docker-tui:
  @make docker-tui

docker-chat:
  @make docker-chat

