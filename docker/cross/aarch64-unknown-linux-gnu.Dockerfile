FROM ghcr.io/cross-rs/aarch64-unknown-linux-gnu:main

RUN dpkg --add-architecture arm64 \
 && apt-get update \
 && apt-get install --assume-yes --no-install-recommends \
      clang \
      libclang-dev \
      llvm-dev \
      pkg-config \
      gcc-aarch64-linux-gnu \
      g++-aarch64-linux-gnu \
      libssl-dev:arm64 \
      zlib1g-dev:arm64 \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*
