alias d := doc
alias l := nix-lint
alias uf := nix-update-flake-dependencies
alias uc := update-cargo-dependencies
#alias r := run
alias t := cargo-test
alias b := build
alias i := install
alias br := build-release
alias rr := run-release
alias cw := cargo-watch

default:
    @just --choose

clippy:
    cargo clippy --all-targets --all-features

nix-actionlint:
    nix develop .#actionlintShell --command actionlint

deny:
    cargo deny check

cargo-test:
    cargo test

nix-cargo-diet:
    nix develop .#lintShell --command cargo diet

nix-cargo-tarpaulin:
    nix develop .#lintShell --command cargo tarpaulin --out html --exclude-files "benches/*"

nix-cargo-public-api:
    nix develop .#lintShell --command cargo public-api

nix-cargo-diff:
    nix develop .#lintShell --command cargo public-api diff

nix-lint:
    nix develop .#lintShell --command cargo diet
    nix develop .#lintShell --command cargo deny check licenses sources
    nix develop .#lintShell --command typos
    nix develop .#lintShell --command lychee *.md
    nix develop .#fmtShell --command treefmt --fail-on-change
    nix develop .#lintShell --command cargo udeps
    nix develop .#lintShell --command cargo machete
    nix develop .#lintShell --command cargo outdated
    nix develop .#lintShell --command taplo lint
    nix develop .#actionlintShell --command actionlint --ignore SC2002
    cargo check --future-incompat-report
    nix flake check

build:
    cargo build

build-release:
    cargo build --release

install:
    cargo install --path .

run-release:build-release
    cargo run --release

doc:
    cargo doc --open --offline

# Update and then commit the `Cargo.lock` file
update-cargo-dependencies:
    cargo update
    git add Cargo.lock
    git commit Cargo.lock -m "update(cargo): \`Cargo.lock\`"

# Future incompatibility report, run regularly
cargo-future:
    cargo check --future-incompat-report

nix-update-flake-dependencies:
    nix flake update --commit-lock-file

cargo-watch:
    cargo watch -x check -x test -x build

# build all examples
nix-examples:
    nix develop --command $SHELL
    example_list=$(cargo build --example 2>&1 | sed '1,2d' | awk '{print $1}')

    # Build each example
    # shellcheck disable=SC2068
    for example in ${example_list[@]}; do
    cargo build --example "$example"
    done

nix-examples-msrv:
    set -x
    nix develop .#msrvShell --command
    rustc --version
    cargo --version
    example_list=$(cargo build --example 2>&1 | grep -v ":")

    # Build each example
    # shellcheck disable=SC2068
    for example in ${example_list[@]}; do
    cargo build --example "$example"
    done
