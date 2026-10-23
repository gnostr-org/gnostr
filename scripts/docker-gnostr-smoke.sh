#!/usr/bin/env bash
set -euo pipefail
# Build and smoke-test the gnostr-node Docker image.
IMAGE="${IMAGE:-gnostr-node-smoke}"
docker build -f docker/Dockerfile.gnostr-node -t "$IMAGE" .
CONTAINER=$(docker run -d -p 3333:3333 "$IMAGE")
trap 'docker rm -f "$CONTAINER" >/dev/null 2>&1 || true' EXIT
echo "Waiting for server..."
for i in $(seq 1 30); do
  if curl -sf http://localhost:3333/ >/dev/null; then
    echo "OK: server responding"
    exit 0
  fi
  sleep 2
done
echo "FAIL: server did not respond"
exit 1
