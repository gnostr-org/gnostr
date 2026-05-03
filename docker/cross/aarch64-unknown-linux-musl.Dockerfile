FROM ghcr.io/cross-rs/aarch64-unknown-linux-musl:main

RUN apt-get update \
 && apt-get install --assume-yes --no-install-recommends \
      clang \
      libclang-dev \
      llvm-dev \
      pkg-config \
      libssl-dev \
  && apt-get clean \
  && rm -rf /var/lib/apt/lists/*
