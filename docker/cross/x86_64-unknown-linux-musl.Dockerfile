FROM ghcr.io/cross-rs/x86_64-unknown-linux-musl:main

RUN DEBIAN_FRONTEND=noninteractive apt-get update \
 && DEBIAN_FRONTEND=noninteractive apt-get install --assume-yes --no-install-recommends \
      -o Dpkg::Options::="--force-confdef" \
      -o Dpkg::Options::="--force-confold" \
      clang \
      libclang-dev \
      llvm-dev \
      pkg-config \
      musl-dev \
      musl-tools \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*
