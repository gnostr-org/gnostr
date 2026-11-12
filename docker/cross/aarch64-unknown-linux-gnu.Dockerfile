FROM ghcr.io/cross-rs/aarch64-unknown-linux-gnu:main

# glibc must be the SAME version on every installed architecture, and apt
# reconciles that by upgrading the native libc6 to the foreign candidate's
# version. The suite indexes currently skew: ports noble-security publishes
# aarch64 glibc 2.39-0ubuntu8.12 while the amd64 indexes top out at
# 2.39-0ubuntu8.9, so apt cannot upgrade the native side and rejects
# libc6:arm64 ("not installable"), breaking the whole install. Pin the
# foreign glibc to the native installed version; that version is always
# present in noble/noble-updates for both arches.
RUN dpkg --add-architecture arm64 \
 && apt-get update \
 && V=$(dpkg-query -W -f='${Version}' libc6:amd64) \
 && printf 'Package: libc6:arm64 libc6-dev:arm64\nPin: version %s\nPin-Priority: 1001\n' "$V" \
      > /etc/apt/preferences.d/glibc-multiarch \
 && apt-get install --assume-yes --no-install-recommends \
      clang \
      libclang-dev \
      llvm-dev \
      pkg-config \
      gcc-aarch64-linux-gnu \
      libssl-dev:arm64 \
      zlib1g-dev:arm64 \
 && LIBCLANG_SO=$(find /usr -name 'libclang.so' 2>/dev/null | head -1 || \
      find /usr -name 'libclang*.so*' 2>/dev/null | sort -V | tail -n 1) \
 && { [ -n "$LIBCLANG_SO" ] || { echo "ERROR: could not find libclang.so after installing libclang-dev" >&2; exit 1; }; } \
 && mkdir -p /usr/local/lib \
 && ln -sf "$LIBCLANG_SO" /usr/local/lib/libclang.so \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*

# Provide a stable LIBCLANG_PATH inside the container so that bindgen/librocksdb-sys
# can locate libclang without relying on host-side paths passed via Cross.toml.
ENV LIBCLANG_PATH=/usr/local/lib
