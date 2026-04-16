#!/usr/bin/env bash
set -e

CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

DEPTH=""
POW=""
while [[ "$#" -gt 0 ]]; do
    case $1 in
        --depth) DEPTH="$2"; shift ;;
        --pow) POW="$2"; shift ;;
        *) echo -e "${RED}Unknown parameter: $1${NC}"; exit 1 ;;
    esac
    shift
done

if [[ -z "$DEPTH" ]]; then
    echo "Usage: $0 --depth <N> [--pow <N>]"; exit 1
fi

if ! git diff-index --quiet HEAD --; then
    echo -e "${RED}Error: Worktree has uncommitted changes.${NC}"; exit 1
fi

echo -e "${CYAN}=== Starting Sovereign Rebase with Hash Reference (Depth: $DEPTH) ===${NC}"
TMP_DIR=$(mktemp -d)

for i in $(seq 0 $((DEPTH - 1))); do
    REV="HEAD~$i"
    OLD_HASH=$(git rev-parse "$REV")

    # Capture message body
    git log -1 --pretty=format:%B "$REV" > "$TMP_DIR/msg_$((DEPTH - 1 - i)).txt"
    # Capture the last line (the old nonce)
    OLD_NONCE=$(tail -n 1 "$TMP_DIR/msg_$((DEPTH - 1 - i)).txt")

    echo "$OLD_HASH" > "$TMP_DIR/hash_$((DEPTH - 1 - i)).txt"
    echo "$OLD_NONCE" > "$TMP_DIR/nonce_$((DEPTH - 1 - i)).txt"

    git format-patch -1 "$REV" --stdout > "$TMP_DIR/patch_$((DEPTH - 1 - i)).patch"
done

CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
git checkout "HEAD~$DEPTH" --quiet

for i in $(seq 0 $((DEPTH - 1))); do
    MSG_FILE="$TMP_DIR/msg_$i.txt"
    PATCH_FILE="$TMP_DIR/patch_$i.patch"
    OLD_HASH=$(cat "$TMP_DIR/hash_$i.txt")
    OLD_NONCE=$(cat "$TMP_DIR/nonce_$i.txt")

    echo -e "${CYAN}Rewriting commit $((i + 1)) (Old Hash: ${OLD_HASH:0:9})...${NC}"

    git apply --allow-empty --index "$PATCH_FILE"

    # Construct the re-mined message including the reference
    RAW_BODY=$(cat "$MSG_FILE")

    # Call git-legit with the original message + the reference block
    GIT_LEGIT_ARGS=(
        cargo run --bin git-legit --
        -m "$RAW_BODY" \
        -m "Ref: $OLD_HASH" \
        -m "Old Nonce: $OLD_NONCE"
    )

    if [[ -n "$POW" ]]; then
        GIT_LEGIT_ARGS+=(--pow "$POW")
    fi

    "${GIT_LEGIT_ARGS[@]}"

    rm "$PATCH_FILE" "$MSG_FILE" "$TMP_DIR/hash_$i.txt" "$TMP_DIR/nonce_$i.txt"
done

echo -e "\n${GREEN}=== Rebase Complete ===${NC}"
rm -rf "$TMP_DIR"
git checkout -b "$(gnostr --weeble)/$(gnostr --blockheight)/$(gnostr --wobble)-end-test"
