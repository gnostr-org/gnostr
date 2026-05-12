#!/usr/bin/env bash
set -euo pipefail

if ! bash -n "${BASH_SOURCE[0]}"; then
  exit 1
fi

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

BASE_IMAGE="${GNOSTR_STACK_BASE_IMAGE:-gnostr-stack-base:local}"
CONTAINER="${GNOSTR_STACK_CONTAINER:-gnostr-stack}"
TOPIC="${GNOSTR_STACK_TOPIC:-gnostr-dev}"
BUILD_ONLY="${GNOSTR_STACK_BUILD_ONLY:-false}"
FOLLOW_LOGS="${GNOSTR_STACK_FOLLOW:-false}"
COMMAND="${1:-up}"

usage() {
  cat <<'EOF'
Usage: docker-gnostr-stack.sh [up|down|logs|status|shell|restart]

Environment:
  GNOSTR_STACK_BASE_IMAGE   Base Docker image tag to build from docker/Dockerfile.gnostr
  GNOSTR_STACK_CONTAINER   Container name to manage
  GNOSTR_STACK_TOPIC       Chat topic for the headless peer
  GNOSTR_STACK_BUILD_ONLY  "true" to build the image and stop
  GNOSTR_STACK_FOLLOW      "true" to follow container logs after startup
EOF
}

build_image() {
  docker build -t "$BASE_IMAGE" -f docker/Dockerfile.gnostr .
}

wait_for_ready() {
  local attempt
  for attempt in $(seq 1 60); do
    if docker logs "$CONTAINER" 2>&1 | grep -q 'local p2p relay service started for chat'; then
      return 0
    fi
    sleep 2
  done

  docker logs "$CONTAINER" || true
  echo "headless chat container never became ready" >&2
  return 1
}

start_stack() {
  docker rm -f "$CONTAINER" >/dev/null 2>&1 || true
  docker run -d \
    --name "$CONTAINER" \
    -v "$ROOT_DIR:/workspace" \
    -w /workspace \
    --entrypoint gnostr \
    "$BASE_IMAGE" \
    chat \
    --debug \
    --headless \
    --topic "$TOPIC" >/dev/null

  wait_for_ready
  docker ps --filter "name=^/${CONTAINER}$" --format 'running {{.Names}} {{.Status}}'
  docker logs --tail 20 "$CONTAINER"

  if [[ "$FOLLOW_LOGS" == "true" ]]; then
    docker logs -f "$CONTAINER"
  fi
}

stop_stack() {
  docker rm -f "$CONTAINER"
}

show_status() {
  docker inspect --format 'running={{.State.Running}} status={{.State.Status}} image={{.Config.Image}}' "$CONTAINER"
}

show_logs() {
  docker logs -f "$CONTAINER"
}

open_shell() {
  docker exec -it "$CONTAINER" bash
}

case "$COMMAND" in
  up)
    build_image
    if [[ "$BUILD_ONLY" == "true" ]]; then
      exit 0
    fi
    start_stack
    ;;
  down)
    stop_stack
    ;;
  logs)
    show_logs
    ;;
  status)
    show_status
    ;;
  shell)
    open_shell
    ;;
  restart)
    stop_stack >/dev/null 2>&1 || true
    build_image
    start_stack
    ;;
  -h|--help|help)
    usage
    ;;
  *)
    echo "Unknown command: $COMMAND" >&2
    usage >&2
    exit 1
    ;;
esac
