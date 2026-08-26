#!/usr/bin/env bash

set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

node --check asyncgit/src/lib/js/core.js && \
node --check asyncgit/src/lib/js/event.js && \
node --check asyncgit/src/lib/js/model.js && \
node --check asyncgit/src/lib/js/main.js && \
node --check asyncgit/src/lib/js/db.js && \
node --check asyncgit/src/lib/js/ui/fmt.js && \
node --check asyncgit/src/lib/js/ui/util.js && \
node --check asyncgit/src/lib/js/ui/render.js && \
node --check asyncgit/src/lib/js/ui/state.js
