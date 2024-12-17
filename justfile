default:
    @just --list

build-all:
    cargo b

build-all-release:
    cargo b -r

install-all:
    cargo install --force --path .
