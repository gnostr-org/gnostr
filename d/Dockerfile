# syntax=docker/dockerfile:1.5-labs
FROM rust:1.75.0 as builder

RUN rustup target add x86_64-unknown-linux-musl
RUN --mount=type=cache,target=/var/cache/apt apt-get update && apt-get install -y musl-dev musl-tools

# Run with access to the target cache to speed up builds
RUN git clone --depth 1 --recurse-submodules --branch v0.0.43 https://github.com/gnostr-org/gnostr.git /workspace
WORKDIR /workspace
RUN --mount=type=cache,target=target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    cargo build --release --manifest-path ./d/Cargo.toml --bin gnostrd --target x86_64-unknown-linux-musl

RUN --mount=type=cache,target=target \
    mv ./target/x86_64-unknown-linux-musl/release/gnostrd /usr/local/bin/gnostrd

FROM alpine:3
WORKDIR /app
COPY --from=builder /usr/local/bin/gnostrd /usr/bin/gnostrd
RUN --mount=type=cache,target=/var/cache/apk apk add bind-tools

ENV RUST_BACKTRACE=1

CMD ["gnostrd"]
