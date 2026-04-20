#!/usr/bin/env sh

set -eu

: "${BLOSSOM_BIND:=0.0.0.0:3000}"
: "${BLOSSOM_BASE_URL:=http://localhost:3000}"
: "${BLOSSOM_DATA_DIR:=/var/lib/blossom/blobs}"
: "${BLOSSOM_DB_PATH:=/var/lib/blossom/blossom.db}"
: "${BLOSSOM_LOG_LEVEL:=info}"

mkdir -p "$BLOSSOM_DATA_DIR" "$(dirname "$BLOSSOM_DB_PATH")"

set -- \
  blossom-server \
  --bind "$BLOSSOM_BIND" \
  --base-url "$BLOSSOM_BASE_URL" \
  --data-dir "$BLOSSOM_DATA_DIR" \
  --db-path "$BLOSSOM_DB_PATH" \
  --log-level "$BLOSSOM_LOG_LEVEL" \
  "$@"

if [ -n "${BLOSSOM_EXTRA_ARGS:-}" ]; then
  # Intentionally split BLOSSOM_EXTRA_ARGS into individual CLI arguments.
  # shellcheck disable=SC2086
  set -- "$@" ${BLOSSOM_EXTRA_ARGS}
fi

exec "$@"
