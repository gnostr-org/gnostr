FROM ghcr.io/cross-rs/x86_64-unknown-linux-musl:main

RUN set -eux; \
    DEBIAN_FRONTEND=noninteractive apt-get update || { \
      sed -i \
        -e 's|http://archive.archive.ubuntu.com/ubuntu|http://archive.ubuntu.com/ubuntu|g' \
        /etc/apt/sources.list /etc/apt/sources.list.d/*.list 2>/dev/null || true; \
      DEBIAN_FRONTEND=noninteractive apt-get update; \
    }; \
    DEBIAN_FRONTEND=noninteractive apt-get install --assume-yes --no-install-recommends \
      clang \
      libclang-dev \
      llvm-dev \
      pkg-config; \
    apt-get clean; \
    rm -rf /var/lib/apt/lists/*
