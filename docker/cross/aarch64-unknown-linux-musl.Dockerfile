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
    LIBCLANG_SO=$(find /usr -name 'libclang.so' 2>/dev/null | head -1 || \
      find /usr -name 'libclang*.so*' 2>/dev/null | sort -V | tail -n 1); \
    { [ -n "$LIBCLANG_SO" ] || { echo "ERROR: could not find libclang.so after installing libclang-dev" >&2; exit 1; }; }; \
    mkdir -p /usr/local/lib; \
    ln -sf "$LIBCLANG_SO" /usr/local/lib/libclang.so; \
    apt-get clean; \
    rm -rf /var/lib/apt/lists/*

# Provide a stable LIBCLANG_PATH inside the container so that bindgen/librocksdb-sys
# can locate libclang without relying on host-side paths passed via Cross.toml.
ENV LIBCLANG_PATH=/usr/local/lib
