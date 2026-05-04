FROM ghcr.io/cross-rs/aarch64-unknown-linux-musl:main

RUN set -eux; \
    apt-get update || { \
      sed -i \
        -e 's|http://archive.archive.ubuntu.com/ubuntu|http://archive.ubuntu.com/ubuntu|g' \
        /etc/apt/sources.list /etc/apt/sources.list.d/*.list 2>/dev/null || true; \
      apt-get update; \
    }; \
    apt-get install --assume-yes --no-install-recommends \
      clang \
      libclang-dev \
      llvm-dev \
      pkg-config; \
    apt-get clean; \
    rm -rf /var/lib/apt/lists/*
