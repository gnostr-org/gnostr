#!/usr/bin/env bash
set -euo pipefail

if command -v pkg-config >/dev/null 2>&1 && pkg-config --exists rocksdb; then
  export ROCKSDB_COMPILE=0
  export ROCKSDB_LIB_DIR="$(pkg-config --variable=libdir rocksdb)"
  export ROCKSDB_INCLUDE_DIR="$(pkg-config --variable=includedir rocksdb)"
fi

exec "$@"
